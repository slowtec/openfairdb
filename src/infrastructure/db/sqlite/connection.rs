use super::{util::load_url, *};
use crate::core::prelude::*;
use anyhow::anyhow;
use chrono::prelude::*;
use diesel::{
    self,
    prelude::{Connection as DieselConnection, *},
    result::{DatabaseErrorKind, Error as DieselError},
};
use snailquote::escape;
use std::result;

type Result<T> = result::Result<T, RepoError>;

fn load_review_status(status: ReviewStatusPrimitive) -> Result<ReviewStatus> {
    ReviewStatus::try_from(status)
        .ok_or_else(|| RepoError::Other(anyhow!("Invalid review status: {}", status)))
}

fn load_place_revision_tags(
    conn: &SqliteConnection,
    place_revision_rowid: i64,
) -> Result<Vec<String>> {
    use schema::place_revision_tag::dsl;
    Ok(schema::place_revision_tag::table
        .filter(dsl::parent_rowid.eq(&place_revision_rowid))
        .load::<models::PlaceRevisionTag>(conn)?
        .into_iter()
        .map(
            |models::PlaceRevisionTag {
                 parent_rowid: _,
                 tag,
             }| tag,
        )
        .collect())
}

fn load_place_revision_custom_links(
    conn: &SqliteConnection,
    place_revision_rowid: i64,
) -> Result<Vec<CustomLink>> {
    use schema::place_revision_custom_link::dsl;
    Ok(schema::place_revision_custom_link::table
        .filter(dsl::parent_rowid.eq(&place_revision_rowid))
        .load::<models::PlaceRevisionCustomLink>(conn)?
        .into_iter()
        .filter_map(
            |models::PlaceRevisionCustomLink {
                 parent_rowid: _,
                 url,
                 title,
                 description,
             }| {
                url.parse()
                    .map_err(|err| {
                        // This should never happen if URLs have been validated properly on insert
                        log::error!("Failed to load custom link with invalid URL: {}", err);
                        err
                    })
                    .ok()
                    .map(|url| CustomLink {
                        url,
                        title,
                        description,
                    })
            },
        )
        .collect())
}

fn load_place(
    conn: &SqliteConnection,
    place: models::JoinedPlaceRevision,
) -> Result<(Place, ReviewStatus)> {
    let models::JoinedPlaceRevision {
        id,
        place_id,
        place_license: license,
        rev,
        created_at,
        created_by: created_by_id,
        current_status,
        title,
        desc: description,
        lat,
        lon,
        street,
        zip,
        city,
        country,
        state,
        contact_name,
        email,
        phone,
        homepage,
        opening_hours,
        founded_on,
        image_url,
        image_link_url,
        ..
    } = place;

    let location = Location {
        pos: MapPoint::try_from_lat_lng_deg(lat, lon).unwrap_or_default(),
        address: Some(Address {
            street,
            zip,
            city,
            country,
            state,
        }),
    };

    let tags = load_place_revision_tags(conn, id)?;

    let custom_links = load_place_revision_custom_links(conn, id)?;

    let created_by = if let Some(user_id) = created_by_id {
        use schema::users::dsl;
        Some(
            schema::users::table
                .select(dsl::email)
                .filter(dsl::id.eq(&user_id))
                .first::<String>(conn)?,
        )
    } else {
        None
    };

    let place = Place {
        id: place_id.into(),
        license,
        revision: Revision::from(rev as u64),
        created: Activity {
            at: TimestampMs::from_inner(created_at),
            by: created_by.map(Into::into),
        },
        title,
        description,
        location,
        contact: Some(Contact {
            name: contact_name,
            email: email.map(Into::into),
            phone,
        }),
        links: Some(Links {
            homepage: homepage.and_then(load_url),
            image: image_url.and_then(load_url),
            image_href: image_link_url.and_then(load_url),
            custom: custom_links,
        }),
        opening_hours: opening_hours.map(Into::into),
        founded_on,
        tags,
    };

    Ok((place, load_review_status(current_status)?))
}

fn load_place_with_status_review(
    conn: &SqliteConnection,
    place_with_status_review: models::JoinedPlaceRevisionWithStatusReview,
) -> Result<(Place, ReviewStatus, ActivityLog)> {
    let models::JoinedPlaceRevisionWithStatusReview {
        id,
        rev,
        created_at,
        created_by: created_by_id,
        title,
        desc: description,
        lat,
        lon,
        street,
        zip,
        city,
        country,
        state,
        contact_name,
        email,
        phone,
        homepage,
        opening_hours,
        founded_on,
        image_url,
        image_link_url,
        place_id,
        place_license: license,
        review_created_at,
        review_created_by: review_created_by_id,
        review_status,
        review_context,
        review_comment,
        review_rev: _,
    } = place_with_status_review;

    let location = Location {
        pos: MapPoint::try_from_lat_lng_deg(lat, lon).unwrap_or_default(),
        address: Some(Address {
            street,
            zip,
            city,
            country,
            state,
        }),
    };

    let tags = load_place_revision_tags(conn, id)?;

    let custom_links = load_place_revision_custom_links(conn, id)?;

    let created_by = if let Some(user_id) = created_by_id {
        use schema::users::dsl;
        Some(
            schema::users::table
                .select(dsl::email)
                .filter(dsl::id.eq(&user_id))
                .first::<String>(conn)?,
        )
    } else {
        None
    };

    let links = Links {
        homepage: homepage.and_then(load_url),
        image: image_url.and_then(load_url),
        image_href: image_link_url.and_then(load_url),
        custom: custom_links,
    };

    let contact = Contact {
        name: contact_name,
        email: email.map(Into::into),
        phone,
    };

    let review_created_by = if review_created_by_id == created_by_id {
        created_by.clone()
    } else if let Some(user_id) = review_created_by_id {
        use schema::users::dsl;
        Some(
            schema::users::table
                .select(dsl::email)
                .filter(dsl::id.eq(&user_id))
                .first::<String>(conn)?,
        )
    } else {
        None
    };

    let place = Place {
        id: place_id.into(),
        license,
        revision: Revision::from(rev as u64),
        created: Activity {
            at: TimestampMs::from_inner(created_at),
            by: created_by.map(Into::into),
        },
        title,
        description,
        location,
        contact: Some(contact),
        opening_hours: opening_hours.map(Into::into),
        founded_on,
        links: Some(links),
        tags,
    };

    let activity_log = ActivityLog {
        activity: Activity {
            at: TimestampMs::from_inner(review_created_at),
            by: review_created_by.map(Into::into),
        },
        context: review_context,
        comment: review_comment,
    };

    Ok((place, load_review_status(review_status)?, activity_log))
}

#[derive(QueryableByName)]
struct TagCountRow {
    #[sql_type = "diesel::sql_types::Text"]
    tag: String,

    #[sql_type = "diesel::sql_types::BigInt"]
    count: i64,
}

fn resolve_organization_rowid(conn: &SqliteConnection, id: &Id) -> Result<i64> {
    use schema::organization::dsl;
    Ok(schema::organization::table
        .select(dsl::rowid)
        .filter(dsl::id.eq(id.as_str()))
        .first::<i64>(conn)
        .map_err(|e| {
            log::warn!("Failed to resolve organization id '{}': {}", id, e);
            e
        })?)
}

fn resolve_place_rowid(conn: &SqliteConnection, id: &Id) -> Result<i64> {
    use schema::place::dsl;
    Ok(schema::place::table
        .select(dsl::rowid)
        .filter(dsl::id.eq(id.as_str()))
        .first::<i64>(conn)
        .map_err(|e| {
            log::warn!("Failed to resolve place id '{}': {}", id, e);
            e
        })?)
}

fn resolve_place_rowid_verify_revision(
    conn: &SqliteConnection,
    id: &Id,
    revision: Revision,
) -> Result<i64> {
    use schema::place::dsl;
    use schema::place_revision::dsl as rev_dsl;
    let revision = RevisionValue::from(revision);
    Ok(schema::place::table
        .inner_join(schema::place_revision::table)
        .select(dsl::rowid)
        .filter(dsl::id.eq(id.as_str()))
        .filter(rev_dsl::rev.eq(revision as i64))
        .first::<i64>(conn)
        .map_err(|e| {
            log::warn!(
                "Failed to resolve place id '{}' with revision {}: {}",
                id,
                revision,
                e
            );
            e
        })?)
}

fn resolve_place_rowid_with_current_revision(
    conn: &SqliteConnection,
    id: &Id,
) -> Result<(i64, Revision)> {
    use schema::place::dsl;
    Ok(schema::place::table
        .select((dsl::rowid, dsl::current_rev))
        .filter(dsl::id.eq(id.as_str()))
        .first::<(i64, i64)>(conn)
        .map_err(|e| {
            log::warn!("Failed to resolve place id '{}': {}", id, e);
            e
        })
        .map(|(id, rev)| (id, Revision::from(rev as u64)))?)
}

fn resolve_rating_rowid(conn: &SqliteConnection, id: &str) -> Result<i64> {
    use schema::place_rating::dsl;
    Ok(schema::place_rating::table
        .select(dsl::rowid)
        .filter(dsl::id.eq(id))
        .first::<i64>(conn)
        .map_err(|e| {
            log::warn!("Failed to resolve place rating id '{}': {}", id, e);
            e
        })?)
}

fn into_new_place_revision(
    conn: &SqliteConnection,
    place: Place,
) -> Result<(Id, models::NewPlaceRevision, Vec<String>, Vec<CustomLink>)> {
    let Place {
        id: place_id,
        license,
        revision: new_revision,
        created,
        title,
        description,
        location: Location { pos, address },
        contact,
        opening_hours,
        founded_on,
        tags,
        links,
    } = place;
    let parent_rowid = if new_revision.is_initial() {
        // Create a new place
        let new_place = models::NewPlace {
            id: place_id.as_ref(),
            license: &license,
            current_rev: u64::from(new_revision) as i64,
        };
        diesel::insert_into(schema::place::table)
            .values(new_place)
            .execute(conn)?;
        let (rowid, _revision) = resolve_place_rowid_with_current_revision(conn, &place_id)?;
        debug_assert_eq!(new_revision, _revision);
        rowid
    } else {
        // Update the existing place with a new revision
        let (rowid, revision) = resolve_place_rowid_with_current_revision(conn, &place_id)?;
        // Check for a contiguous revision history without conflicts (optimistic locking)
        if revision.next() != new_revision {
            return Err(RepoError::InvalidVersion);
        }
        use schema::place::dsl;
        let _count = diesel::update(
            schema::place::table
                .filter(dsl::rowid.eq(rowid))
                .filter(dsl::current_rev.eq(u64::from(revision) as i64)),
        )
        .set(dsl::current_rev.eq(u64::from(new_revision) as i64))
        .execute(conn)?;
        debug_assert_eq!(1, _count);
        rowid
    };
    let created_by = if let Some(ref email) = created.by {
        Some(resolve_user_created_by_email(conn, email.as_ref())?)
    } else {
        None
    };
    let Contact {
        name: contact_name,
        email,
        phone,
    } = contact.unwrap_or_default();
    debug_assert!(pos.is_valid());
    let Address {
        street,
        zip,
        city,
        country,
        state,
    } = address.unwrap_or_default();
    let Links {
        homepage,
        image: image_url,
        image_href: image_link_url,
        custom: custom_links,
    } = links.unwrap_or_default();
    let new_place = models::NewPlaceRevision {
        parent_rowid,
        rev: u64::from(new_revision) as i64,
        created_at: created.at.into_inner(),
        created_by,
        current_status: ReviewStatus::Created.into(),
        title,
        description,
        lat: pos.lat().to_deg(),
        lon: pos.lng().to_deg(),
        street,
        zip,
        city,
        country,
        state,
        contact_name,
        email: email.map(Into::into),
        phone,
        homepage: homepage.map(Into::into),
        opening_hours: opening_hours.map(Into::into),
        founded_on,
        image_url: image_url.map(Into::into),
        image_link_url: image_link_url.map(Into::into),
    };
    Ok((place_id, new_place, tags, custom_links))
}

impl PlaceRepo for SqliteConnection {
    fn create_or_update_place(&self, place: Place) -> Result<()> {
        let (_place_id, new_place, tags, custom_links) = into_new_place_revision(self, place)?;
        diesel::insert_into(schema::place_revision::table)
            .values(&new_place)
            .execute(self)?;

        use schema::place_revision::dsl;
        let parent_rowid = schema::place_revision::table
            .select(dsl::rowid)
            .filter(dsl::parent_rowid.eq(new_place.parent_rowid))
            .filter(dsl::rev.eq(new_place.rev))
            .first::<i64>(self)
            .map_err(|e| {
                log::warn!(
                    "Newly inserted place {} revision {} not found: {}",
                    new_place.parent_rowid,
                    new_place.rev,
                    e
                );
                e
            })?;

        // Insert into place_revision_review
        let new_review = models::NewPlaceReviewedRevision {
            parent_rowid,
            rev: u64::from(Revision::initial()) as i64,
            created_at: new_place.created_at,
            created_by: new_place.created_by,
            status: new_place.current_status,
            context: None,
            comment: Some("created"),
        };
        diesel::insert_into(schema::place_revision_review::table)
            .values(new_review)
            .execute(self)?;

        // Insert into place_revision_tag
        let insertable_tags: Vec<_> = tags
            .iter()
            .map(|tag| models::NewPlaceRevisionTag {
                parent_rowid,
                tag: tag.as_str(),
            })
            .collect();
        diesel::insert_into(schema::place_revision_tag::table)
            .values(&insertable_tags)
            .execute(self)?;

        // Insert into place_revision_custom_link
        let insertable_custom_links: Vec<_> = custom_links
            .iter()
            .map(
                |CustomLink {
                     url,
                     title,
                     description,
                 }| models::NewPlaceRevisionCustomLink {
                    parent_rowid,
                    url: url.as_str(),
                    title: title.as_ref().map(String::as_str),
                    description: description.as_ref().map(String::as_str),
                },
            )
            .collect();
        diesel::insert_into(schema::place_revision_custom_link::table)
            .values(&insertable_custom_links)
            .execute(self)?;

        Ok(())
    }

    fn review_places(
        &self,
        ids: &[&str],
        status: ReviewStatus,
        activity_log: &ActivityLog,
    ) -> Result<usize> {
        use schema::place::dsl;
        use schema::place_revision::dsl as rev_dsl;

        let rev_ids = schema::place_revision::table
            .inner_join(
                schema::place::table.on(rev_dsl::parent_rowid
                    .eq(dsl::rowid)
                    .and(rev_dsl::rev.eq(dsl::current_rev))),
            )
            .select(rev_dsl::rowid)
            .filter(dsl::id.eq_any(ids))
            .filter(rev_dsl::current_status.ne(ReviewStatusPrimitive::from(status)))
            .load(self)?;
        let ActivityLog {
            activity,
            context,
            comment,
        } = activity_log;
        let changed_at = activity.at.into_inner();
        let changed_by = if let Some(ref email) = activity.by {
            Some(resolve_user_created_by_email(self, email.as_ref())?)
        } else {
            None
        };
        let status = ReviewStatusPrimitive::from(status);
        let mut total_update_count = 0;
        for rev_id in rev_ids {
            let update_count = diesel::update(
                schema::place_revision::table
                    .filter(rev_dsl::rowid.eq(rev_id))
                    .filter(rev_dsl::current_status.ne(status)),
            )
            .set(rev_dsl::current_status.eq(status))
            .execute(self)?;
            debug_assert!(update_count <= 1);
            if update_count > 0 {
                use schema::place_revision_review::dsl as review_dsl;
                let prev_rev = Revision::from(
                    schema::place_revision_review::table
                        .select(diesel::dsl::max(review_dsl::rev))
                        .filter(review_dsl::parent_rowid.eq(rev_id))
                        .first::<Option<i64>>(self)?
                        .ok_or(RepoError::NotFound)? as u64,
                );
                let next_rev = prev_rev.next();
                let new_review = models::NewPlaceReviewedRevision {
                    parent_rowid: rev_id,
                    rev: u64::from(next_rev) as i64,
                    status,
                    created_at: changed_at,
                    created_by: changed_by,
                    context: context.as_deref(),
                    comment: comment.as_deref(),
                };
                diesel::insert_into(schema::place_revision_review::table)
                    .values(new_review)
                    .execute(self)?;
                total_update_count += update_count;
            }
        }
        Ok(total_update_count)
    }

    fn get_places(&self, place_ids: &[&str]) -> Result<Vec<(Place, ReviewStatus)>> {
        use schema::place::dsl;
        use schema::place_revision::dsl as rev_dsl;

        let mut query = schema::place_revision::table
            .inner_join(
                schema::place::table.on(rev_dsl::parent_rowid
                    .eq(dsl::rowid)
                    .and(rev_dsl::rev.eq(dsl::current_rev))),
            )
            .select((
                rev_dsl::rowid,
                rev_dsl::rev,
                rev_dsl::created_at,
                rev_dsl::created_by,
                rev_dsl::current_status,
                rev_dsl::title,
                rev_dsl::description,
                rev_dsl::lat,
                rev_dsl::lon,
                rev_dsl::street,
                rev_dsl::zip,
                rev_dsl::city,
                rev_dsl::country,
                rev_dsl::state,
                rev_dsl::contact_name,
                rev_dsl::email,
                rev_dsl::phone,
                rev_dsl::homepage,
                rev_dsl::opening_hours,
                rev_dsl::founded_on,
                rev_dsl::image_url,
                rev_dsl::image_link_url,
                dsl::id,
                dsl::license,
            ))
            .into_boxed();
        if place_ids.is_empty() {
            warn!("Loading all entries at once");
        } else {
            // TODO: Split loading into chunks of fixed size
            info!("Loading multiple ({}) entries at once", place_ids.len());
            query = query.filter(dsl::id.eq_any(place_ids));
        }

        let rows = query.load::<models::JoinedPlaceRevision>(self)?;
        let mut results = Vec::with_capacity(rows.len());
        for row in rows {
            results.push(load_place(self, row)?);
        }
        Ok(results)
    }

    fn get_place(&self, place_id: &str) -> Result<(Place, ReviewStatus)> {
        let places = self.get_places(&[place_id])?;
        debug_assert!(places.len() <= 1);
        places.into_iter().next().ok_or(RepoError::NotFound)
    }

    fn all_places(&self) -> Result<Vec<(Place, ReviewStatus)>> {
        self.get_places(&[])
    }

    fn recently_changed_places(
        &self,
        params: &RecentlyChangedEntriesParams,
        pagination: &Pagination,
    ) -> Result<Vec<(Place, ReviewStatus, ActivityLog)>> {
        use schema::place::dsl;
        use schema::place_revision::dsl as rev_dsl;
        use schema::place_revision_review::dsl as review_dsl;

        let mut query = schema::place_revision::table
            .inner_join(
                schema::place::table.on(rev_dsl::parent_rowid
                    .eq(dsl::rowid)
                    .and(rev_dsl::rev.eq(dsl::current_rev))),
            )
            .inner_join(
                schema::place_revision_review::table
                    .on(review_dsl::parent_rowid.eq(rev_dsl::rowid)),
            )
            .select((
                rev_dsl::rowid,
                rev_dsl::rev,
                rev_dsl::created_at,
                rev_dsl::created_by,
                rev_dsl::title,
                rev_dsl::description,
                rev_dsl::lat,
                rev_dsl::lon,
                rev_dsl::street,
                rev_dsl::zip,
                rev_dsl::city,
                rev_dsl::country,
                rev_dsl::state,
                rev_dsl::contact_name,
                rev_dsl::email,
                rev_dsl::phone,
                rev_dsl::homepage,
                rev_dsl::opening_hours,
                rev_dsl::founded_on,
                rev_dsl::image_url,
                rev_dsl::image_link_url,
                dsl::id,
                dsl::license,
                review_dsl::rev,
                review_dsl::created_at,
                review_dsl::created_by,
                review_dsl::status,
                review_dsl::context,
                review_dsl::comment,
            ))
            .order_by(review_dsl::created_at.desc())
            .then_order_by(review_dsl::rev.desc()) // disambiguation of equal time stamps
            .into_boxed();

        // Since (inclusive)
        if let Some(since) = params.since {
            query = query.filter(review_dsl::created_at.ge(since.into_inner()));
        }

        // Until (exclusive)
        if let Some(until) = params.until {
            query = query.filter(review_dsl::created_at.lt(until.into_inner()));
        }

        // Pagination
        let offset = pagination.offset.unwrap_or(0);
        if offset > 0 {
            query = query.offset(offset as i64);
        }
        if let Some(limit) = pagination.limit {
            query = query.limit(limit as i64);
        }

        let rows = query.load::<models::JoinedPlaceRevisionWithStatusReview>(self)?;
        let mut results = Vec::with_capacity(rows.len());
        for row in rows {
            results.push(load_place_with_status_review(self, row)?);
        }
        Ok(results)
    }

    fn most_popular_place_revision_tags(
        &self,
        params: &MostPopularTagsParams,
        pagination: &Pagination,
    ) -> Result<Vec<TagFrequency>> {
        // TODO: Diesel 1.4.x does not support the HAVING clause
        // that is required to filter the aggregated column.
        let mut sql = "SELECT tag, COUNT(*) as count \
                       FROM place_revision_tag \
                       WHERE {like_condition} parent_rowid IN \
                       (SELECT rowid FROM place_revision WHERE (parent_rowid, rev) IN (SELECT rowid, current_rev FROM place) AND current_status > 0) \
                       GROUP BY tag"
                       .to_string();
            

        let mut like_condition = "".to_string();
        if let Some(like) = &params.like {
            like_condition = format!("tag LIKE \"%{}%\" AND ", escape(like));
        }

        sql = sql.replace("{like_condition}", &like_condition[..]);


        if params.min_count.is_some() || params.max_count.is_some() {
            if let Some(min_count) = params.min_count {
                sql.push_str(&format!(" HAVING count>={}", min_count));
                if let Some(max_count) = params.max_count {
                    sql.push_str(&format!(" AND count<={}", max_count));
                }
            } else if let Some(max_count) = params.max_count {
                sql.push_str(&format!(" HAVING count<={}", max_count));
            }
        }
        sql.push_str(" ORDER BY count DESC, tag");
        if let Some(limit) = pagination.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
            // LIMIT must precede OFFSET, i.e. OFFSET without LIMIT
            // is not supported!
            let offset = pagination.offset.unwrap_or(0);
            if offset > 0 {
                sql.push_str(&format!(" OFFSET {}", offset));
            }
        }
        let rows = diesel::dsl::sql_query(sql).load::<TagCountRow>(self)?;
        Ok(rows
            .into_iter()
            .map(|row| TagFrequency(row.tag, row.count as TagCount))
            .collect())
    }

    fn count_places(&self) -> Result<usize> {
        use schema::place::dsl;
        use schema::place_revision::dsl as rev_dsl;
        Ok(schema::place_revision::table
            .inner_join(
                schema::place::table.on(rev_dsl::parent_rowid
                    .eq(dsl::rowid)
                    .and(rev_dsl::rev.eq(dsl::current_rev))),
            )
            .select(diesel::dsl::count(rev_dsl::parent_rowid))
            .filter(rev_dsl::current_status.ge(ReviewStatusPrimitive::from(ReviewStatus::Created)))
            .first::<i64>(self)? as usize)
    }

    fn get_place_history(&self, id: &str, revision: Option<Revision>) -> Result<PlaceHistory> {
        use schema::place::dsl;
        use schema::place_revision::dsl as rev_dsl;

        let mut query = schema::place_revision::table
            .inner_join(schema::place::table.on(rev_dsl::parent_rowid.eq(dsl::rowid)))
            .select((
                rev_dsl::rowid,
                rev_dsl::rev,
                rev_dsl::created_at,
                rev_dsl::created_by,
                rev_dsl::current_status,
                rev_dsl::title,
                rev_dsl::description,
                rev_dsl::lat,
                rev_dsl::lon,
                rev_dsl::street,
                rev_dsl::zip,
                rev_dsl::city,
                rev_dsl::country,
                rev_dsl::state,
                rev_dsl::contact_name,
                rev_dsl::email,
                rev_dsl::phone,
                rev_dsl::homepage,
                rev_dsl::opening_hours,
                rev_dsl::founded_on,
                rev_dsl::image_url,
                rev_dsl::image_link_url,
                dsl::id,
                dsl::license,
            ))
            .filter(dsl::id.eq(id))
            .order_by(rev_dsl::rev.desc())
            .into_boxed();
        if let Some(revision) = revision {
            query = query.filter(rev_dsl::rev.eq(RevisionValue::from(revision) as i64));
        }
        let rows = query.load::<models::JoinedPlaceRevision>(self)?;
        let mut place_history = None;
        let num_revisions = rows.len();
        for row in rows {
            let parent_rowid = row.id;
            let (place, _) = load_place(self, row)?;
            let (place, place_revision) = place.into();
            if place_history.is_none() {
                place_history = Some(PlaceHistory {
                    place,
                    revisions: Vec::with_capacity(num_revisions),
                });
            };
            use schema::place_revision_review::dsl as review_dsl;
            use schema::users::dsl as user_dsl;
            let rows = schema::place_revision_review::table
                .left_outer_join(
                    schema::users::table.on(review_dsl::created_by.eq(user_dsl::id.nullable())),
                )
                .select((
                    review_dsl::rev,
                    review_dsl::created_at,
                    review_dsl::created_by,
                    user_dsl::email.nullable(),
                    review_dsl::status,
                    review_dsl::context,
                    review_dsl::comment,
                ))
                .filter(review_dsl::parent_rowid.eq(parent_rowid))
                .order_by(review_dsl::rev.desc())
                .load::<models::PlaceReviewedRevision>(self)?;
            let mut review_logs = Vec::with_capacity(rows.len());
            for row in rows {
                let review_log = ReviewStatusLog {
                    revision: Revision::from(row.rev as u64),
                    activity: ActivityLog {
                        activity: Activity {
                            at: TimestampMs::from_inner(row.created_at),
                            by: row.created_by_email.map(Into::into),
                        },
                        context: row.context,
                        comment: row.comment,
                    },
                    status: ReviewStatus::try_from(row.status).unwrap(),
                };
                review_logs.push(review_log);
            }
            place_history
                .as_mut()
                .unwrap()
                .revisions
                .push((place_revision, review_logs));
        }
        place_history.ok_or(RepoError::NotFound)
    }

    fn load_place_revision(&self, id: &str, rev: Revision) -> Result<(Place, ReviewStatus)> {
        use schema::place::dsl;
        use schema::place_revision::dsl as rev_dsl;

        let query = schema::place_revision::table
            .inner_join(
                schema::place::table.on(rev_dsl::parent_rowid
                    .eq(dsl::rowid)
                    .and(rev_dsl::rev.eq(RevisionValue::from(rev) as i64))),
            )
            .select((
                rev_dsl::rowid,
                rev_dsl::rev,
                rev_dsl::created_at,
                rev_dsl::created_by,
                rev_dsl::current_status,
                rev_dsl::title,
                rev_dsl::description,
                rev_dsl::lat,
                rev_dsl::lon,
                rev_dsl::street,
                rev_dsl::zip,
                rev_dsl::city,
                rev_dsl::country,
                rev_dsl::state,
                rev_dsl::contact_name,
                rev_dsl::email,
                rev_dsl::phone,
                rev_dsl::homepage,
                rev_dsl::opening_hours,
                rev_dsl::founded_on,
                rev_dsl::image_url,
                rev_dsl::image_link_url,
                dsl::id,
                dsl::license,
            ))
            .filter(dsl::id.eq(id));
        let row = query.first::<models::JoinedPlaceRevision>(self)?;
        load_place(self, row)
    }
}

fn into_new_event_with_tags(
    conn: &SqliteConnection,
    event: Event,
) -> Result<(models::NewEvent, Vec<String>)> {
    let Event {
        id,
        title,
        start,
        end,
        description,
        location,
        contact,
        homepage,
        created_by,
        registration,
        archived,
        image_url,
        image_link_url,
        tags,
        ..
    } = event;

    let (lat, lng, address) = if let Some(l) = location {
        let Location { pos, address } = l;
        // The position might be invalid if no geo coords have
        // been provided. Nevertheless some address fields might
        // have been provided.
        if pos.is_valid() {
            (
                Some(pos.lat().to_deg()),
                Some(pos.lng().to_deg()),
                address.unwrap_or_default(),
            )
        } else {
            (None, None, address.unwrap_or_default())
        }
    } else {
        (None, None, Default::default())
    };
    let Address {
        street,
        zip,
        city,
        country,
        state,
    } = address;

    let (organizer, email, telephone) = if let Some(c) = contact {
        (c.name, c.email, c.phone)
    } else {
        (None, None, None)
    };

    let registration = registration.map(util::registration_type_into_i16);

    let created_by = if let Some(ref email) = created_by {
        Some(resolve_user_created_by_email(conn, email)?)
    } else {
        None
    };

    Ok((
        models::NewEvent {
            uid: id.into(),
            title,
            description,
            start: start.timestamp(),
            end: end.map(|x| x.timestamp()),
            lat,
            lng,
            street,
            zip,
            city,
            country,
            state,
            telephone,
            email: email.map(Into::into),
            homepage: homepage.map(Into::into),
            created_by,
            registration,
            organizer,
            archived: archived.map(Timestamp::into_inner),
            image_url: image_url.map(Into::into),
            image_link_url: image_link_url.map(Into::into),
        },
        tags,
    ))
}

fn resolve_event_id(conn: &SqliteConnection, uid: &str) -> Result<i64> {
    use schema::events::dsl;
    Ok(dsl::events
        .select(dsl::id)
        .filter(dsl::uid.eq(uid))
        .first(conn)?)
}

impl EventGateway for SqliteConnection {
    fn create_event(&self, e: Event) -> Result<()> {
        let (new_event, tags) = into_new_event_with_tags(self, e)?;
        self.transaction::<_, diesel::result::Error, _>(|| {
            // Insert event
            diesel::insert_into(schema::events::table)
                .values(&new_event)
                .execute(self)?;
            let id = resolve_event_id(self, new_event.uid.as_ref()).map_err(|err| {
                warn!(
                    "Failed to resolve id of newly created event {}: {}",
                    new_event.uid, err,
                );
                diesel::result::Error::RollbackTransaction
            })?;
            // Insert event tags
            let tags: Vec<_> = tags
                .iter()
                .map(|tag| models::NewEventTag {
                    event_id: id,
                    tag: &tag,
                })
                .collect();
            diesel::insert_or_ignore_into(schema::event_tags::table)
                .values(&tags)
                .execute(self)?;
            Ok(())
        })?;
        Ok(())
    }

    fn update_event(&self, event: &Event) -> Result<()> {
        let id = resolve_event_id(self, event.id.as_ref())?;
        let (new_event, new_tags) = into_new_event_with_tags(self, event.clone())?;
        self.transaction::<_, diesel::result::Error, _>(|| {
            use schema::event_tags::dsl as et_dsl;
            use schema::events::dsl as e_dsl;
            // Update event
            diesel::update(e_dsl::events.filter(e_dsl::id.eq(&id)))
                .set(&new_event)
                .execute(self)?;
            // Update event tags
            let tags_diff = {
                let old_tags = et_dsl::event_tags
                    .select(et_dsl::tag)
                    .filter(et_dsl::event_id.eq(id))
                    .load(self)?;
                super::util::tags_diff(&old_tags, &new_tags)
            };
            diesel::delete(
                et_dsl::event_tags
                    .filter(et_dsl::event_id.eq(id))
                    .filter(et_dsl::tag.eq_any(&tags_diff.deleted)),
            )
            .execute(self)?;
            {
                let new_tags: Vec<_> = tags_diff
                    .added
                    .iter()
                    .map(|tag| models::NewEventTag {
                        event_id: id,
                        tag: &tag,
                    })
                    .collect();
                diesel::insert_or_ignore_into(et_dsl::event_tags)
                    .values(&new_tags)
                    .execute(self)?;
            }
            Ok(())
        })?;
        Ok(())
    }

    fn get_events_chronologically(&self, ids: &[&str]) -> Result<Vec<Event>> {
        use schema::{event_tags::dsl as et_dsl, events::dsl as e_dsl, users::dsl as u_dsl};

        let rows = e_dsl::events
            .left_outer_join(u_dsl::users)
            .select((
                e_dsl::id,
                e_dsl::uid,
                e_dsl::title,
                e_dsl::description,
                e_dsl::start,
                e_dsl::end,
                e_dsl::lat,
                e_dsl::lng,
                e_dsl::street,
                e_dsl::zip,
                e_dsl::city,
                e_dsl::country,
                e_dsl::state,
                e_dsl::email,
                e_dsl::telephone,
                e_dsl::homepage,
                e_dsl::created_by,
                e_dsl::registration,
                e_dsl::organizer,
                e_dsl::archived,
                e_dsl::image_url,
                e_dsl::image_link_url,
                u_dsl::email.nullable(),
            ))
            .filter(e_dsl::uid.eq_any(ids))
            .filter(e_dsl::archived.is_null())
            .order_by(e_dsl::start)
            .load::<models::EventEntity>(self)?;
        debug_assert!(rows.len() <= ids.len());
        let mut events = Vec::with_capacity(rows.len());
        for row in rows.into_iter() {
            let models::EventEntity {
                id,
                uid,
                title,
                description,
                start,
                end,
                lat,
                lng,
                street,
                zip,
                city,
                country,
                state,
                email,
                telephone,
                homepage,
                registration,
                organizer,
                archived,
                image_url,
                image_link_url,
                created_by_email,
                ..
            } = row;

            let tags = et_dsl::event_tags
                .select(et_dsl::tag)
                .filter(et_dsl::event_id.eq(id))
                .load::<String>(self)?;

            let address = Address {
                street,
                zip,
                city,
                country,
                state,
            };

            let address = if address.is_empty() {
                None
            } else {
                Some(address)
            };

            let pos = if let (Some(lat), Some(lng)) = (lat, lng) {
                MapPoint::try_from_lat_lng_deg(lat, lng)
                    .map(Some)
                    .unwrap_or_default()
            } else {
                None
            };
            let location = if pos.is_some() || address.is_some() {
                Some(Location {
                    pos: pos.unwrap_or_default(),
                    address,
                })
            } else {
                None
            };
            let contact = if organizer.is_some() || email.is_some() || telephone.is_some() {
                Some(Contact {
                    name: organizer,
                    email: email.map(Into::into),
                    phone: telephone,
                })
            } else {
                None
            };

            let registration = registration.map(util::registration_type_from_i16);

            let event = Event {
                id: uid.into(),
                title,
                start: NaiveDateTime::from_timestamp(start, 0),
                end: end.map(|x| NaiveDateTime::from_timestamp(x, 0)),
                description,
                location,
                contact,
                homepage: homepage.and_then(load_url),
                tags,
                created_by: created_by_email,
                registration,
                archived: archived.map(Timestamp::from_inner),
                image_url: image_url.and_then(load_url),
                image_link_url: image_link_url.and_then(load_url),
            };
            events.push(event);
        }

        Ok(events)
    }

    fn get_event(&self, id: &str) -> Result<Event> {
        let events = self.get_events_chronologically(&[id])?;
        debug_assert!(events.len() <= 1);
        events.into_iter().next().ok_or(RepoError::NotFound)
    }

    fn all_events_chronologically(&self) -> Result<Vec<Event>> {
        use schema::{event_tags::dsl as et_dsl, events::dsl as e_dsl, users::dsl as u_dsl};
        let events: Vec<_> = e_dsl::events
            .left_outer_join(u_dsl::users)
            .select((
                e_dsl::id,
                e_dsl::uid,
                e_dsl::title,
                e_dsl::description,
                e_dsl::start,
                e_dsl::end,
                e_dsl::lat,
                e_dsl::lng,
                e_dsl::street,
                e_dsl::zip,
                e_dsl::city,
                e_dsl::country,
                e_dsl::state,
                e_dsl::email,
                e_dsl::telephone,
                e_dsl::homepage,
                e_dsl::created_by,
                e_dsl::registration,
                e_dsl::organizer,
                e_dsl::archived,
                e_dsl::image_url,
                e_dsl::image_link_url,
                u_dsl::email.nullable(),
            ))
            .filter(e_dsl::archived.is_null())
            .order_by(e_dsl::start)
            .load::<models::EventEntity>(self)?;
        let tag_rels = et_dsl::event_tags.load(self)?;
        Ok(events
            .into_iter()
            .map(|e| util::event_from_event_entity_and_tags(e, &tag_rels))
            .collect())
    }

    fn count_events(&self) -> Result<usize> {
        use schema::events::dsl;
        Ok(dsl::events
            .select(diesel::dsl::count(dsl::id))
            .filter(dsl::archived.is_null())
            .first::<i64>(self)? as usize)
    }

    fn archive_events(&self, ids: &[&str], archived: Timestamp) -> Result<usize> {
        use schema::events::dsl;
        let count = diesel::update(
            dsl::events
                .filter(dsl::uid.eq_any(ids))
                .filter(dsl::archived.is_null()),
        )
        .set(dsl::archived.eq(Some(archived.into_inner())))
        .execute(self)?;
        debug_assert!(count <= ids.len());
        Ok(count)
    }

    fn delete_event_with_matching_tags(&self, id: &str, tags: &[&str]) -> Result<bool> {
        use schema::{event_tags::dsl as et_dsl, events::dsl as e_dsl};
        let id = resolve_event_id(self, id)?;
        if !tags.is_empty() {
            let ids: Vec<_> = et_dsl::event_tags
                .select(et_dsl::event_id)
                .distinct()
                .filter(et_dsl::event_id.eq(id))
                .filter(et_dsl::tag.eq_any(tags))
                .load::<i64>(self)?;
            debug_assert!(ids.len() <= 1);
            if ids.is_empty() {
                return Ok(false);
            }
            debug_assert_eq!(id, *ids.first().unwrap());
        }
        diesel::delete(et_dsl::event_tags.filter(et_dsl::event_id.eq(id))).execute(self)?;
        diesel::delete(e_dsl::events.filter(e_dsl::id.eq(id))).execute(self)?;
        Ok(true)
    }

    fn is_event_owned_by_any_organization(&self, id: &str) -> Result<bool> {
        use schema::{event_tags, events, organization_tag};
        Ok(events::table
            .select(events::id)
            .filter(events::uid.eq(id))
            .filter(
                events::id.eq_any(
                    event_tags::table.select(event_tags::event_id).filter(
                        event_tags::tag
                            .eq_any(organization_tag::table.select(organization_tag::tag_label)),
                    ),
                ),
            )
            .first::<i64>(self)
            .optional()?
            .is_some())
    }
}

fn resolve_user_created_by_email(conn: &SqliteConnection, email: &str) -> Result<i64> {
    use schema::users::dsl;
    Ok(dsl::users
        .select(dsl::id)
        .filter(dsl::email.eq(email))
        .first(conn)
        .map_err(|e| {
            log::warn!("Failed to resolve user by e-mail '{}': {}", email, e);
            e
        })?)
}

impl UserGateway for SqliteConnection {
    fn create_user(&self, u: &User) -> Result<()> {
        let new_user = models::NewUser::from(u);
        diesel::insert_into(schema::users::table)
            .values(&new_user)
            .execute(self)?;
        Ok(())
    }

    fn update_user(&self, u: &User) -> Result<()> {
        use schema::users::dsl;
        let new_user = models::NewUser::from(u);
        diesel::update(dsl::users.filter(dsl::email.eq(new_user.email)))
            .set(&new_user)
            .execute(self)?;
        Ok(())
    }

    fn delete_user_by_email(&self, email: &str) -> Result<()> {
        use schema::users::dsl;
        diesel::delete(dsl::users.filter(dsl::email.eq(email))).execute(self)?;
        Ok(())
    }

    fn get_user_by_email(&self, email: &str) -> Result<User> {
        use schema::users::dsl;
        Ok(dsl::users
            .filter(dsl::email.eq(email))
            .first::<models::UserEntity>(self)?
            .into())
    }

    fn try_get_user_by_email(&self, email: &str) -> Result<Option<User>> {
        use schema::users::dsl;
        Ok(dsl::users
            .filter(dsl::email.eq(email))
            .first::<models::UserEntity>(self)
            .optional()?
            .map(Into::into))
    }

    fn all_users(&self) -> Result<Vec<User>> {
        use schema::users::dsl;
        Ok(dsl::users
            .load::<models::UserEntity>(self)?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    fn count_users(&self) -> Result<usize> {
        use schema::users::dsl;
        Ok(dsl::users
            .select(diesel::dsl::count(dsl::id))
            .first::<i64>(self)? as usize)
    }
}

impl RatingRepository for SqliteConnection {
    fn create_rating(&self, rating: Rating) -> Result<()> {
        let Rating {
            id,
            place_id,
            created_at,
            archived_at,
            title,
            value,
            context,
            source,
        } = rating;
        let parent_rowid = resolve_place_rowid(self, &place_id)?;
        let new_place_rating = models::NewPlaceRating {
            id: id.into(),
            parent_rowid,
            created_at: created_at.into_inner(),
            created_by: None,
            archived_at: archived_at.map(Timestamp::into_inner),
            archived_by: None,
            title,
            value: i8::from(value).into(),
            context: util::rating_context_to_string(context),
            source,
        };
        let _count = diesel::insert_into(schema::place_rating::table)
            .values(&new_place_rating)
            .execute(self)?;
        debug_assert_eq!(1, _count);
        Ok(())
    }

    fn load_ratings(&self, ids: &[&str]) -> Result<Vec<Rating>> {
        use schema::place::dsl;
        use schema::place_rating::dsl as rating_dsl;
        Ok(schema::place_rating::table
            .inner_join(schema::place::table)
            .select((
                rating_dsl::rowid,
                rating_dsl::created_at,
                rating_dsl::created_by,
                rating_dsl::archived_at,
                rating_dsl::archived_by,
                rating_dsl::id,
                rating_dsl::title,
                rating_dsl::value,
                rating_dsl::context,
                rating_dsl::source,
                dsl::id,
            ))
            .filter(rating_dsl::id.eq_any(ids))
            .filter(rating_dsl::archived_at.is_null())
            .load::<models::PlaceRating>(self)?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    fn load_rating(&self, id: &str) -> Result<Rating> {
        let ratings = self.load_ratings(&[id])?;
        debug_assert!(ratings.len() <= 1);
        ratings.into_iter().next().ok_or(RepoError::NotFound)
    }

    fn load_ratings_of_place(&self, place_id: &str) -> Result<Vec<Rating>> {
        use schema::place::dsl;
        use schema::place_rating::dsl as rating_dsl;
        Ok(schema::place_rating::table
            .inner_join(schema::place::table)
            .select((
                rating_dsl::rowid,
                rating_dsl::created_at,
                rating_dsl::created_by,
                rating_dsl::archived_at,
                rating_dsl::archived_by,
                rating_dsl::id,
                rating_dsl::title,
                rating_dsl::value,
                rating_dsl::context,
                rating_dsl::source,
                dsl::id,
            ))
            .filter(dsl::id.eq(place_id))
            .filter(rating_dsl::archived_at.is_null())
            .load::<models::PlaceRating>(self)?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    fn load_place_ids_of_ratings(&self, ids: &[&str]) -> Result<Vec<String>> {
        use schema::place::dsl;
        use schema::place_rating::dsl as rating_dsl;
        Ok(schema::place_rating::table
            .inner_join(schema::place::table)
            .select(dsl::id)
            .filter(rating_dsl::id.eq_any(ids))
            .load::<String>(self)?)
    }

    fn archive_ratings(&self, ids: &[&str], activity: &Activity) -> Result<usize> {
        use schema::place_rating::dsl;
        let archived_at = Some(activity.at.into_inner());
        let archived_by = if let Some(ref email) = activity.by {
            Some(resolve_user_created_by_email(self, email.as_ref())?)
        } else {
            None
        };
        let count = diesel::update(
            schema::place_rating::table
                .filter(dsl::id.eq_any(ids))
                .filter(dsl::archived_at.is_null()),
        )
        .set((
            dsl::archived_at.eq(archived_at),
            dsl::archived_by.eq(archived_by),
        ))
        .execute(self)?;
        debug_assert!(count <= ids.len());
        Ok(count)
    }

    fn archive_ratings_of_places(&self, place_ids: &[&str], activity: &Activity) -> Result<usize> {
        use schema::place::dsl;
        use schema::place_rating::dsl as rating_dsl;
        let archived_at = Some(activity.at.into_inner());
        let archived_by = if let Some(ref email) = activity.by {
            Some(resolve_user_created_by_email(self, email.as_ref())?)
        } else {
            None
        };
        Ok(diesel::update(
            schema::place_rating::table
                .filter(
                    rating_dsl::parent_rowid.eq_any(
                        schema::place::table
                            .select(dsl::rowid)
                            .filter(dsl::id.eq_any(place_ids)),
                    ),
                )
                .filter(rating_dsl::archived_at.is_null()),
        )
        .set((
            rating_dsl::archived_at.eq(archived_at),
            rating_dsl::archived_by.eq(archived_by),
        ))
        .execute(self)?)
    }
}

impl CommentRepository for SqliteConnection {
    fn create_comment(&self, comment: Comment) -> Result<()> {
        let Comment {
            id,
            rating_id,
            created_at,
            archived_at,
            text,
            ..
        } = comment;
        let parent_rowid = resolve_rating_rowid(self, rating_id.as_ref())?;
        let new_place_rating_comment = models::NewPlaceRatingComment {
            id: id.into(),
            parent_rowid,
            created_at: created_at.into_inner(),
            created_by: None,
            archived_at: archived_at.map(Timestamp::into_inner),
            archived_by: None,
            text,
        };
        let _count = diesel::insert_into(schema::place_rating_comment::table)
            .values(&new_place_rating_comment)
            .execute(self)?;
        debug_assert_eq!(1, _count);
        Ok(())
    }

    fn load_comments(&self, ids: &[&str]) -> Result<Vec<Comment>> {
        use schema::place_rating::dsl as rating_dsl;
        use schema::place_rating_comment::dsl as comment_dsl;
        // TODO: Split loading into chunks of fixed size
        info!("Loading multiple ({}) comments at once", ids.len());
        Ok(schema::place_rating_comment::table
            .inner_join(schema::place_rating::table)
            .select((
                comment_dsl::rowid,
                comment_dsl::created_at,
                comment_dsl::created_by,
                comment_dsl::archived_at,
                comment_dsl::archived_by,
                comment_dsl::id,
                comment_dsl::text,
                rating_dsl::id,
            ))
            .filter(comment_dsl::id.eq_any(ids))
            .filter(comment_dsl::archived_at.is_null())
            .load::<models::PlaceRatingComment>(self)?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    fn load_comment(&self, id: &str) -> Result<Comment> {
        let comments = self.load_comments(&[id])?;
        debug_assert!(comments.len() <= 1);
        comments.into_iter().next().ok_or(RepoError::NotFound)
    }

    fn load_comments_of_rating(&self, rating_id: &str) -> Result<Vec<Comment>> {
        use schema::place_rating::dsl as rating_dsl;
        use schema::place_rating_comment::dsl as comment_dsl;
        Ok(schema::place_rating_comment::table
            .inner_join(schema::place_rating::table)
            .select((
                comment_dsl::rowid,
                comment_dsl::created_at,
                comment_dsl::created_by,
                comment_dsl::archived_at,
                comment_dsl::archived_by,
                comment_dsl::id,
                comment_dsl::text,
                rating_dsl::id,
            ))
            .filter(rating_dsl::id.eq(rating_id))
            .filter(comment_dsl::archived_at.is_null())
            .load::<models::PlaceRatingComment>(self)?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    fn archive_comments(&self, ids: &[&str], activity: &Activity) -> Result<usize> {
        use schema::place_rating_comment::dsl;
        let archived_at = Some(activity.at.into_inner());
        let archived_by = if let Some(ref email) = activity.by {
            Some(resolve_user_created_by_email(self, email.as_ref())?)
        } else {
            None
        };
        let count = diesel::update(
            schema::place_rating_comment::table
                .filter(dsl::id.eq_any(ids))
                .filter(dsl::archived_at.is_null()),
        )
        .set((
            dsl::archived_at.eq(archived_at),
            dsl::archived_by.eq(archived_by),
        ))
        .execute(self)?;
        debug_assert!(count <= ids.len());
        Ok(count)
    }

    fn archive_comments_of_ratings(
        &self,
        rating_ids: &[&str],
        activity: &Activity,
    ) -> Result<usize> {
        use schema::place_rating::dsl as rating_dsl;
        use schema::place_rating_comment::dsl as comment_dsl;
        let archived_at = Some(activity.at.into_inner());
        let archived_by = if let Some(ref email) = activity.by {
            Some(resolve_user_created_by_email(self, email.as_ref())?)
        } else {
            None
        };
        Ok(diesel::update(
            schema::place_rating_comment::table
                .filter(
                    comment_dsl::parent_rowid.eq_any(
                        schema::place_rating::table
                            .select(rating_dsl::rowid)
                            .filter(rating_dsl::id.eq_any(rating_ids)),
                    ),
                )
                .filter(comment_dsl::archived_at.is_null()),
        )
        .set((
            comment_dsl::archived_at.eq(archived_at),
            comment_dsl::archived_by.eq(archived_by),
        ))
        .execute(self)?)
    }

    fn archive_comments_of_places(&self, place_ids: &[&str], activity: &Activity) -> Result<usize> {
        use schema::place::dsl;
        use schema::place_rating::dsl as rating_dsl;
        use schema::place_rating_comment::dsl as comment_dsl;
        let archived_at = Some(activity.at.into_inner());
        let archived_by = if let Some(ref email) = activity.by {
            Some(resolve_user_created_by_email(self, email.as_ref())?)
        } else {
            None
        };
        Ok(diesel::update(
            schema::place_rating_comment::table
                .filter(
                    comment_dsl::parent_rowid.eq_any(
                        schema::place_rating::table
                            .select(rating_dsl::rowid)
                            .filter(
                                rating_dsl::parent_rowid.eq_any(
                                    schema::place::table
                                        .select(dsl::rowid)
                                        .filter(dsl::id.eq_any(place_ids)),
                                ),
                            ),
                    ),
                )
                .filter(comment_dsl::archived_at.is_null()),
        )
        .set((
            comment_dsl::archived_at.eq(archived_at),
            comment_dsl::archived_by.eq(archived_by),
        ))
        .execute(self)
        .optional()?
        .unwrap_or_default())
    }
}

impl Db for SqliteConnection {
    fn create_tag_if_it_does_not_exist(&self, t: &Tag) -> Result<()> {
        let res = diesel::insert_into(schema::tags::table)
            .values(&models::Tag::from(t.clone()))
            .execute(self);
        if let Err(err) = res {
            match err {
                DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                    // that's ok :)
                }
                _ => {
                    return Err(err.into());
                }
            }
        }
        Ok(())
    }

    fn create_bbox_subscription(&self, new: &BboxSubscription) -> Result<()> {
        let user_id = resolve_user_created_by_email(self, &new.user_email)?;
        let (south_west_lat, south_west_lng) = new.bbox.southwest().to_lat_lng_deg();
        let (north_east_lat, north_east_lng) = new.bbox.northeast().to_lat_lng_deg();
        let insertable = models::NewBboxSubscription {
            uid: new.id.as_ref(),
            user_id,
            south_west_lat,
            south_west_lng,
            north_east_lat,
            north_east_lng,
        };
        diesel::insert_into(schema::bbox_subscriptions::table)
            .values(&insertable)
            .execute(self)?;
        Ok(())
    }

    fn all_bbox_subscriptions(&self) -> Result<Vec<BboxSubscription>> {
        use schema::bbox_subscriptions::dsl as s_dsl;
        use schema::users::dsl as u_dsl;
        Ok(s_dsl::bbox_subscriptions
            .inner_join(u_dsl::users)
            .select((
                s_dsl::id,
                s_dsl::uid,
                s_dsl::user_id,
                s_dsl::south_west_lat,
                s_dsl::south_west_lng,
                s_dsl::north_east_lat,
                s_dsl::north_east_lng,
                u_dsl::email,
            ))
            .load::<models::BboxSubscriptionEntity>(self)?
            .into_iter()
            .map(BboxSubscription::from)
            .collect())
    }
    fn all_bbox_subscriptions_by_email(&self, email: &str) -> Result<Vec<BboxSubscription>> {
        use schema::bbox_subscriptions::dsl as s_dsl;
        use schema::users::dsl as u_dsl;
        Ok(s_dsl::bbox_subscriptions
            .inner_join(u_dsl::users)
            .filter(u_dsl::email.eq(email))
            .select((
                s_dsl::id,
                s_dsl::uid,
                s_dsl::user_id,
                s_dsl::south_west_lat,
                s_dsl::south_west_lng,
                s_dsl::north_east_lat,
                s_dsl::north_east_lng,
                u_dsl::email,
            ))
            .load::<models::BboxSubscriptionEntity>(self)?
            .into_iter()
            .map(BboxSubscription::from)
            .collect())
    }
    fn delete_bbox_subscriptions_by_email(&self, email: &str) -> Result<()> {
        use schema::bbox_subscriptions::dsl as s_dsl;
        use schema::users::dsl as u_dsl;
        let users_id = u_dsl::users
            .select(u_dsl::id)
            .filter(u_dsl::email.eq(email));
        diesel::delete(s_dsl::bbox_subscriptions.filter(s_dsl::user_id.eq_any(users_id)))
            .execute(self)?;
        Ok(())
    }
    fn all_tags(&self) -> Result<Vec<Tag>> {
        use schema::tags::dsl::*;
        Ok(tags
            .load::<models::Tag>(self)?
            .into_iter()
            .map(Tag::from)
            .collect())
    }
    fn count_tags(&self) -> Result<usize> {
        use schema::tags::dsl::*;
        Ok(tags.select(diesel::dsl::count(id)).first::<i64>(self)? as usize)
    }
}

impl OrganizationRepo for SqliteConnection {
    fn create_org(&mut self, mut o: Organization) -> Result<()> {
        let org_id = o.id.clone();
        let moderated_tags = std::mem::take(&mut o.moderated_tags);
        let new_org = models::NewOrganization::from(o);
        self.transaction::<_, diesel::result::Error, _>(|| {
            diesel::insert_into(schema::organization::table)
                .values(&new_org)
                .execute(self)?;
            let org_rowid = resolve_organization_rowid(self, &org_id).map_err(|err| {
                warn!(
                    "Failed to resolve id of newly created organization '{}': {}",
                    org_id, err
                );
                diesel::result::Error::RollbackTransaction
            })?;
            for ModeratedTag {
                label,
                allow_add,
                allow_remove,
                require_clearance,
            } in &moderated_tags
            {
                let org_tag = models::NewOrganizationTag {
                    org_rowid,
                    tag_label: label,
                    tag_allow_add: if *allow_add { 1 } else { 0 },
                    tag_allow_remove: if *allow_remove { 1 } else { 0 },
                    require_clearance: if *require_clearance { 1 } else { 0 },
                };
                diesel::insert_into(schema::organization_tag::table)
                    .values(&org_tag)
                    .execute(self)?;
            }
            Ok(())
        })?;
        Ok(())
    }

    fn get_org_by_api_token(&self, token: &str) -> Result<Organization> {
        use schema::{organization::dsl as org_dsl, organization_tag::dsl as org_tag_dsl};

        let models::Organization {
            rowid,
            id,
            name,
            api_token,
        } = org_dsl::organization
            .filter(org_dsl::api_token.eq(token))
            .first(self)?;

        let moderated_tags = org_tag_dsl::organization_tag
            .filter(org_tag_dsl::org_rowid.eq(rowid))
            .load::<models::OrganizationTag>(self)?
            .into_iter()
            .map(Into::into)
            .collect();

        Ok(Organization {
            id: id.into(),
            name,
            api_token,
            moderated_tags,
        })
    }

    fn map_tag_to_clearance_org_id(&self, tag: &str) -> Result<Option<Id>> {
        use schema::{organization::dsl, organization_tag::dsl as tag_dsl};
        Ok(schema::organization::table
            .inner_join(schema::organization_tag::table)
            .select(dsl::id)
            .filter(tag_dsl::tag_label.eq(tag))
            .filter(tag_dsl::require_clearance.ne(0))
            .first::<String>(self)
            .optional()?
            .map(Into::into))
    }

    fn get_moderated_tags_by_org(
        &self,
        excluded_org_id: Option<&Id>,
    ) -> Result<Vec<(Id, ModeratedTag)>> {
        use schema::organization::dsl as org_dsl;
        use schema::organization_tag::dsl as org_tag_dsl;
        let query = org_tag_dsl::organization_tag
            .inner_join(org_dsl::organization)
            .select((
                org_dsl::id,
                org_tag_dsl::tag_label,
                org_tag_dsl::tag_allow_add,
                org_tag_dsl::tag_allow_remove,
                org_tag_dsl::require_clearance,
            ))
            .order_by(org_dsl::id);
        let moderated_tags = if let Some(excluded_org_id) = excluded_org_id {
            query
                .filter(org_dsl::id.ne(excluded_org_id.as_str()))
                .load::<models::OrganizationTagWithId>(self)?
        } else {
            query.load::<models::OrganizationTagWithId>(self)?
        };
        Ok(moderated_tags.into_iter().map(Into::into).collect())
    }
}

impl PlaceClearanceRepo for SqliteConnection {
    fn add_pending_clearance_for_places(
        &self,
        org_ids: &[Id],
        pending_clearance: &PendingClearanceForPlace,
    ) -> Result<usize> {
        let PendingClearanceForPlace {
            place_id,
            created_at,
            last_cleared_revision,
        } = pending_clearance;
        let place_rowid = resolve_place_rowid(self, place_id)?;
        let created_at = created_at.into_inner();
        let last_cleared_revision =
            last_cleared_revision.map(|rev| RevisionValue::from(rev) as i64);
        let mut insert_count = 0;
        for org_id in org_ids {
            let org_rowid = resolve_organization_rowid(self, org_id)?;
            let insertable = models::NewPendingClearanceForPlace {
                org_rowid,
                place_rowid,
                created_at,
                last_cleared_revision,
            };
            insert_count +=
                diesel::insert_or_ignore_into(schema::organization_place_clearance::table)
                    .values(&insertable)
                    .execute(self)?;
        }
        Ok(insert_count)
    }

    fn count_pending_clearances_for_places(&self, org_id: &Id) -> Result<u64> {
        use schema::organization::dsl as org_dsl;
        use schema::organization_place_clearance::dsl;
        Ok(schema::organization_place_clearance::table
            .filter(
                dsl::org_rowid.eq_any(
                    schema::organization::table
                        .select(org_dsl::rowid)
                        .filter(org_dsl::id.eq(org_id.as_str())),
                ),
            )
            .count()
            .get_result::<i64>(self)? as u64)
    }

    fn list_pending_clearances_for_places(
        &self,
        org_id: &Id,
        pagination: &Pagination,
    ) -> Result<Vec<PendingClearanceForPlace>> {
        use schema::organization::dsl as org_dsl;
        use schema::organization_place_clearance::dsl;
        use schema::place::dsl as place_dsl;
        let mut query = schema::organization_place_clearance::table
            .inner_join(schema::place::table)
            .select((place_dsl::id, dsl::created_at, dsl::last_cleared_revision))
            .filter(
                dsl::org_rowid.eq_any(
                    schema::organization::table
                        .select(org_dsl::rowid)
                        .filter(org_dsl::id.eq(org_id.as_str())),
                ),
            )
            .order_by(dsl::created_at)
            .into_boxed();

        // Pagination
        let offset = pagination.offset.unwrap_or(0);
        if offset > 0 {
            query = query.offset(offset as i64);
        }
        if let Some(limit) = pagination.limit {
            query = query.limit(limit as i64);
        }

        Ok(query
            .load::<models::PendingClearanceForPlace>(self)?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    fn load_pending_clearances_for_places(
        &self,
        org_id: &Id,
        place_ids: &[&str],
    ) -> Result<Vec<PendingClearanceForPlace>> {
        use schema::organization::dsl as org_dsl;
        use schema::organization_place_clearance::dsl;
        use schema::place::dsl as place_dsl;
        Ok(schema::organization_place_clearance::table
            .inner_join(schema::place::table)
            .select((place_dsl::id, dsl::created_at, dsl::last_cleared_revision))
            .filter(
                dsl::org_rowid.eq_any(
                    schema::organization::table
                        .select(org_dsl::rowid)
                        .filter(org_dsl::id.eq(org_id.as_str())),
                ),
            )
            .filter(place_dsl::id.eq_any(place_ids))
            .load::<models::PendingClearanceForPlace>(self)?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    fn update_pending_clearances_for_places(
        &self,
        org_id: &Id,
        clearances: &[ClearanceForPlace],
    ) -> Result<usize> {
        let org_rowid = resolve_organization_rowid(self, org_id)?;
        let created_at = TimestampMs::now().into_inner();
        let mut total_rows_affected = 0;
        for clearance in clearances {
            let ClearanceForPlace {
                place_id,
                cleared_revision,
            } = clearance;
            let (place_rowid, cleared_revision) = if let Some(cleared_revision) = cleared_revision {
                let place_rowid =
                    resolve_place_rowid_verify_revision(self, place_id, *cleared_revision)?;
                (place_rowid, *cleared_revision)
            } else {
                let (place_rowid, current_revision) =
                    resolve_place_rowid_with_current_revision(self, place_id)?;
                (place_rowid, current_revision)
            };
            use schema::organization::dsl as org_dsl;
            use schema::organization_place_clearance::dsl;
            let last_cleared_revision = Some(RevisionValue::from(cleared_revision) as i64);
            let updatable = models::NewPendingClearanceForPlace {
                org_rowid,
                place_rowid,
                created_at,
                last_cleared_revision,
            };
            let rows_affected = diesel::update(schema::organization_place_clearance::table)
                .set(&updatable)
                .filter(
                    dsl::org_rowid.eq_any(
                        schema::organization::table
                            .select(org_dsl::rowid)
                            .filter(org_dsl::id.eq(org_id.as_str())),
                    ),
                )
                .filter(dsl::place_rowid.eq(place_rowid))
                .execute(self)?;
            debug_assert!(rows_affected <= 1);
            total_rows_affected += rows_affected;
        }
        Ok(total_rows_affected)
    }

    fn cleanup_pending_clearances_for_places(&self, org_id: &Id) -> Result<u64> {
        let org_rowid = resolve_organization_rowid(self, org_id)?;
        use schema::organization_place_clearance::dsl;
        use schema::place::dsl as place_dsl;
        use schema::place_revision::dsl as place_rev_dsl;
        let current_place_revisions = schema::place::table
            .inner_join(schema::place_revision::table)
            .filter(place_dsl::current_rev.eq(place_rev_dsl::rev));
        let subselect = current_place_revisions
            .inner_join(
                schema::organization_place_clearance::table
                    .on(dsl::place_rowid.eq(place_dsl::rowid)),
            )
            .select(dsl::rowid)
            .filter(dsl::org_rowid.eq(org_rowid))
            .filter(dsl::last_cleared_revision.eq(place_dsl::current_rev.nullable()));
        // TODO: Diesel 1.4.5 does not allow to use a subselect in the
        // following delete statement and requires to temporarily load
        // the subselect results into memory
        let delete_rowids = subselect.load::<i64>(self)?;
        let delete_count = diesel::delete(
            schema::organization_place_clearance::table.filter(dsl::rowid.eq_any(delete_rowids)),
        )
        .execute(self)?;
        Ok(delete_count as u64)
    }
}

impl UserTokenRepo for SqliteConnection {
    fn replace_user_token(&self, token: UserToken) -> Result<EmailNonce> {
        use schema::user_tokens::dsl;
        let user_id = resolve_user_created_by_email(self, &token.email_nonce.email)?;
        let model = models::NewUserToken {
            user_id,
            nonce: token.email_nonce.nonce.to_string(),
            expires_at: token.expires_at.into_inner(),
        };
        // Insert...
        if diesel::insert_into(schema::user_tokens::table)
            .values(&model)
            .execute(self)?
            == 0
        {
            // ...or update
            let _count = diesel::update(schema::user_tokens::table)
                .filter(dsl::user_id.eq(model.user_id))
                .set(&model)
                .execute(self)?;
            debug_assert_eq!(1, _count);
        }
        Ok(token.email_nonce)
    }

    fn consume_user_token(&self, email_nonce: &EmailNonce) -> Result<UserToken> {
        use schema::user_tokens::dsl as t_dsl;
        use schema::users::dsl as u_dsl;
        let token = self.get_user_token_by_email(&email_nonce.email)?;
        let user_id_subselect = u_dsl::users
            .select(u_dsl::id)
            .filter(u_dsl::email.eq(&email_nonce.email));
        let target = t_dsl::user_tokens
            .filter(t_dsl::nonce.eq(email_nonce.nonce.to_string()))
            .filter(t_dsl::user_id.eq_any(user_id_subselect));
        if diesel::delete(target).execute(self)? == 0 {
            return Err(RepoError::NotFound);
        }
        debug_assert_eq!(email_nonce, &token.email_nonce);
        Ok(token)
    }

    fn delete_expired_user_tokens(&self, expired_before: Timestamp) -> Result<usize> {
        use schema::user_tokens::dsl;
        Ok(
            diesel::delete(
                dsl::user_tokens.filter(dsl::expires_at.lt(expired_before.into_inner())),
            )
            .execute(self)?,
        )
    }

    fn get_user_token_by_email(&self, email: &str) -> Result<UserToken> {
        use schema::user_tokens::dsl as t_dsl;
        use schema::users::dsl as u_dsl;
        Ok(t_dsl::user_tokens
            .inner_join(u_dsl::users)
            .select((u_dsl::id, t_dsl::nonce, t_dsl::expires_at, u_dsl::email))
            .filter(u_dsl::email.eq(email))
            .first::<models::UserTokenEntity>(self)?
            .into())
    }
}
