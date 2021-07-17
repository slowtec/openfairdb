use crate::{
    core::db::PlaceRepo,
    infrastructure::db::sqlite,
    ports::web::{MostPopularTagsParams, Pagination},
};
use anyhow::Result;
use chrono::{DateTime, Utc};
use ofdb_boundary::TagFrequency;
use std::{
    collections::HashMap,
    sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
    time::Duration,
};

type Request = (MostPopularTagsParams, Pagination);
type Cache = HashMap<Request, (DateTime<Utc>, Vec<TagFrequency>)>;

pub struct PopularTagsCache(RwLock<Cache>);

impl PopularTagsCache {
    pub fn new_from_db<R: PlaceRepo>(db: &R) -> Result<PopularTagsCache> {
        let params = MostPopularTagsParams::default();
        let pagination = Pagination::default();
        let cache = Self(RwLock::new(HashMap::new()));
        let _ = cache.query_and_update(db, &params, &pagination)?;
        Ok(cache)
    }

    pub fn most_popular_place_revision_tags(
        &self,
        db: &sqlite::Connections,
        params: &MostPopularTagsParams,
        pagination: &Pagination,
        max_cache_age: Duration,
    ) -> Result<Vec<TagFrequency>> {
        let cached_results = self.read().get(&(params.clone(), *pagination)).cloned();
        if let Some((created_at, data)) = cached_results {
            let age_in_seconds = (Utc::now() - created_at).num_seconds() as u64;
            if age_in_seconds < max_cache_age.as_secs() {
                return Ok(data);
            }
        }
        self.query_and_update(&*db.shared()?, params, pagination)
    }

    fn query_and_update<R: PlaceRepo>(
        &self,
        db: &R,
        params: &MostPopularTagsParams,
        pagination: &Pagination,
    ) -> Result<Vec<TagFrequency>> {
        let results = db
            .most_popular_place_revision_tags(&params, &pagination)?
            .into_iter()
            .map(Into::into)
            .collect::<Vec<_>>();
        let mut cache = self.write();
        cache.insert((params.clone(), *pagination), (Utc::now(), results.clone()));
        Ok(results)
    }

    fn read(&self) -> RwLockReadGuard<Cache> {
        match self.0.read() {
            Ok(guard) => guard,
            Err(poison_err) => {
                log::error!("A poisoned RwLockReadGuard for the PopularTagsCache was found.");
                poison_err.into_inner()
            }
        }
    }

    fn write(&self) -> RwLockWriteGuard<Cache> {
        match self.0.write() {
            Ok(guard) => guard,
            Err(poison_err) => {
                log::error!("A poisoned RwLockWriteGuard for the PopularTagsCache was found.");
                poison_err.into_inner()
            }
        }
    }
}
