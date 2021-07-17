use super::{super::guards::*, Result};
use crate::{
    adapters::json,
    core::{prelude::*, usecases, util},
    infrastructure::{
        cfg::Cfg,
        db::{sqlite, tantivy},
        flows::prelude as flows,
    },
    ports::web::{notify::*, popular_tags_cache::PopularTagsCache},
};
use rocket::{self, request::Form, State};
use rocket_contrib::json::Json;
use std::time::Duration;

#[derive(FromForm, Clone)]
pub struct GetEntryQuery {
    org_tag: Option<String>,
}

#[get("/entries/<ids>?<query..>")]
pub fn get_entry(
    db: sqlite::Connections,
    ids: String,
    query: Form<GetEntryQuery>,
) -> Result<Vec<json::Entry>> {
    // TODO: Only lookup and return a single entity
    // TODO: Add a new method for searching multiple ids
    let ids = util::split_ids(&ids);
    if ids.is_empty() {
        return Ok(Json(vec![]));
    }
    let GetEntryQuery { ref org_tag } = query.into_inner();
    let results = {
        let db = db.shared()?;
        let places = usecases::load_places(&*db, &ids, org_tag.as_ref().map(String::as_str))?;
        let mut results = Vec::with_capacity(places.len());
        for (place, _) in places.into_iter() {
            let r = db.load_ratings_of_place(place.id.as_ref())?;
            results.push(json::entry_from_place_with_ratings(place, r));
        }
        results
    };
    Ok(Json(results))
}

// Limit the total number of recently changed entries to avoid cloning
// the whole database!!

const ENTRIES_RECECENTLY_CHANGED_MAX_COUNT: u64 = 1000;

const ENTRIES_RECECENTLY_CHANGED_MAX_AGE_IN_DAYS: i64 = 100;

const SECONDS_PER_DAY: i64 = 24 * 60 * 60;

#[get("/entries/recently-changed?<since>&<until>&<with_ratings>&<offset>&<limit>")]
pub fn get_entries_recently_changed(
    db: sqlite::Connections,
    since: Option<i64>, // in seconds
    until: Option<i64>, // in seconds
    with_ratings: Option<bool>,
    offset: Option<u64>,
    mut limit: Option<u64>,
) -> Result<Vec<json::Entry>> {
    let since_min = Timestamp::now().into_seconds()
        - ENTRIES_RECECENTLY_CHANGED_MAX_AGE_IN_DAYS * SECONDS_PER_DAY;
    let since = if let Some(since) = since {
        if since < since_min {
            log::warn!(
                "Maximum available age of recently changed entries exceeded: {} < {}",
                since,
                since_min
            );
            Some(since_min)
        } else {
            Some(since)
        }
    } else {
        Some(since_min)
    };
    debug_assert!(since.is_some());
    let mut total_count = 0;
    if let Some(offset) = offset {
        total_count += offset;
    }
    total_count += limit.unwrap_or(ENTRIES_RECECENTLY_CHANGED_MAX_COUNT);
    if total_count > ENTRIES_RECECENTLY_CHANGED_MAX_COUNT {
        log::warn!(
            "Maximum available number of recently changed entries exceeded: {} > {}",
            total_count,
            ENTRIES_RECECENTLY_CHANGED_MAX_COUNT
        );
        if let Some(offset) = offset {
            limit = Some(
                ENTRIES_RECECENTLY_CHANGED_MAX_COUNT
                    - offset.min(ENTRIES_RECECENTLY_CHANGED_MAX_COUNT),
            );
        } else {
            limit = Some(ENTRIES_RECECENTLY_CHANGED_MAX_COUNT);
        }
    } else {
        limit = Some(limit.unwrap_or(ENTRIES_RECECENTLY_CHANGED_MAX_COUNT - offset.unwrap_or(0)));
    }
    debug_assert!(limit.is_some());
    // Conversion from seconds (external) to milliseconds (internal)
    let params = RecentlyChangedEntriesParams {
        since: since.map(TimestampMs::from_seconds),
        until: until.map(TimestampMs::from_seconds),
    };
    let pagination = Pagination { offset, limit };
    let results = {
        let db = db.shared()?;
        let entries = db.recently_changed_places(&params, &pagination)?;
        if with_ratings.unwrap_or(false) {
            let mut results = Vec::with_capacity(entries.len());
            for (place, _, _) in entries.into_iter() {
                let r = db.load_ratings_of_place(place.id.as_ref())?;
                results.push(json::entry_from_place_with_ratings(place, r));
            }
            results
        } else {
            entries
                .into_iter()
                .map(|(place, _, _)| json::entry_from_place_with_ratings(place, vec![]))
                .collect()
        }
    };
    Ok(Json(results))
}

const ENTRIES_MOST_POPULAR_TAGS_PAGINATION_LIMIT_MAX: u64 = 1000;
const ENTRIES_MOST_POPULAR_TAGS_DEFAULT_MAX_CACHE_AGE_SECONDS: u64 = 3600;

#[get("/entries/most-popular-tags?<min_count>&<max_count>&<offset>&<limit>&<max_cache_age>&<like>")]
pub fn get_entries_most_popular_tags(
    db: sqlite::Connections,
    tags_cache: State<PopularTagsCache>,
    min_count: Option<u64>,
    max_count: Option<u64>,
    offset: Option<u64>,
    limit: Option<u64>,
    like: Option<String>,
    max_cache_age: Option<u64>,
) -> Result<Vec<json::TagFrequency>> {
    let params = MostPopularTagsParams {
        min_count,
        max_count,
        like,
    };
    let limit = Some(
        limit
            .unwrap_or(ENTRIES_MOST_POPULAR_TAGS_PAGINATION_LIMIT_MAX)
            .min(ENTRIES_MOST_POPULAR_TAGS_PAGINATION_LIMIT_MAX),
    );
    let pagination = Pagination { offset, limit };
    let max_cache_age =
        max_cache_age.unwrap_or(ENTRIES_MOST_POPULAR_TAGS_DEFAULT_MAX_CACHE_AGE_SECONDS);

    let results = tags_cache.most_popular_place_revision_tags(
        &db,
        &params,
        &pagination,
        Duration::from_secs(max_cache_age),
    )?;
    Ok(Json(results))
}

#[post("/entries", format = "application/json", data = "<body>")]
pub fn post_entry(
    auth: Auth,
    connections: sqlite::Connections,
    notify: Notify,
    mut search_engine: tantivy::SearchEngine,
    body: Json<json::NewPlace>,
    cfg: State<Cfg>,
) -> Result<String> {
    let org = auth.organization(&*connections.shared()?).ok();
    if org.is_none() && auth.account_email().is_err() && cfg.protect_with_captcha {
        auth.has_captcha()?;
    }
    let new_place = body.into_inner().into();
    Ok(Json(
        flows::create_place(
            &connections,
            &mut search_engine,
            &*notify,
            new_place,
            auth.account_email().ok(),
            org.as_ref(),
            &cfg,
        )?
        .id
        .to_string(),
    ))
}

#[put("/entries/<id>", format = "application/json", data = "<data>")]
pub fn put_entry(
    auth: Auth,
    connections: sqlite::Connections,
    mut search_engine: tantivy::SearchEngine,
    notify: Notify,
    id: String,
    data: Json<json::UpdatePlace>,
    cfg: State<Cfg>,
) -> Result<String> {
    let org = auth.organization(&*connections.shared()?).ok();
    if org.is_none() && auth.account_email().is_err() && cfg.protect_with_captcha {
        auth.has_captcha()?;
    }
    Ok(Json(
        flows::update_place(
            &connections,
            &mut search_engine,
            &*notify,
            id.into(),
            data.into_inner().into(),
            auth.account_email().ok(),
            org.as_ref(),
            &cfg,
        )?
        .id
        .into(),
    ))
}
