use crate::{
    core::{
        error::{Error, ParameterError},
        prelude::*,
        usecases,
    },
    infrastructure::{db::sqlite, error::*, flows::prelude::*},
    ports::web::{guards::*, tantivy::SearchEngine},
};
use maud::Markup;
use num_traits::FromPrimitive;
use rocket::{
    self,
    http::{ContentType, RawStr},
    request::Form,
    response::{
        content::{Content, Css, Html, JavaScript},
        Flash, Redirect,
    },
    Route,
};

mod login;
mod password;
mod register;
#[cfg(test)]
mod tests;
mod view;

const MAP_JS: &str = include_str!("map.js");
const MAIN_CSS: &str = include_str!("main.css");
const CLEARANCE_HTML: &str = include_str!("../../../../ofdb-app-clearance/index.html");
const CLEARANCE_JS: &str = include_str!("../../../../ofdb-app-clearance/pkg/clearance.js");
const CLEARANCE_WASM: &[u8] =
    include_bytes!("../../../../ofdb-app-clearance/pkg/clearance_bg.wasm");
const APP_HTML: &str = include_str!("../../../../ofdb-app/index.html");
const APP_JS: &str = include_str!("../../../../ofdb-app/pkg/ofdb_app.js");
const APP_WASM: &[u8] = include_bytes!("../../../../ofdb-app/pkg/ofdb_app_bg.wasm");

type Result<T> = std::result::Result<T, AppError>;

#[get("/")]
pub fn get_index_user(account: Option<Account>) -> Markup {
    view::index(account.as_ref().map(Account::email))
}

#[get("/", rank = 2)]
pub fn get_index() -> Markup {
    view::index(None)
}

#[get("/index.html")]
pub fn get_index_html() -> Markup {
    view::index(None)
}

#[get("/clearance")]
pub fn get_clearance_html() -> Html<&'static str> {
    Html(CLEARANCE_HTML)
}

#[get("/pkg/clearance.js")]
pub fn get_clearance_js() -> JavaScript<&'static str> {
    JavaScript(CLEARANCE_JS)
}

#[get("/pkg/clearance_bg.wasm")]
pub fn get_clearance_wasm() -> Content<&'static [u8]> {
    Content(ContentType::WASM, CLEARANCE_WASM)
}

#[get("/app.html")]
pub fn get_app_html() -> Html<&'static str> {
    Html(APP_HTML)
}

#[get("/pkg/ofdb_app.js")]
pub fn get_app_js() -> JavaScript<&'static str> {
    JavaScript(APP_JS)
}

#[get("/pkg/ofdb_app_bg.wasm")]
pub fn get_app_wasm() -> Content<&'static [u8]> {
    Content(ContentType::WASM, APP_WASM)
}

#[get("/search?<q>&<limit>")]
pub fn get_search(search_engine: SearchEngine, q: &RawStr, limit: Option<usize>) -> Result<Markup> {
    let q = q.url_decode()?;
    let entries = usecases::global_search(&search_engine, &q, limit.unwrap_or(10))?;
    Ok(view::search_results(None, &q, &entries))
}

#[get("/search-users?<email>")]
pub fn get_search_users(
    pool: sqlite::Connections,
    email: &RawStr,
    account: Account,
) -> Result<Markup> {
    let email = email.url_decode()?;
    {
        let db = pool.shared()?;
        let admin = usecases::authorize_user_by_email(&*db, account.email(), Role::Admin)?;
        let users: Vec<_> = db.try_get_user_by_email(&email)?.into_iter().collect();
        Ok(view::user_search_result(&admin.email, &users))
    }
}

#[derive(FromForm)]
pub struct ChangeUserRoleAction {
    email: String,
    role: u8,
}

#[post("/change-user-role", data = "<data>")]
pub fn post_change_user_role(
    db: sqlite::Connections,
    account: Account,
    data: Form<ChangeUserRoleAction>,
) -> std::result::Result<Redirect, Flash<Redirect>> {
    let d = data.into_inner();
    match Role::from_u8(d.role) {
        None => Err(Flash::error(
            Redirect::to(uri!(get_search_users:d.email)),
            "Failed to change user role: invalid role.",
        )),
        Some(role) => match change_user_role(&db, account.email(), &d.email, role) {
            Err(_) => Err(Flash::error(
                Redirect::to(uri!(get_search_users:d.email)),
                "Failed to change user role.",
            )),
            Ok(_) => Ok(Redirect::to(uri!(get_search_users:d.email))),
        },
    }
}

#[get("/map.js")]
pub fn get_map_js() -> JavaScript<&'static str> {
    JavaScript(MAP_JS)
}

#[get("/main.css")]
pub fn get_main_css() -> Css<&'static str> {
    Css(MAIN_CSS)
}

#[get("/places/<id>/history")]
pub fn get_place_history(db: sqlite::Connections, id: &RawStr, account: Account) -> Result<Markup> {
    let db = db.shared()?;
    let user = db
        .try_get_user_by_email(account.email())?
        .ok_or(Error::Parameter(ParameterError::Unauthorized))?;
    let place_history = {
        // The history contains e-mail addresses of registered users
        // and is only permitted for scouts and admins!
        usecases::authorize_user_by_email(&*db, &account.email(), Role::Scout)?;

        db.get_place_history(&id, None)?
    };
    Ok(view::place_history(&user, &place_history))
}

#[get("/places/<id>/review")]
pub fn get_place_review(db: sqlite::Connections, id: &RawStr, account: Account) -> Result<Markup> {
    let db = db.shared()?;
    // Only scouts and admins are entitled to review places
    let reviewer_email =
        usecases::authorize_user_by_email(&*db, &account.email(), Role::Scout)?.email;
    let (place, review_status) = db.get_place(&id)?;
    Ok(view::place_review(&reviewer_email, &place, review_status))
}

#[derive(FromForm)]
pub struct Review {
    pub comment: String,
    pub status: i16,
}

#[post("/places/<id>/review", data = "<review>")]
pub fn post_place_review(
    db: sqlite::Connections,
    search_engine: SearchEngine,
    id: &RawStr,
    review: Form<Review>,
    account: Account,
) -> std::result::Result<Redirect, Flash<Redirect>> {
    let Review { status, comment } = review.into_inner();
    let id = id.as_str();
    review_place(&db, account.email(), status, comment, id, search_engine)
        .map(|_| Redirect::to(uri!(get_entry: id)))
        .map_err(|_| {
            Flash::error(
                Redirect::to(uri!(get_place_review: id)),
                "Failed to archive the place.",
            )
        })
}

fn review_place(
    db: &sqlite::Connections,
    email: &str,
    status: i16,
    comment: String,
    id: &str,
    mut search_engine: SearchEngine,
) -> Result<()> {
    let reviewer_email = {
        let db = db.shared()?;
        usecases::authorize_user_by_email(&*db, email, Role::Scout)?.email
    };
    let status = ReviewStatus::try_from(status)
        .ok_or_else(|| Error::Parameter(ParameterError::RatingContext(status.to_string())))?;
    // TODO: Record context information
    let context = None;
    let review = usecases::Review {
        context,
        reviewer_email: reviewer_email.into(),
        status,
        comment: Some(comment),
    };
    let update_count = review_places(&db, &mut search_engine, &[&id], review)?;
    if update_count == 0 {
        return Err(Error::Repo(RepoError::NotFound).into());
    }
    Ok(())
}

#[get("/entries/<id>")]
pub fn get_entry(
    pool: sqlite::Connections,
    id: &RawStr,
    account: Option<Account>,
) -> Result<Markup> {
    //TODO: dry out
    let (user, place, ratings): (Option<User>, _, _) = {
        let db = pool.shared()?;
        let (place, _) = db.get_place(id.as_str())?;
        let ratings = db.load_ratings_of_place(place.id.as_ref())?;
        let ratings_with_comments = db.zip_ratings_with_comments(ratings)?;
        let user = if let Some(a) = account {
            db.try_get_user_by_email(a.email())?
        } else {
            None
        };
        (user, place, ratings_with_comments)
    };
    Ok(match user {
        Some(u) => view::entry(Some(&u.email), (place, ratings, u.role).into()),
        None => view::entry(None, (place, ratings).into()),
    })
}

#[get("/events/<id>")]
pub fn get_event(
    pool: sqlite::Connections,
    id: &RawStr,
    account: Option<Account>,
) -> Result<Markup> {
    let (user, mut ev): (Option<User>, _) = {
        let db = pool.shared()?;
        let ev = usecases::get_event(&*db, &id)?;
        let user = if let Some(a) = account {
            db.try_get_user_by_email(a.email())?
        } else {
            None
        };
        (user, ev)
    };

    // TODO:Make sure within usecase that the creator email
    // is not shown to unregistered users
    ev.created_by = None;

    Ok(view::event(user, ev))
}

#[post("/events/<id>/archive")]
pub fn post_archive_event(
    account: Account,
    pool: sqlite::Connections,
    mut search_engine: SearchEngine,
    id: &RawStr,
) -> std::result::Result<Redirect, Flash<Redirect>> {
    let archived_by_email = pool
        .shared()
        .and_then(|db| {
            // Only scouts and admins are entitled to review events
            let user = usecases::authorize_user_by_email(&*db, &account.email(), Role::Scout)?;
            Ok(user.email)
        })
        .map_err(|_| {
            Flash::error(
                Redirect::to(uri!(get_event: id)),
                "Failed to achive the event.",
            )
        })?;
    archive_events(&pool, &mut search_engine, &[id], &archived_by_email)
        .map_err(|_| {
            Flash::error(
                Redirect::to(uri!(get_event: id)),
                "Failed to achive the event.",
            )
        })
        .map(|update_count| {
            if update_count != 1 {
                log::info!("Archived more than one event: {}", update_count);
            }
            Redirect::to("/events") //TODO: use uri! macro
        })
}

#[get("/events?<query..>")]
pub fn get_events_chronologically(
    db: sqlite::Connections,
    search_engine: SearchEngine,
    mut query: usecases::EventQuery,
    account: Option<Account>,
) -> Result<Markup> {
    if query.created_by.is_some() {
        return Err(Error::Parameter(ParameterError::Unauthorized).into());
    }

    if query.start_min.is_none() && query.start_max.is_none() {
        let start_min = chrono::Utc::now()
            .checked_sub_signed(chrono::Duration::days(1))
            .unwrap()
            .naive_utc();
        query.start_min = Some(start_min.into());
    }

    let events = usecases::query_events(&*db.shared()?, &search_engine, query)?;
    let email = account.as_ref().map(Account::email);
    Ok(view::events(email, &events))
}

#[get("/dashboard")]
pub fn get_dashboard(db: sqlite::Connections, account: Account) -> Result<Markup> {
    let db = db.shared()?;
    let tag_count = db.count_tags()?;
    let place_count = db.count_places()?;
    let user_count = db.count_users()?;
    let event_count = db.count_events()?;
    let user = db
        .try_get_user_by_email(account.email())?
        .ok_or(Error::Parameter(ParameterError::Unauthorized))?;
    if user.role == Role::Admin {
        return Ok(view::dashboard(view::DashBoardPresenter {
            user,
            place_count,
            event_count,
            tag_count,
            user_count,
        }));
    }
    Err(Error::Parameter(ParameterError::Unauthorized).into())
}

#[derive(FromForm)]
pub struct ArchiveAction {
    ids: String,
    place_id: String,
}

#[post("/comments/actions/archive", data = "<data>")]
pub fn post_comments_archive(
    account: Account,
    db: sqlite::Connections,
    data: Form<ArchiveAction>,
) -> std::result::Result<Redirect, Flash<Redirect>> {
    //TODO: dry out
    let d = data.into_inner();
    let ids: Vec<_> = d.ids.split(',').filter(|id| !id.is_empty()).collect();
    match archive_comments(&db, account.email(), &ids) {
        Err(_) => Err(Flash::error(
            Redirect::to(uri!(get_entry:d.place_id)),
            "Failed to achive the comment.",
        )),
        Ok(_) => Ok(Redirect::to(uri!(get_entry:d.place_id))),
    }
}

#[post("/ratings/actions/archive", data = "<data>")]
pub fn post_ratings_archive(
    account: Account,
    db: sqlite::Connections,
    mut search_engine: SearchEngine,
    data: Form<ArchiveAction>,
) -> std::result::Result<Redirect, Flash<Redirect>> {
    let d = data.into_inner();
    let ids: Vec<_> = d.ids.split(',').filter(|id| !id.is_empty()).collect();
    match archive_ratings(&db, &mut search_engine, account.email(), &ids) {
        Err(_) => Err(Flash::error(
            Redirect::to(uri!(get_entry:d.place_id)),
            "Failed to archive the rating.",
        )),
        Ok(_) => Ok(Redirect::to(uri!(get_entry:d.place_id))),
    }
}

pub fn routes() -> Vec<Route> {
    routes![
        get_clearance_html,
        get_clearance_js,
        get_clearance_wasm,
        get_app_html,
        get_app_js,
        get_app_wasm,
        get_index_user,
        get_index,
        get_index_html,
        get_dashboard,
        get_search,
        get_entry,
        get_place_history,
        get_place_review,
        post_place_review,
        get_events_chronologically,
        get_event,
        get_main_css,
        get_map_js,
        get_search_users,
        post_comments_archive,
        post_ratings_archive,
        post_change_user_role,
        post_archive_event,
        login::get_login,
        login::post_login,
        login::post_logout,
        register::get_register,
        register::post_register,
        register::get_email_confirmation,
        password::get_reset_password,
        password::post_reset_password_request,
        password::post_reset_password,
    ]
}
