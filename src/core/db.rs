use super::{
    entities::*,
    error::RepoError,
    repositories::*,
    util::{
        geo::{MapBbox, MapPoint},
        time::{Timestamp, TimestampMs},
    },
};

use anyhow::Result as Fallible;

type Result<T> = std::result::Result<T, RepoError>;

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct MostPopularTagsParams {
    pub min_count: Option<u64>,
    pub max_count: Option<u64>,
    pub like:  Option<String>
}

#[derive(Clone, Debug)]
pub struct RecentlyChangedEntriesParams {
    pub since: Option<TimestampMs>,
    pub until: Option<TimestampMs>,
}

pub trait PlaceRepo {
    fn get_place(&self, id: &str) -> Result<(Place, ReviewStatus)>;
    fn get_places(&self, ids: &[&str]) -> Result<Vec<(Place, ReviewStatus)>>;

    fn all_places(&self) -> Result<Vec<(Place, ReviewStatus)>>;
    fn count_places(&self) -> Result<usize>;

    fn recently_changed_places(
        &self,
        params: &RecentlyChangedEntriesParams,
        pagination: &Pagination,
    ) -> Result<Vec<(Place, ReviewStatus, ActivityLog)>>;

    fn most_popular_place_revision_tags(
        &self,
        params: &MostPopularTagsParams,
        pagination: &Pagination,
    ) -> Result<Vec<TagFrequency>>;

    fn review_places(
        &self,
        ids: &[&str],
        status: ReviewStatus,
        activity: &ActivityLog,
    ) -> Result<usize>;

    fn create_or_update_place(&self, place: Place) -> Result<()>;

    fn get_place_history(&self, id: &str, revision: Option<Revision>) -> Result<PlaceHistory>;

    fn load_place_revision(&self, id: &str, rev: Revision) -> Result<(Place, ReviewStatus)>;
}

pub trait EventGateway {
    fn create_event(&self, _: Event) -> Result<()>;
    fn update_event(&self, _: &Event) -> Result<()>;
    fn archive_events(&self, ids: &[&str], archived: Timestamp) -> Result<usize>;

    fn get_event(&self, id: &str) -> Result<Event>;
    fn get_events_chronologically(&self, ids: &[&str]) -> Result<Vec<Event>>;

    fn all_events_chronologically(&self) -> Result<Vec<Event>>;

    fn count_events(&self) -> Result<usize>;

    // Delete an event, but only if tagged with at least one of the given tags.
    // If no tags are provided the event is deleted unconditionally.
    // Ok(true)  => Found and deleted
    // Ok(false) => Found but no matching tags
    // TODO: Use explicit result semantics
    fn delete_event_with_matching_tags(&self, id: &str, tags: &[&str]) -> Result<bool>;

    fn is_event_owned_by_any_organization(&self, id: &str) -> Result<bool>;
}

pub trait UserGateway {
    fn create_user(&self, user: &User) -> Result<()>;
    fn update_user(&self, user: &User) -> Result<()>;
    fn delete_user_by_email(&self, email: &str) -> Result<()>;

    fn all_users(&self) -> Result<Vec<User>>;
    fn count_users(&self) -> Result<usize>;

    fn get_user_by_email(&self, email: &str) -> Result<User>;
    fn try_get_user_by_email(&self, email: &str) -> Result<Option<User>>;
}

pub trait OrganizationRepo {
    fn create_org(&mut self, _: Organization) -> Result<()>;
    fn get_org_by_api_token(&self, token: &str) -> Result<Organization>;
    fn map_tag_to_clearance_org_id(&self, tag: &str) -> Result<Option<Id>>;
    fn get_moderated_tags_by_org(
        &self,
        excluded_org_id: Option<&Id>,
    ) -> Result<Vec<(Id, ModeratedTag)>>;
}

pub trait PlaceClearanceRepo {
    fn add_pending_clearance_for_places(
        &self,
        org_ids: &[Id],
        pending_clearance: &PendingClearanceForPlace,
    ) -> Result<usize>;
    fn count_pending_clearances_for_places(&self, org_id: &Id) -> Result<u64>;
    fn list_pending_clearances_for_places(
        &self,
        org_id: &Id,
        pagination: &Pagination,
    ) -> Result<Vec<PendingClearanceForPlace>>;
    fn load_pending_clearances_for_places(
        &self,
        org_id: &Id,
        place_ids: &[&str],
    ) -> Result<Vec<PendingClearanceForPlace>>;
    fn update_pending_clearances_for_places(
        &self,
        org_id: &Id,
        clearances: &[ClearanceForPlace],
    ) -> Result<usize>;
    fn cleanup_pending_clearances_for_places(&self, org_id: &Id) -> Result<u64>;
}

//TODO:
//  - TagGeatway
//  - SubscriptionGateway

#[derive(Clone, Debug, Copy, Default, PartialEq, Eq, Hash)]
pub struct Pagination {
    pub offset: Option<u64>,
    pub limit: Option<u64>,
}

pub trait Db:
    PlaceRepo
    + UserGateway
    + EventGateway
    + OrganizationRepo
    + CommentRepository
    + RatingRepository
    + UserTokenRepo
    + PlaceClearanceRepo
{
    fn create_tag_if_it_does_not_exist(&self, _: &Tag) -> Result<()>;

    fn all_categories(&self) -> Result<Vec<Category>> {
        Ok(vec![
            Category::new_non_profit(),
            Category::new_commercial(),
            Category::new_event(),
        ])
    }
    fn all_tags(&self) -> Result<Vec<Tag>>;
    fn count_tags(&self) -> Result<usize>;

    fn create_bbox_subscription(&self, _: &BboxSubscription) -> Result<()>;
    fn all_bbox_subscriptions(&self) -> Result<Vec<BboxSubscription>>;
    fn all_bbox_subscriptions_by_email(&self, user_email: &str) -> Result<Vec<BboxSubscription>>;
    fn delete_bbox_subscriptions_by_email(&self, user_email: &str) -> Result<()>;
}

#[derive(Copy, Clone, Debug)]
pub enum IndexQueryMode {
    WithRating,
    WithoutRating,
}

#[derive(Debug, Default, Clone)]
pub struct IndexQuery<'a, 'b> {
    // status = None: Don't filter by review status, i.e. return all entries
    //          independent of their current review status
    // status = Some(empty vector): Exclude all invisible/inexistent entries, i.e.
    //          return only visible/existent entries
    // status = Some(non-empty vector): Include entries only if their current review
    //          status matches one of the given values
    pub status: Option<Vec<ReviewStatus>>,
    pub include_bbox: Option<MapBbox>,
    pub exclude_bbox: Option<MapBbox>,
    pub categories: Vec<&'a str>,
    pub ids: Vec<&'b str>,
    pub hash_tags: Vec<String>,
    pub text_tags: Vec<String>,
    pub text: Option<String>,
    pub ts_min_lb: Option<Timestamp>, // lower bound (inclusive)
    pub ts_min_ub: Option<Timestamp>, // upper bound (inclusive)
    pub ts_max_lb: Option<Timestamp>, // lower bound (inclusive)
    pub ts_max_ub: Option<Timestamp>, // upper bound (inclusive)
}

pub trait Indexer {
    fn flush_index(&mut self) -> Fallible<()>;
}

pub trait IdIndex {
    fn query_ids(
        &self,
        mode: IndexQueryMode,
        query: &IndexQuery,
        limit: usize,
    ) -> Fallible<Vec<Id>>;
}

pub trait IdIndexer: Indexer + IdIndex {
    fn remove_by_id(&self, id: &Id) -> Fallible<()>;
}

#[derive(Debug, Default, Clone)]
pub struct IndexedPlace {
    pub id: String,
    pub status: Option<ReviewStatus>,
    pub pos: MapPoint,
    pub title: String,
    pub description: String,
    pub tags: Vec<String>,
    pub ratings: AvgRatings,
}

pub trait PlaceIndex {
    fn query_places(&self, query: &IndexQuery, limit: usize) -> Fallible<Vec<IndexedPlace>>;
}

pub trait PlaceIndexer: IdIndexer + PlaceIndex {
    fn add_or_update_place(
        &self,
        place: &Place,
        status: ReviewStatus,
        ratings: &AvgRatings,
    ) -> Fallible<()>;
}

pub trait EventIndexer: IdIndexer {
    fn add_or_update_event(&self, event: &Event) -> Fallible<()>;
}

pub trait EventAndPlaceIndexer: PlaceIndexer + EventIndexer {}
