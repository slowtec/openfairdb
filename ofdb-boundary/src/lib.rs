//#![deny(missing_docs)] // TODO: Complete missing documentation and enable this option
#![cfg_attr(features = "extra-derive", deny(missing_debug_implementations))]
#![deny(broken_intra_doc_links)]
#![cfg_attr(test, deny(warnings))]

//! # ofdb-boundary
//!
//! Serializable, anemic data structures for accessing the OpenFairDB API in a type-safe manner.
//!
//! Only supposed to be used as short-lived, transitional instances for (de-)serializing entities!

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[cfg(feature = "entity-conversions")]
mod conv;

type RevisionValue = u64;
type Url = String;

#[rustfmt::skip]
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "extra-derive", derive(Debug, Clone, PartialEq))]
pub struct Entry {
    pub id             : String,
    pub created        : i64,
    pub version        : u64,
    pub title          : String,
    pub description    : String,
    pub lat            : f64,
    pub lng            : f64,
    pub street         : Option<String>,
    pub zip            : Option<String>,
    pub city           : Option<String>,
    pub country        : Option<String>,
    pub state          : Option<String>,
    pub contact_name   : Option<String>,
    pub email          : Option<String>,
    pub telephone      : Option<String>,
    pub homepage       : Option<Url>,
    pub opening_hours  : Option<String>,
    pub founded_on     : Option<NaiveDate>,
    pub categories     : Vec<String>,
    pub tags           : Vec<String>,
    pub ratings        : Vec<String>,
    pub license        : Option<String>,
    pub image_url      : Option<Url>,
    pub image_link_url : Option<Url>,

    #[serde(rename = "custom", skip_serializing_if = "Vec::is_empty", default = "Default::default")]
    pub custom_links   : Vec<CustomLink>,
}

#[rustfmt::skip]
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "extra-derive", derive(Debug, Clone, PartialEq, Eq))]
pub struct CustomLink {
    pub url            : String,
    pub title          : Option<String>,
    pub description    : Option<String>,
}

#[rustfmt::skip]
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "extra-derive", derive(Debug, Clone, PartialEq))]
pub struct NewPlace {
    pub title          : String,
    pub description    : String,
    pub lat            : f64,
    pub lng            : f64,
    pub street         : Option<String>,
    pub zip            : Option<String>,
    pub city           : Option<String>,
    pub country        : Option<String>,
    pub state          : Option<String>,
    pub contact_name   : Option<String>,
    pub email          : Option<String>,
    pub telephone      : Option<String>,
    pub homepage       : Option<String>,
    pub opening_hours  : Option<String>,
    pub founded_on     : Option<NaiveDate>,
    pub categories     : Vec<String>,
    pub tags           : Vec<String>,
    pub license        : String,
    pub image_url      : Option<String>,
    pub image_link_url : Option<String>,

    #[serde(skip_serializing_if = "Vec::is_empty", default = "Default::default")]
    pub links          : Vec<CustomLink>,
}

#[rustfmt::skip]
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "extra-derive", derive(Debug, Clone, PartialEq))]
pub struct UpdatePlace {
    pub version        : u64,
    pub title          : String,
    pub description    : String,
    pub lat            : f64,
    pub lng            : f64,
    pub street         : Option<String>,
    pub zip            : Option<String>,
    pub city           : Option<String>,
    pub country        : Option<String>,
    pub state          : Option<String>,
    pub contact_name   : Option<String>,
    pub email          : Option<String>,
    pub telephone      : Option<String>,
    pub homepage       : Option<String>,
    pub opening_hours  : Option<String>,
    pub founded_on     : Option<NaiveDate>,
    pub categories     : Vec<String>,
    pub tags           : Vec<String>,
    pub image_url      : Option<String>,
    pub image_link_url : Option<String>,

    #[serde(skip_serializing_if = "Vec::is_empty", default = "Default::default")]
    pub links          : Vec<CustomLink>,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "extra-derive", derive(Debug, Clone))]
pub struct Event {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub start: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lat: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lng: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub street: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zip: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub telephone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registration: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organizer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_link_url: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "extra-derive", derive(Debug, Clone, Copy, PartialEq))]
pub struct Coordinate {
    pub lat: f64,
    pub lng: f64,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "extra-derive", derive(Debug, Clone))]
pub struct User {
    pub email: String,
    pub email_confirmed: bool,
    pub role: UserRole,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "extra-derive", derive(Debug, Clone, Copy))]
pub struct RatingValue(i8);

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "extra-derive", derive(Debug, Clone, Copy))]
pub struct AvgRatingValue(f64);

impl From<f64> for AvgRatingValue {
    fn from(v: f64) -> Self {
        Self(v)
    }
}

impl From<AvgRatingValue> for f64 {
    fn from(from: AvgRatingValue) -> Self {
        from.0
    }
}

impl From<i8> for RatingValue {
    fn from(from: i8) -> Self {
        Self(from)
    }
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(
    feature = "extra-derive",
    derive(Debug, Clone, Copy, PartialEq, Eq, Hash)
)]
#[serde(rename_all = "snake_case")]
pub enum RatingContext {
    Diversity,
    Renewable,
    Fairness,
    Humanity,
    Transparency,
    Solidarity,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "extra-derive", derive(Debug, Clone))]
pub struct EntrySearchRatings {
    pub total: AvgRatingValue,
    pub diversity: AvgRatingValue,
    pub fairness: AvgRatingValue,
    pub humanity: AvgRatingValue,
    pub renewable: AvgRatingValue,
    pub solidarity: AvgRatingValue,
    pub transparency: AvgRatingValue,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "extra-derive", derive(Debug, Clone))]
pub struct Comment {
    pub id: String,
    pub created: i64,
    pub text: String,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "extra-derive", derive(Debug, Clone))]
pub struct Category {
    pub id: String,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "extra-derive", derive(Debug, Clone))]
pub struct PlaceSearchResult {
    pub id: String,
    pub status: Option<ReviewStatus>,
    pub lat: f64,
    pub lng: f64,
    pub title: String,
    pub description: String,
    pub categories: Vec<String>,
    pub tags: Vec<String>,
    pub ratings: EntrySearchRatings,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(
    feature = "extra-derive",
    derive(Debug, Clone, Copy, PartialEq, Eq, Hash)
)]
#[serde(rename_all = "lowercase")]
pub enum ReviewStatus {
    Archived,
    Confirmed,
    Created,
    Rejected,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "extra-derive", derive(Debug, Clone))]
pub struct SearchResponse {
    pub visible: Vec<PlaceSearchResult>,
    pub invisible: Vec<PlaceSearchResult>,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(
    feature = "extra-derive",
    derive(Debug, Clone, Copy, PartialEq, Eq, Hash)
)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Guest,
    User,
    Scout,
    Admin,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "extra-derive", derive(Debug, Clone))]
pub struct BboxSubscription {
    pub id: String,
    pub south_west_lat: f64,
    pub south_west_lng: f64,
    pub north_east_lat: f64,
    pub north_east_lng: f64,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "extra-derive", derive(Debug, Clone))]
pub struct MapBbox {
    pub sw: MapPoint,
    pub ne: MapPoint,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "extra-derive", derive(Debug, Clone))]
pub struct MapPoint {
    pub lat: f64,
    pub lng: f64,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "extra-derive", derive(Debug, Clone))]
pub struct RequestPasswordReset {
    pub email: String,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "extra-derive", derive(Debug, Clone))]
pub struct ResetPassword {
    pub token: String,
    pub new_password: String,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "extra-derive", derive(Debug, Clone))]
pub struct Credentials {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "extra-derive", derive(Debug, Clone))]
pub struct TagFrequency(pub String, pub u64);

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "extra-derive", derive(Debug, Clone))]
pub struct Rating {
    pub id: String,
    pub title: String,
    pub created: i64,
    pub value: RatingValue,
    pub context: RatingContext,
    pub comments: Vec<Comment>,
    pub source: String,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "extra-derive", derive(Debug, Clone, PartialEq, Eq))]
pub struct PendingClearanceForPlace {
    pub place_id: String,
    pub created_at: i64,
    pub last_cleared_revision: Option<RevisionValue>,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "extra-derive", derive(Debug, Clone, PartialEq, Eq))]
pub struct ClearanceForPlace {
    pub place_id: String,
    pub cleared_revision: Option<RevisionValue>,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "extra-derive", derive(Debug, Clone, PartialEq, Eq))]
pub struct ResultCount {
    pub count: u64,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "extra-derive", derive(Debug, PartialEq))]
pub struct LatLonDegrees(f64, f64);

#[derive(Serialize, Deserialize, Default)]
#[cfg_attr(feature = "extra-derive", derive(Debug, PartialEq, Eq))]
pub struct Address {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub street: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub zip: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
}

impl Address {
    pub fn is_empty(&self) -> bool {
        self.street.is_none()
            && self.zip.is_none()
            && self.city.is_none()
            && self.country.is_none()
            && self.state.is_none()
    }
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "extra-derive", derive(Debug, PartialEq))]
pub struct Location {
    #[serde(rename = "deg")]
    pub latlon: LatLonDegrees,

    #[serde(
        rename = "adr",
        skip_serializing_if = "Address::is_empty",
        default = "Default::default"
    )]
    pub address: Address,
}

#[derive(Serialize, Deserialize, Default)]
#[cfg_attr(feature = "extra-derive", derive(Debug, PartialEq, Eq))]
pub struct Contact {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
}

impl Contact {
    pub fn is_empty(&self) -> bool {
        self.email.is_none() && self.phone.is_none()
    }
}

#[derive(Serialize, Deserialize, Default)]
#[cfg_attr(feature = "extra-derive", derive(Debug, PartialEq, Eq))]
pub struct Links {
    #[serde(rename = "www", skip_serializing_if = "Option::is_none")]
    pub homepage: Option<Url>,

    #[serde(rename = "img", skip_serializing_if = "Option::is_none")]
    pub image: Option<Url>,

    #[serde(rename = "img_href", skip_serializing_if = "Option::is_none")]
    pub image_href: Option<Url>,

    #[serde(
        rename = "custom",
        skip_serializing_if = "Vec::is_empty",
        default = "Default::default"
    )]
    pub custom: Vec<CustomLink>,
}

impl Links {
    pub fn is_empty(&self) -> bool {
        let Self {
            homepage,
            image,
            image_href,
            custom,
        } = self;
        homepage.is_none() && image.is_none() && image_href.is_none() && custom.is_empty()
    }
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "extra-derive", derive(Debug, PartialEq, Eq))]
pub struct Activity {
    pub at: i64,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub by: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "extra-derive", derive(Debug))]
pub struct PlaceRoot {
    pub id: String,

    #[serde(rename = "lic")]
    pub license: String,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "extra-derive", derive(Debug))]
pub struct PlaceRevision {
    #[serde(rename = "rev")]
    pub revision: u64,

    pub created: Activity,

    #[serde(rename = "tit")]
    pub title: String,

    #[serde(rename = "dsc")]
    pub description: String,

    #[serde(rename = "loc")]
    pub location: Location,

    #[serde(
        rename = "cnt",
        skip_serializing_if = "Contact::is_empty",
        default = "Default::default"
    )]
    pub contact: Contact,

    #[serde(rename = "hrs", skip_serializing_if = "Option::is_none")]
    pub opening_hours: Option<String>,

    #[serde(rename = "fnd", skip_serializing_if = "Option::is_none")]
    pub founded_on: Option<NaiveDate>,

    #[serde(
        rename = "lnk",
        skip_serializing_if = "Links::is_empty",
        default = "Default::default"
    )]
    pub links: Links,

    #[serde(
        rename = "tag",
        skip_serializing_if = "Vec::is_empty",
        default = "Default::default"
    )]
    pub tags: Vec<String>,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "extra-derive", derive(Debug))]
pub struct PlaceHistory {
    pub place: PlaceRoot,
    pub revisions: Vec<(PlaceRevision, Vec<ReviewStatusLog>)>,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "extra-derive", derive(Debug))]
pub struct ActivityLog {
    pub at: i64,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub by: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ctx: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "extra-derive", derive(Debug))]
pub struct ReviewStatusLog {
    pub rev: u64,
    pub act: ActivityLog,
    pub status: ReviewStatus,
}

impl From<Entry> for UpdatePlace {
    fn from(e: Entry) -> Self {
        let Entry {
            version,
            title,
            description,
            lat,
            lng,
            street,
            zip,
            city,
            country,
            state,
            contact_name,
            email,
            telephone,
            homepage,
            opening_hours,
            founded_on,
            categories,
            tags,
            image_url,
            image_link_url,
            custom_links,
            ..
        } = e;

        UpdatePlace {
            version,
            title,
            description,
            lat,
            lng,
            street,
            zip,
            city,
            country,
            state,
            contact_name,
            email,
            telephone,
            homepage,
            opening_hours,
            founded_on,
            categories,
            tags,
            image_url,
            image_link_url,
            links: custom_links,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "extra-derive", derive(Debug, Clone, PartialEq))]
pub struct Error {
    /// HTTP status code
    pub status: u16,
    /// Error message
    pub message: String,
}
