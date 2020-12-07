// TODO: use crate::{
// TODO:     core::{
// TODO:         error::{Error, ParameterError},
// TODO:         prelude::*,
// TODO:         usecases,
// TODO:     },
// TODO:     infrastructure::{db::sqlite, error::*, flows::prelude::*},
// TODO:     ports::web::{guards::*, tantivy::SearchEngine},
// TODO: };
// TODO: use maud::Markup;
// TODO: use num_traits::FromPrimitive;
// TODO: use rocket::{
// TODO:     self,
// TODO:     http::{ContentType, RawStr},
// TODO:     request::Form,
// TODO:     response::{
// TODO:         content::{Content, Css, Html, JavaScript},
// TODO:         Flash, Redirect,
// TODO:     },
// TODO:     Route,
// TODO: };
// TODO: 
// TODO: mod login;
// TODO: mod password;
// TODO: mod register;
// TODO: #[cfg(test)]
// TODO: mod tests;
// TODO: mod view;
// TODO: 
// TODO: const MAP_JS: &str = include_str!("map.js");
// TODO: const MAIN_CSS: &str = include_str!("main.css");
// TODO: const APP_HTML: &str = include_str!("../../../../ofdb-app/index.html");
// TODO: const APP_JS: &str = include_str!("../../../../ofdb-app/pkg/ofdb_app.js");
// TODO: const APP_WASM: &[u8] = include_bytes!("../../../../ofdb-app/pkg/ofdb_app_bg.wasm");
// TODO: 
// TODO: type Result<T> = std::result::Result<T, AppError>;
// TODO: 
// TODO: #[get("/")]
// TODO: pub fn get_index_user(account: Account) -> Markup {
// TODO:     view::index(Some(&account.email()))
// TODO: }
// TODO: 
// TODO: #[get("/", rank = 2)]
// TODO: pub fn get_index() -> Markup {
// TODO:     view::index(None)
// TODO: }
// TODO: 
// TODO: #[get("/index.html")]
// TODO: pub fn get_index_html() -> Markup {
// TODO:     view::index(None)
// TODO: }
// TODO: 
// TODO: #[get("/app.html")]
// TODO: pub fn get_app_html() -> Html<&'static str> {
// TODO:     Html(APP_HTML)
// TODO: }
// TODO: 
// TODO: #[get("/pkg/ofdb_app.js")]
// TODO: pub fn get_app_js() -> JavaScript<&'static str> {
// TODO:     JavaScript(APP_JS)
// TODO: }
// TODO: 
// TODO: #[get("/pkg/ofdb_app_bg.wasm")]
// TODO: pub fn get_app_wasm() -> Content<&'static [u8]> {
// TODO:     Content(ContentType::WASM, APP_WASM)
// TODO: }
// TODO: 
// TODO: #[get("/search?<q>&<limit>")]
// TODO: pub fn get_search(search_engine: SearchEngine, q: &RawStr, limit: Option<usize>) -> Result<Markup> {
// TODO:     let q = q.url_decode()?;
// TODO:     let entries = usecases::global_search(&search_engine, &q, limit.unwrap_or(10))?;
// TODO:     Ok(view::search_results(None, &q, &entries))
// TODO: }
// TODO: 
// TODO: #[get("/search-users?<email>")]
// TODO: pub fn get_search_users(
// TODO:     pool: sqlite::Connections,
// TODO:     email: &RawStr,
// TODO:     account: Account,
// TODO: ) -> Result<Markup> {
// TODO:     let email = email.url_decode()?;
// TODO:     {
// TODO:         let db = pool.shared()?;
// TODO:         let admin = usecases::authorize_user_by_email(&*db, account.email(), Role::Admin)?;
// TODO:         let users: Vec<_> = db.try_get_user_by_email(&email)?.into_iter().collect();
// TODO:         Ok(view::user_search_result(&admin.email, &users))
// TODO:     }
// TODO: }
// TODO: 
// TODO: #[derive(FromForm)]
// TODO: pub struct ChangeUserRoleAction {
// TODO:     email: String,
// TODO:     role: u8,
// TODO: }
// TODO: 
// TODO: #[post("/change-user-role", data = "<data>")]
// TODO: pub fn post_change_user_role(
// TODO:     db: sqlite::Connections,
// TODO:     account: Account,
// TODO:     data: Form<ChangeUserRoleAction>,
// TODO: ) -> std::result::Result<Redirect, Flash<Redirect>> {
// TODO:     let d = data.into_inner();
// TODO:     match Role::from_u8(d.role) {
// TODO:         None => Err(Flash::error(
// TODO:             Redirect::to(uri!(get_search_users:d.email)),
// TODO:             "Failed to change user role: invalid role.",
// TODO:         )),
// TODO:         Some(role) => match change_user_role(&db, account.email(), &d.email, role) {
// TODO:             Err(_) => Err(Flash::error(
// TODO:                 Redirect::to(uri!(get_search_users:d.email)),
// TODO:                 "Failed to change user role.",
// TODO:             )),
// TODO:             Ok(_) => Ok(Redirect::to(uri!(get_search_users:d.email))),
// TODO:         },
// TODO:     }
// TODO: }
// TODO: 
// TODO: #[get("/map.js")]
// TODO: pub fn get_map_js() -> JavaScript<&'static str> {
// TODO:     JavaScript(MAP_JS)
// TODO: }
// TODO: 
// TODO: #[get("/main.css")]
// TODO: pub fn get_main_css() -> Css<&'static str> {
// TODO:     Css(MAIN_CSS)
// TODO: }
// TODO: 
// TODO: #[get("/places/<id>/history")]
// TODO: pub fn get_place_history(db: sqlite::Connections, id: &RawStr, account: Account) -> Result<Markup> {
// TODO:     let db = db.shared()?;
// TODO:     let user = db
// TODO:         .try_get_user_by_email(account.email())?
// TODO:         .ok_or(Error::Parameter(ParameterError::Unauthorized))?;
// TODO:     let place_history = {
// TODO:         // The history contains e-mail addresses of registered users
// TODO:         // and is only permitted for scouts and admins!
// TODO:         usecases::authorize_user_by_email(&*db, &account.email(), Role::Scout)?;
// TODO: 
// TODO:         db.get_place_history(&id)?
// TODO:     };
// TODO:     Ok(view::place_history(&user, &place_history))
// TODO: }
// TODO: 
// TODO: #[get("/places/<id>/review")]
// TODO: pub fn get_place_review(db: sqlite::Connections, id: &RawStr, account: Account) -> Result<Markup> {
// TODO:     let db = db.shared()?;
// TODO:     // Only scouts and admins are entitled to review places
// TODO:     let reviewer_email =
// TODO:         usecases::authorize_user_by_email(&*db, &account.email(), Role::Scout)?.email;
// TODO:     let (place, review_status) = db.get_place(&id)?;
// TODO:     Ok(view::place_review(&reviewer_email, &place, review_status))
// TODO: }
// TODO: 
// TODO: #[derive(FromForm)]
// TODO: pub struct Review {
// TODO:     pub comment: String,
// TODO:     pub status: i16,
// TODO: }
// TODO: 
// TODO: #[post("/places/<id>/review", data = "<review>")]
// TODO: pub fn post_place_review(
// TODO:     db: sqlite::Connections,
// TODO:     search_engine: SearchEngine,
// TODO:     id: &RawStr,
// TODO:     review: Form<Review>,
// TODO:     account: Account,
// TODO: ) -> std::result::Result<Redirect, Flash<Redirect>> {
// TODO:     let Review { status, comment } = review.into_inner();
// TODO:     let id = id.as_str();
// TODO:     review_place(&db, account.email(), status, comment, id, search_engine)
// TODO:         .map(|_| Redirect::to(uri!(get_entry: id)))
// TODO:         .map_err(|_| {
// TODO:             Flash::error(
// TODO:                 Redirect::to(uri!(get_place_review: id)),
// TODO:                 "Failed to archive the place.",
// TODO:             )
// TODO:         })
// TODO: }
// TODO: 
// TODO: fn review_place(
// TODO:     db: &sqlite::Connections,
// TODO:     email: &str,
// TODO:     status: i16,
// TODO:     comment: String,
// TODO:     id: &str,
// TODO:     mut search_engine: SearchEngine,
// TODO: ) -> Result<()> {
// TODO:     let reviewer_email = {
// TODO:         let db = db.shared()?;
// TODO:         usecases::authorize_user_by_email(&*db, email, Role::Scout)?.email
// TODO:     };
// TODO:     let status = ReviewStatus::try_from(status)
// TODO:         .ok_or_else(|| Error::Parameter(ParameterError::RatingContext(status.to_string())))?;
// TODO:     // TODO: Record context information
// TODO:     let context = None;
// TODO:     let review = usecases::Review {
// TODO:         context,
// TODO:         reviewer_email: reviewer_email.into(),
// TODO:         status,
// TODO:         comment: Some(comment),
// TODO:     };
// TODO:     let update_count = review_places(&db, &mut search_engine, &[&id], review)?;
// TODO:     if update_count == 0 {
// TODO:         return Err(Error::Repo(RepoError::NotFound).into());
// TODO:     }
// TODO:     Ok(())
// TODO: }
// TODO: 
// TODO: #[get("/entries/<id>")]
// TODO: pub fn get_entry(
// TODO:     pool: sqlite::Connections,
// TODO:     id: &RawStr,
// TODO:     account: Option<Account>,
// TODO: ) -> Result<Markup> {
// TODO:     //TODO: dry out
// TODO:     let (user, place, ratings): (Option<User>, _, _) = {
// TODO:         let db = pool.shared()?;
// TODO:         let (place, _) = db.get_place(id.as_str())?;
// TODO:         let ratings = db.load_ratings_of_place(place.id.as_ref())?;
// TODO:         let ratings_with_comments = db.zip_ratings_with_comments(ratings)?;
// TODO:         let user = if let Some(a) = account {
// TODO:             db.try_get_user_by_email(a.email())?
// TODO:         } else {
// TODO:             None
// TODO:         };
// TODO:         (user, place, ratings_with_comments)
// TODO:     };
// TODO:     Ok(match user {
// TODO:         Some(u) => view::entry(Some(&u.email), (place, ratings, u.role).into()),
// TODO:         None => view::entry(None, (place, ratings).into()),
// TODO:     })
// TODO: }
// TODO: 
// TODO: #[get("/events/<id>")]
// TODO: pub fn get_event(
// TODO:     pool: sqlite::Connections,
// TODO:     id: &RawStr,
// TODO:     account: Option<Account>,
// TODO: ) -> Result<Markup> {
// TODO:     let (user, mut ev): (Option<User>, _) = {
// TODO:         let db = pool.shared()?;
// TODO:         let ev = usecases::get_event(&*db, &id)?;
// TODO:         let user = if let Some(a) = account {
// TODO:             db.try_get_user_by_email(a.email())?
// TODO:         } else {
// TODO:             None
// TODO:         };
// TODO:         (user, ev)
// TODO:     };
// TODO: 
// TODO:     // TODO:Make sure within usecase that the creator email
// TODO:     // is not shown to unregistered users
// TODO:     ev.created_by = None;
// TODO: 
// TODO:     Ok(view::event(user, ev))
// TODO: }
// TODO: 
// TODO: #[post("/events/<id>/archive")]
// TODO: pub fn post_archive_event(
// TODO:     account: Account,
// TODO:     pool: sqlite::Connections,
// TODO:     mut search_engine: SearchEngine,
// TODO:     id: &RawStr,
// TODO: ) -> std::result::Result<Redirect, Flash<Redirect>> {
// TODO:     let archived_by_email = pool
// TODO:         .shared()
// TODO:         .and_then(|db| {
// TODO:             // Only scouts and admins are entitled to review events
// TODO:             let user = usecases::authorize_user_by_email(&*db, &account.email(), Role::Scout)?;
// TODO:             Ok(user.email)
// TODO:         })
// TODO:         .map_err(|_| {
// TODO:             Flash::error(
// TODO:                 Redirect::to(uri!(get_event: id)),
// TODO:                 "Failed to achive the event.",
// TODO:             )
// TODO:         })?;
// TODO:     archive_events(&pool, &mut search_engine, &[id], &archived_by_email)
// TODO:         .map_err(|_| {
// TODO:             Flash::error(
// TODO:                 Redirect::to(uri!(get_event: id)),
// TODO:                 "Failed to achive the event.",
// TODO:             )
// TODO:         })
// TODO:         .map(|update_count| {
// TODO:             if update_count != 1 {
// TODO:                 log::info!("Archived more than one event: {}", update_count);
// TODO:             }
// TODO:             Redirect::to("/events") //TODO: use uri! macro
// TODO:         })
// TODO: }
// TODO: 
// TODO: #[get("/events?<query..>")]
// TODO: pub fn get_events_chronologically(
// TODO:     db: sqlite::Connections,
// TODO:     search_engine: SearchEngine,
// TODO:     mut query: usecases::EventQuery,
// TODO:     account: Option<Account>,
// TODO: ) -> Result<Markup> {
// TODO:     if query.created_by.is_some() {
// TODO:         return Err(Error::Parameter(ParameterError::Unauthorized).into());
// TODO:     }
// TODO: 
// TODO:     if query.start_min.is_none() && query.start_max.is_none() {
// TODO:         let start_min = chrono::Utc::now()
// TODO:             .checked_sub_signed(chrono::Duration::days(1))
// TODO:             .unwrap()
// TODO:             .naive_utc();
// TODO:         query.start_min = Some(start_min.into());
// TODO:     }
// TODO: 
// TODO:     let events = usecases::query_events(&*db.shared()?, &search_engine, query)?;
// TODO:     let email = account.as_ref().map(Account::email);
// TODO:     Ok(view::events(email, &events))
// TODO: }
// TODO: 
// TODO: #[get("/dashboard")]
// TODO: pub fn get_dashboard(db: sqlite::Connections, account: Account) -> Result<Markup> {
// TODO:     let db = db.shared()?;
// TODO:     let tag_count = db.count_tags()?;
// TODO:     let place_count = db.count_places()?;
// TODO:     let user_count = db.count_users()?;
// TODO:     let event_count = db.count_events()?;
// TODO:     let user = db
// TODO:         .try_get_user_by_email(account.email())?
// TODO:         .ok_or(Error::Parameter(ParameterError::Unauthorized))?;
// TODO:     if user.role == Role::Admin {
// TODO:         return Ok(view::dashboard(view::DashBoardPresenter {
// TODO:             user,
// TODO:             place_count,
// TODO:             event_count,
// TODO:             tag_count,
// TODO:             user_count,
// TODO:         }));
// TODO:     }
// TODO:     Err(Error::Parameter(ParameterError::Unauthorized).into())
// TODO: }
// TODO: 
// TODO: #[derive(FromForm)]
// TODO: pub struct ArchiveAction {
// TODO:     ids: String,
// TODO:     place_id: String,
// TODO: }
// TODO: 
// TODO: #[post("/comments/actions/archive", data = "<data>")]
// TODO: pub fn post_comments_archive(
// TODO:     account: Account,
// TODO:     db: sqlite::Connections,
// TODO:     data: Form<ArchiveAction>,
// TODO: ) -> std::result::Result<Redirect, Flash<Redirect>> {
// TODO:     //TODO: dry out
// TODO:     let d = data.into_inner();
// TODO:     let ids: Vec<_> = d.ids.split(',').filter(|id| !id.is_empty()).collect();
// TODO:     match archive_comments(&db, account.email(), &ids) {
// TODO:         Err(_) => Err(Flash::error(
// TODO:             Redirect::to(uri!(get_entry:d.place_id)),
// TODO:             "Failed to achive the comment.",
// TODO:         )),
// TODO:         Ok(_) => Ok(Redirect::to(uri!(get_entry:d.place_id))),
// TODO:     }
// TODO: }
// TODO: 
// TODO: #[post("/ratings/actions/archive", data = "<data>")]
// TODO: pub fn post_ratings_archive(
// TODO:     account: Account,
// TODO:     db: sqlite::Connections,
// TODO:     mut search_engine: SearchEngine,
// TODO:     data: Form<ArchiveAction>,
// TODO: ) -> std::result::Result<Redirect, Flash<Redirect>> {
// TODO:     let d = data.into_inner();
// TODO:     let ids: Vec<_> = d.ids.split(',').filter(|id| !id.is_empty()).collect();
// TODO:     match archive_ratings(&db, &mut search_engine, account.email(), &ids) {
// TODO:         Err(_) => Err(Flash::error(
// TODO:             Redirect::to(uri!(get_entry:d.place_id)),
// TODO:             "Failed to archive the rating.",
// TODO:         )),
// TODO:         Ok(_) => Ok(Redirect::to(uri!(get_entry:d.place_id))),
// TODO:     }
// TODO: }
// TODO: 
// TODO: pub fn routes() -> Vec<Route> {
// TODO:     routes![
// TODO:         get_app_html,
// TODO:         get_app_js,
// TODO:         get_app_wasm,
// TODO:         get_index_user,
// TODO:         get_index,
// TODO:         get_index_html,
// TODO:         get_dashboard,
// TODO:         get_search,
// TODO:         get_entry,
// TODO:         get_place_history,
// TODO:         get_place_review,
// TODO:         post_place_review,
// TODO:         get_events_chronologically,
// TODO:         get_event,
// TODO:         get_main_css,
// TODO:         get_map_js,
// TODO:         get_search_users,
// TODO:         post_comments_archive,
// TODO:         post_ratings_archive,
// TODO:         post_change_user_role,
// TODO:         post_archive_event,
// TODO:         login::get_login,
// TODO:         login::get_login_user,
// TODO:         login::post_login,
// TODO:         login::post_logout,
// TODO:         register::get_register,
// TODO:         register::post_register,
// TODO:         register::get_email_confirmation,
// TODO:         password::get_reset_password,
// TODO:         password::post_reset_password_request,
// TODO:         password::post_reset_password,
// TODO:     ]
// TODO: }
