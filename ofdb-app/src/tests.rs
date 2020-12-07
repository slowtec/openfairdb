// TODO: use super::*;
// TODO: use crate::{
// TODO:     infrastructure::db::{sqlite::Connections, tantivy},
// TODO:     ports::web::tests::{prelude::*, register_user},
// TODO: };
// TODO: 
// TODO: fn setup() -> (
// TODO:     rocket::local::Client,
// TODO:     sqlite::Connections,
// TODO:     tantivy::SearchEngine,
// TODO: ) {
// TODO:     crate::ports::web::tests::setup(vec![("/", super::routes())])
// TODO: }
// TODO: 
// TODO: fn create_user(pool: &Connections, name: &str, role: Role) {
// TODO:     let email = format!("{}@example.com", name);
// TODO:     register_user(&pool, &email, "secret", true);
// TODO:     let mut user = get_user(pool, name);
// TODO:     user.role = role;
// TODO:     pool.exclusive().unwrap().update_user(&user).unwrap();
// TODO: }
// TODO: 
// TODO: fn get_user(pool: &Connections, name: &str) -> User {
// TODO:     let email = format!("{}@example.com", name);
// TODO:     pool.shared()
// TODO:         .unwrap()
// TODO:         .try_get_user_by_email(&email)
// TODO:         .unwrap()
// TODO:         .into_iter()
// TODO:         .next()
// TODO:         .unwrap()
// TODO: }
// TODO: 
// TODO: fn login_user(client: &Client, name: &str) {
// TODO:     client
// TODO:         .post("/login")
// TODO:         .header(ContentType::Form)
// TODO:         .body(format!("email={}%40example.com&password=secret", name))
// TODO:         .dispatch();
// TODO: }
// TODO: 
// TODO: mod events {
// TODO:     use super::*;
// TODO:     use crate::infrastructure::flows::prelude as flows;
// TODO:     use chrono::prelude::*;
// TODO: 
// TODO:     #[test]
// TODO:     fn search_events() {
// TODO:         let (client, db, mut search_engine) = setup();
// TODO:         let new_events = vec![
// TODO:             usecases::NewEvent {
// TODO:                 title: "x".into(),
// TODO:                 start: Timestamp::from(
// TODO:                     chrono::Utc::now()
// TODO:                         .checked_sub_signed(chrono::Duration::days(2))
// TODO:                         .unwrap(),
// TODO:                 )
// TODO:                 .into_inner(),
// TODO:                 tags: Some(vec!["foo".into()]),
// TODO:                 registration: Some("email".into()),
// TODO:                 email: Some("test@example.com".into()),
// TODO:                 created_by: Some("test@example.com".into()),
// TODO:                 ..Default::default()
// TODO:             },
// TODO:             usecases::NewEvent {
// TODO:                 title: "x".into(),
// TODO:                 start: Timestamp::from(
// TODO:                     chrono::Utc::now()
// TODO:                         .checked_sub_signed(chrono::Duration::hours(2))
// TODO:                         .unwrap(),
// TODO:                 )
// TODO:                 .into_inner(),
// TODO:                 tags: Some(vec!["bla".into()]),
// TODO:                 registration: Some("email".into()),
// TODO:                 email: Some("test@example.com".into()),
// TODO:                 created_by: Some("test@example.com".into()),
// TODO:                 ..Default::default()
// TODO:             },
// TODO:             usecases::NewEvent {
// TODO:                 title: "foo".into(),
// TODO:                 start: Timestamp::from(
// TODO:                     chrono::Utc::now()
// TODO:                         .checked_add_signed(chrono::Duration::days(1))
// TODO:                         .unwrap(),
// TODO:                 )
// TODO:                 .into_inner(),
// TODO:                 registration: Some("email".into()),
// TODO:                 email: Some("test@example.com".into()),
// TODO:                 created_by: Some("test@example.com".into()),
// TODO:                 ..Default::default()
// TODO:             },
// TODO:             usecases::NewEvent {
// TODO:                 title: "x".into(),
// TODO:                 start: Timestamp::from(
// TODO:                     chrono::Utc::now()
// TODO:                         .checked_add_signed(chrono::Duration::days(2))
// TODO:                         .unwrap(),
// TODO:                 )
// TODO:                 .into_inner(),
// TODO:                 tags: Some(vec!["foo".into()]),
// TODO:                 registration: Some("email".into()),
// TODO:                 email: Some("test@example.com".into()),
// TODO:                 created_by: Some("test@example.com".into()),
// TODO:                 ..Default::default()
// TODO:             },
// TODO:         ];
// TODO:         let gw = DummyNotifyGW;
// TODO:         let event_ids = {
// TODO:             let mut event_ids = Vec::with_capacity(new_events.len());
// TODO:             for e in new_events {
// TODO:                 let e = flows::create_event(&db, &mut search_engine, &gw, None, e).unwrap();
// TODO:                 event_ids.push(e.id);
// TODO:             }
// TODO:             event_ids
// TODO:         };
// TODO: 
// TODO:         // All events
// TODO:         let mut res = client.get("/events").dispatch();
// TODO:         assert_eq!(res.status(), Status::Ok);
// TODO:         let body_str = res.body().and_then(|b| b.into_string()).unwrap();
// TODO:         assert!(!body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[0])));
// TODO:         assert!(body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[1])));
// TODO:         assert!(body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[2])));
// TODO:         assert!(body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[3])));
// TODO: 
// TODO:         // Search with simple text
// TODO:         let mut res = client.get("/events?text=foo").dispatch();
// TODO:         assert_eq!(res.status(), Status::Ok);
// TODO:         let body_str = res.body().and_then(|b| b.into_string()).unwrap();
// TODO:         assert!(!body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[0])));
// TODO:         assert!(!body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[1])));
// TODO:         assert!(body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[2])));
// TODO:         assert!(body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[3])));
// TODO: 
// TODO:         // Search with hashtag text
// TODO:         let mut res = client.get("/events?text=%23foo").dispatch();
// TODO:         assert_eq!(res.status(), Status::Ok);
// TODO:         let body_str = res.body().and_then(|b| b.into_string()).unwrap();
// TODO:         assert!(!body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[0])));
// TODO:         assert!(!body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[1])));
// TODO:         assert!(!body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[2])));
// TODO:         assert!(body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[3])));
// TODO: 
// TODO:         // Search with tag
// TODO:         let mut res = client.get("/events?tag=foo").dispatch();
// TODO:         assert_eq!(res.status(), Status::Ok);
// TODO:         let body_str = res.body().and_then(|b| b.into_string()).unwrap();
// TODO:         assert!(!body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[0])));
// TODO:         assert!(!body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[1])));
// TODO:         assert!(!body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[2])));
// TODO:         assert!(body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[3])));
// TODO: 
// TODO:         // Search with simple text (not found)
// TODO:         let mut res = client.get("/events?text=bar").dispatch();
// TODO:         assert_eq!(res.status(), Status::Ok);
// TODO:         let body_str = res.body().and_then(|b| b.into_string()).unwrap();
// TODO:         assert!(!body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[0])));
// TODO:         assert!(!body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[1])));
// TODO:         assert!(!body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[2])));
// TODO:         assert!(!body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[3])));
// TODO: 
// TODO:         // Search with hashtag text (not found)
// TODO:         let mut res = client.get("/events?text=%23bar").dispatch();
// TODO:         assert_eq!(res.status(), Status::Ok);
// TODO:         let body_str = res.body().and_then(|b| b.into_string()).unwrap();
// TODO:         assert!(!body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[0])));
// TODO:         assert!(!body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[1])));
// TODO:         assert!(!body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[2])));
// TODO:         assert!(!body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[3])));
// TODO: 
// TODO:         // Search with tag (not found)
// TODO:         let mut res = client.get("/events?tag=bar").dispatch();
// TODO:         assert_eq!(res.status(), Status::Ok);
// TODO:         let body_str = res.body().and_then(|b| b.into_string()).unwrap();
// TODO:         assert!(!body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[0])));
// TODO:         assert!(!body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[1])));
// TODO:         assert!(!body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[2])));
// TODO:         assert!(!body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[3])));
// TODO:     }
// TODO: 
// TODO:     #[test]
// TODO:     fn get_a_list_of_events_filtered_by_tags() {
// TODO:         let (client, db, mut search_engine) = setup();
// TODO:         let new_events = vec![
// TODO:             usecases::NewEvent {
// TODO:                 title: "x".into(),
// TODO:                 start: Timestamp::from(
// TODO:                     chrono::Utc::now()
// TODO:                         .checked_sub_signed(chrono::Duration::hours(2))
// TODO:                         .unwrap(),
// TODO:                 )
// TODO:                 .into_inner(),
// TODO:                 tags: Some(vec!["bla".into(), "blub".into()]),
// TODO:                 registration: Some("email".into()),
// TODO:                 email: Some("test@example.com".into()),
// TODO:                 created_by: Some("test@example.com".into()),
// TODO:                 ..Default::default()
// TODO:             },
// TODO:             usecases::NewEvent {
// TODO:                 title: "x".into(),
// TODO:                 start: Timestamp::from(
// TODO:                     chrono::Utc::now()
// TODO:                         .checked_add_signed(chrono::Duration::days(2))
// TODO:                         .unwrap(),
// TODO:                 )
// TODO:                 .into_inner(),
// TODO:                 tags: Some(vec!["bli".into(), "blub".into()]),
// TODO:                 registration: Some("email".into()),
// TODO:                 email: Some("test@example.com".into()),
// TODO:                 created_by: Some("test@example.com".into()),
// TODO:                 ..Default::default()
// TODO:             },
// TODO:             usecases::NewEvent {
// TODO:                 title: "x".into(),
// TODO:                 start: Timestamp::from(
// TODO:                     chrono::Utc::now()
// TODO:                         .checked_sub_signed(chrono::Duration::days(2))
// TODO:                         .unwrap(),
// TODO:                 )
// TODO:                 .into_inner(),
// TODO:                 tags: Some(vec!["blub".into()]),
// TODO:                 registration: Some("email".into()),
// TODO:                 email: Some("test@example.com".into()),
// TODO:                 created_by: Some("test@example.com".into()),
// TODO:                 ..Default::default()
// TODO:             },
// TODO:         ];
// TODO:         let gw = DummyNotifyGW;
// TODO:         let event_ids = {
// TODO:             let mut event_ids = Vec::with_capacity(new_events.len());
// TODO:             for e in new_events {
// TODO:                 let e = flows::create_event(&db, &mut search_engine, &gw, None, e).unwrap();
// TODO:                 event_ids.push(e.id);
// TODO:             }
// TODO:             event_ids
// TODO:         };
// TODO: 
// TODO:         let mut res = client.get("/events?tag=blub&tag=bli").dispatch();
// TODO:         assert_eq!(res.status(), Status::Ok);
// TODO:         let body_str = res.body().and_then(|b| b.into_string()).unwrap();
// TODO:         assert!(!body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[0])));
// TODO:         assert!(body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[1])));
// TODO:         assert!(!body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[2])));
// TODO: 
// TODO:         let mut res = client.get("/events?tag=blub").dispatch();
// TODO:         assert_eq!(res.status(), Status::Ok);
// TODO:         let body_str = res.body().and_then(|b| b.into_string()).unwrap();
// TODO:         assert!(body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[0])));
// TODO:         assert!(body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[1])));
// TODO:         assert!(!body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[2])));
// TODO:     }
// TODO: 
// TODO:     #[test]
// TODO:     fn get_a_single_event() {
// TODO:         let (client, db, _) = setup();
// TODO:         let events = vec![Event {
// TODO:             id: "1234".into(),
// TODO:             title: "A great event".into(),
// TODO:             description: Some("Foo bar baz".into()),
// TODO:             start: NaiveDateTime::from_timestamp(0, 0),
// TODO:             end: None,
// TODO:             location: None,
// TODO:             contact: None,
// TODO:             tags: vec!["bla".into()],
// TODO:             homepage: None,
// TODO:             created_by: None,
// TODO:             registration: Some(RegistrationType::Email),
// TODO:             organizer: None,
// TODO:             archived: None,
// TODO:             image_url: None,
// TODO:             image_link_url: None,
// TODO:         }];
// TODO: 
// TODO:         {
// TODO:             let db_conn = db.exclusive().unwrap();
// TODO:             for e in events {
// TODO:                 db_conn.create_event(e).unwrap();
// TODO:             }
// TODO:         }
// TODO: 
// TODO:         let mut res = client.get("/events/1234").dispatch();
// TODO:         assert_eq!(res.status(), Status::Ok);
// TODO:         let body_str = res.body().and_then(|b| b.into_string()).unwrap();
// TODO:         assert!(body_str.contains("<h2>A great event</h2>"));
// TODO:         assert!(body_str.contains("Foo bar baz</p>"));
// TODO:     }
// TODO: }
// TODO: 
// TODO: mod index {
// TODO:     use super::*;
// TODO:     #[test]
// TODO:     fn get_the_index_html() {
// TODO:         let (client, _db, _) = setup();
// TODO:         let mut index = client.get("/").dispatch();
// TODO:         assert_eq!(index.status(), Status::Ok);
// TODO: 
// TODO:         let mut index_html = client.get("/index.html").dispatch();
// TODO:         assert_eq!(index_html.status(), Status::Ok);
// TODO: 
// TODO:         let index_str = index.body().and_then(|b| b.into_string()).unwrap();
// TODO:         let index_html_str = index_html.body().and_then(|b| b.into_string()).unwrap();
// TODO: 
// TODO:         assert_eq!(index_html_str, index_str);
// TODO:         assert!(index_str.contains("<form action=\"search\""));
// TODO:         assert!(index_str.contains("<input type=\"text\""));
// TODO:     }
// TODO: }
// TODO: 
// TODO: mod entry {
// TODO:     use super::*;
// TODO:     use crate::{core::usecases, infrastructure::flows};
// TODO: 
// TODO:     fn create_place_with_rating(
// TODO:         db: &sqlite::Connections,
// TODO:         search: &mut tantivy::SearchEngine,
// TODO:     ) -> (String, String, String) {
// TODO:         let e = usecases::NewPlace {
// TODO:             title: "entry".into(),
// TODO:             description: "desc".into(),
// TODO:             lat: 3.7,
// TODO:             lng: -50.0,
// TODO:             street: None,
// TODO:             zip: None,
// TODO:             city: None,
// TODO:             country: None,
// TODO:             state: None,
// TODO:             email: None,
// TODO:             telephone: None,
// TODO:             homepage: None,
// TODO:             opening_hours: None,
// TODO:             categories: vec![],
// TODO:             tags: vec![],
// TODO:             license: "CC0-1.0".into(),
// TODO:             image_url: None,
// TODO:             image_link_url: None,
// TODO:         };
// TODO:         let gw = DummyNotifyGW;
// TODO:         let e_id = flows::prelude::create_place(db, search, &gw, e, None)
// TODO:             .unwrap()
// TODO:             .id;
// TODO:         let r = usecases::NewPlaceRating {
// TODO:             title: "A rating".into(),
// TODO:             comment: "Foo".into(),
// TODO:             context: ofdb_boundary::RatingContext::Diversity,
// TODO:             source: None,
// TODO:             user: None,
// TODO:             value: 1.into(),
// TODO:             entry: e_id.clone().into(),
// TODO:         };
// TODO:         let (r_id, c_id) = flows::prelude::create_rating(db, search, r).unwrap();
// TODO:         (e_id.into(), r_id, c_id)
// TODO:     }
// TODO: 
// TODO:     #[test]
// TODO:     fn get_entry_details() {
// TODO:         let (client, db, mut search) = setup();
// TODO:         let (id, _, _) = create_place_with_rating(&db, &mut search);
// TODO:         let mut res = client.get(format!("/entries/{}", id)).dispatch();
// TODO:         assert_eq!(res.status(), Status::Ok);
// TODO:         let body_str = res.body().and_then(|b| b.into_string()).unwrap();
// TODO:         assert_eq!(body_str.contains("<form"), false);
// TODO:         assert_eq!(
// TODO:             body_str.contains("action=\"/comments/actions/archive\""),
// TODO:             false
// TODO:         );
// TODO:     }
// TODO: 
// TODO:     #[test]
// TODO:     fn get_entry_details_as_admin() {
// TODO:         let (client, db, mut search) = setup();
// TODO:         let (id, _, _) = create_place_with_rating(&db, &mut search);
// TODO:         create_user(&db, "foo", Role::Admin);
// TODO:         login_user(&client, "foo");
// TODO:         let mut res = client.get(format!("/entries/{}", id)).dispatch();
// TODO:         assert_eq!(res.status(), Status::Ok);
// TODO:         let body_str = res.body().and_then(|b| b.into_string()).unwrap();
// TODO:         assert_eq!(body_str.contains("<form"), true);
// TODO:         assert_eq!(
// TODO:             body_str.contains("action=\"/comments/actions/archive\""),
// TODO:             true
// TODO:         );
// TODO:     }
// TODO: 
// TODO:     #[test]
// TODO:     fn get_entry_details_as_scout() {
// TODO:         let (client, db, mut search) = setup();
// TODO:         let (id, _, _) = create_place_with_rating(&db, &mut search);
// TODO:         create_user(&db, "foo", Role::Scout);
// TODO:         login_user(&client, "foo");
// TODO:         let mut res = client.get(format!("/entries/{}", id)).dispatch();
// TODO:         assert_eq!(res.status(), Status::Ok);
// TODO:         let body_str = res.body().and_then(|b| b.into_string()).unwrap();
// TODO:         assert_eq!(body_str.contains("<form"), true);
// TODO:         assert_eq!(
// TODO:             body_str.contains("action=\"/comments/actions/archive\""),
// TODO:             true
// TODO:         );
// TODO:     }
// TODO: 
// TODO:     #[test]
// TODO:     fn archive_comment_as_admin() {
// TODO:         let (client, db, mut search) = setup();
// TODO:         create_user(&db, "foo", Role::Admin);
// TODO:         login_user(&client, "foo");
// TODO:         let (e_id, _, c_id) = create_place_with_rating(&db, &mut search);
// TODO:         let comment = db.shared().unwrap().load_comment(&c_id).unwrap();
// TODO:         assert!(comment.archived_at.is_none());
// TODO:         let res = client
// TODO:             .post("/comments/actions/archive")
// TODO:             .header(ContentType::Form)
// TODO:             .body(format!("ids={}&place_id={}", c_id, e_id))
// TODO:             .dispatch();
// TODO:         assert_eq!(res.status(), Status::SeeOther);
// TODO:         //TODO: archived comments should be loaded too.
// TODO:         let err = db.shared().unwrap().load_comment(&c_id).err().unwrap();
// TODO:         match err {
// TODO:             RepoError::NotFound => {}
// TODO:             _ => panic!("Expected {}", RepoError::NotFound),
// TODO:         }
// TODO:     }
// TODO: 
// TODO:     #[test]
// TODO:     fn archive_comment_as_scout() {
// TODO:         let (client, db, mut search) = setup();
// TODO:         create_user(&db, "foo", Role::Scout);
// TODO:         login_user(&client, "foo");
// TODO:         let (e_id, _, c_id) = create_place_with_rating(&db, &mut search);
// TODO:         let comment = db.shared().unwrap().load_comment(&c_id).unwrap();
// TODO:         assert!(comment.archived_at.is_none());
// TODO:         let res = client
// TODO:             .post("/comments/actions/archive")
// TODO:             .header(ContentType::Form)
// TODO:             .body(format!("ids={}&place_id={}", c_id, e_id))
// TODO:             .dispatch();
// TODO:         assert_eq!(res.status(), Status::SeeOther);
// TODO:         //TODO: archived comments should be loaded too.
// TODO:         let err = db.shared().unwrap().load_comment(&c_id).err().unwrap();
// TODO:         match err {
// TODO:             RepoError::NotFound => {}
// TODO:             _ => panic!("Expected {}", RepoError::NotFound),
// TODO:         }
// TODO:     }
// TODO: 
// TODO:     #[test]
// TODO:     fn archive_comment_as_guest() {
// TODO:         let (client, db, mut search) = setup();
// TODO:         let (e_id, _, c_id) = create_place_with_rating(&db, &mut search);
// TODO:         let res = client
// TODO:             .post("/comments/actions/archive")
// TODO:             .header(ContentType::Form)
// TODO:             .body(format!("ids={}&place_id={}", c_id, e_id))
// TODO:             .dispatch();
// TODO:         assert_eq!(res.status(), Status::NotFound);
// TODO:         let comment = db.shared().unwrap().load_comment(&c_id).unwrap();
// TODO:         assert!(comment.archived_at.is_none());
// TODO:     }
// TODO: 
// TODO:     #[test]
// TODO:     fn archive_rating_as_guest() {
// TODO:         let (client, db, mut search) = setup();
// TODO:         let (e_id, r_id, _) = create_place_with_rating(&db, &mut search);
// TODO:         let res = client
// TODO:             .post("/ratings/actions/archive")
// TODO:             .header(ContentType::Form)
// TODO:             .body(format!("ids={}&place_id={}", r_id, e_id))
// TODO:             .dispatch();
// TODO:         assert_eq!(res.status(), Status::NotFound);
// TODO:     }
// TODO: }
// TODO: 
// TODO: mod admin {
// TODO:     use super::*;
// TODO: 
// TODO:     #[test]
// TODO:     fn change_user_role() {
// TODO:         let (client, db, _) = setup();
// TODO:         create_user(&db, "admin", Role::Admin);
// TODO:         create_user(&db, "user", Role::User);
// TODO:         let user = get_user(&db, "user");
// TODO:         let admin = get_user(&db, "admin");
// TODO:         assert_eq!(admin.role, Role::Admin);
// TODO:         assert_eq!(user.role, Role::User);
// TODO:         login_user(&client, "admin");
// TODO:         let login_res = client
// TODO:             .post("/change-user-role")
// TODO:             .header(ContentType::Form)
// TODO:             .body("email=user%40example.com&role=2")
// TODO:             .dispatch();
// TODO:         assert_eq!(login_res.status(), Status::SeeOther);
// TODO:         let user = get_user(&db, "user");
// TODO:         assert_eq!(user.role, Role::Scout);
// TODO:     }
// TODO: }
// TODO: 
// TODO: mod pw_reset {
// TODO:     use super::*;
// TODO: 
// TODO:     #[test]
// TODO:     fn reset_password() {
// TODO:         let (client, db, _) = setup();
// TODO:         register_user(&db, "user@example.com", "secret", true);
// TODO: 
// TODO:         // User opens the form to request a new password
// TODO:         let mut res = client.get("/reset-password").dispatch();
// TODO:         assert_eq!(res.status(), Status::Ok);
// TODO:         let body_str = res.body().and_then(|b| b.into_string()).unwrap();
// TODO:         assert!(body_str.contains("<form"));
// TODO:         assert!(body_str.contains("action=\"/users/actions/reset-password-request\""));
// TODO:         assert!(body_str.contains("name=\"email\""));
// TODO:         assert!(body_str.contains("type=\"submit\""));
// TODO: 
// TODO:         // User sends the request
// TODO:         let res = client
// TODO:             .post("/users/actions/reset-password-request")
// TODO:             .header(ContentType::Form)
// TODO:             .body("email=user%40example.com")
// TODO:             .dispatch();
// TODO:         assert_eq!(res.status(), Status::SeeOther);
// TODO:         let h = res
// TODO:             .headers()
// TODO:             .iter()
// TODO:             .find(|h| h.name.as_str() == "Location")
// TODO:             .unwrap();
// TODO:         assert_eq!(h.value, "/reset-password?success=true");
// TODO: 
// TODO:         // User gets a sucess message
// TODO:         let mut res = client.get("/reset-password?success=true").dispatch();
// TODO:         assert_eq!(res.status(), Status::Ok);
// TODO:         let body_str = res.body().and_then(|b| b.into_string()).unwrap();
// TODO:         assert!(body_str.contains("success"));
// TODO: 
// TODO:         // User gets an email with the corresponding token
// TODO:         let token = db
// TODO:             .shared()
// TODO:             .unwrap()
// TODO:             .get_user_token_by_email("user@example.com")
// TODO:             .unwrap()
// TODO:             .email_nonce
// TODO:             .encode_to_string();
// TODO: 
// TODO:         // User opens the link
// TODO:         let mut res = client
// TODO:             .get(format!("/reset-password?token={}", token))
// TODO:             .dispatch();
// TODO:         assert_eq!(res.status(), Status::Ok);
// TODO:         let body_str = res.body().and_then(|b| b.into_string()).unwrap();
// TODO:         assert!(body_str.contains("<form"));
// TODO:         assert!(body_str.contains("action=\"/users/actions/reset-password\""));
// TODO:         assert!(body_str.contains("name=\"new_password\""));
// TODO:         assert!(body_str.contains("name=\"new_password_repeated\""));
// TODO:         assert!(body_str.contains("name=\"token\""));
// TODO:         assert!(body_str.contains("type=\"submit\""));
// TODO: 
// TODO:         // User send the new password to the server
// TODO:         let res = client
// TODO:             .post("/users/actions/reset-password")
// TODO:             .header(ContentType::Form)
// TODO:             .body(format!(
// TODO:                 "new_password=12345678&new_password_repeated=12345678&token={}",
// TODO:                 token
// TODO:             ))
// TODO:             .dispatch();
// TODO:         assert_eq!(res.status(), Status::SeeOther);
// TODO:         let h = res
// TODO:             .headers()
// TODO:             .iter()
// TODO:             .find(|h| h.name.as_str() == "Location")
// TODO:             .unwrap();
// TODO:         assert_eq!(
// TODO:             h.value,
// TODO:             format!("/reset-password?token={}&success=true", token)
// TODO:         );
// TODO:         let mut res = client
// TODO:             .get(format!("/reset-password?token={}&success=true", token))
// TODO:             .dispatch();
// TODO:         assert_eq!(res.status(), Status::Ok);
// TODO:         let body_str = res.body().and_then(|b| b.into_string()).unwrap();
// TODO:         assert!(body_str.contains("success"));
// TODO: 
// TODO:         // User can't login with old password
// TODO:         let res = client
// TODO:             .post("/login")
// TODO:             .header(ContentType::Form)
// TODO:             .body("email=user%40example.com&password=secret")
// TODO:             .dispatch();
// TODO:         assert_eq!(res.status(), Status::SeeOther);
// TODO:         let h = res
// TODO:             .headers()
// TODO:             .iter()
// TODO:             .find(|h| h.name.as_str() == "Location")
// TODO:             .unwrap();
// TODO:         assert_eq!(h.value, "/login");
// TODO: 
// TODO:         // User can login with the new password
// TODO:         let res = client
// TODO:             .post("/login")
// TODO:             .header(ContentType::Form)
// TODO:             .body("email=user%40example.com&password=12345678")
// TODO:             .dispatch();
// TODO:         assert_eq!(res.status(), Status::SeeOther);
// TODO:         let h = res
// TODO:             .headers()
// TODO:             .iter()
// TODO:             .find(|h| h.name.as_str() == "Location")
// TODO:             .unwrap();
// TODO:         assert_eq!(h.value, "/");
// TODO:     }
// TODO: }
