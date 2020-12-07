// TODO: use super::super::guards::*;
// TODO: use super::view;
// TODO: use crate::{
// TODO:     core::{prelude::*, usecases},
// TODO:     ports::web::sqlite::Connections,
// TODO: };
// TODO: use maud::Markup;
// TODO: use rocket::{
// TODO:     self,
// TODO:     http::{Cookie, Cookies},
// TODO:     request::{FlashMessage, Form},
// TODO:     response::{Flash, Redirect},
// TODO: };
// TODO: 
// TODO: #[derive(FromForm)]
// TODO: pub struct LoginCredentials {
// TODO:     pub email: String,
// TODO:     password: String,
// TODO: }
// TODO: 
// TODO: impl<'a> LoginCredentials {
// TODO:     pub fn as_login(&'a self) -> usecases::Credentials<'a> {
// TODO:         let LoginCredentials {
// TODO:             ref email,
// TODO:             ref password,
// TODO:         } = self;
// TODO:         usecases::Credentials { email, password }
// TODO:     }
// TODO: }
// TODO: 
// TODO: #[get("/login")]
// TODO: pub fn get_login_user(_account: Account) -> Redirect {
// TODO:     Redirect::to(uri!(super::get_index))
// TODO: }
// TODO: 
// TODO: #[get("/login", rank = 2)]
// TODO: pub fn get_login(flash: Option<FlashMessage>) -> Markup {
// TODO:     view::login(flash, "/reset-password")
// TODO: }
// TODO: 
// TODO: #[post("/login", data = "<credentials>")]
// TODO: pub fn post_login(
// TODO:     db: Connections,
// TODO:     credentials: Form<LoginCredentials>,
// TODO:     mut cookies: Cookies,
// TODO: ) -> std::result::Result<Redirect, Flash<Redirect>> {
// TODO:     match db.shared() {
// TODO:         Err(_) => Err(Flash::error(
// TODO:             Redirect::to(uri!(get_login)),
// TODO:             "We are so sorry! An internal server error has occurred. Please try again later.",
// TODO:         )),
// TODO:         Ok(db) => {
// TODO:             let credentials = credentials.into_inner();
// TODO:             match usecases::login_with_email(&*db, &credentials.as_login()) {
// TODO:                 Err(err) => {
// TODO:                     let msg = match err {
// TODO:                         Error::Parameter(ParameterError::EmailNotConfirmed) => {
// TODO:                             "You have to confirm your email address first."
// TODO:                         }
// TODO:                         Error::Parameter(ParameterError::Credentials) => {
// TODO:                             "Invalid email or password."
// TODO:                         }
// TODO:                         _ => panic!(),
// TODO:                     };
// TODO:                     Err(Flash::error(Redirect::to(uri!(get_login)), msg))
// TODO:                 }
// TODO:                 Ok(_) => {
// TODO:                     cookies.add_private(Cookie::new(COOKIE_EMAIL_KEY, credentials.email));
// TODO:                     Ok(Redirect::to(uri!(super::get_index)))
// TODO:                 }
// TODO:             }
// TODO:         }
// TODO:     }
// TODO: }
// TODO: 
// TODO: #[post("/logout")]
// TODO: pub fn post_logout(mut cookies: Cookies) -> Flash<Redirect> {
// TODO:     cookies.remove_private(Cookie::named(COOKIE_EMAIL_KEY));
// TODO:     Flash::success(
// TODO:         Redirect::to(uri!(super::get_index)),
// TODO:         "Sie haben sich erfolgreich abgemeldet.",
// TODO:     )
// TODO: }
// TODO: 
// TODO: #[cfg(test)]
// TODO: pub mod tests {
// TODO:     use super::*;
// TODO:     use crate::ports::web::{
// TODO:         self,
// TODO:         tests::{prelude::*, register_user},
// TODO:     };
// TODO:     use rocket::http::Status as HttpStatus;
// TODO: 
// TODO:     fn setup() -> (Client, Connections) {
// TODO:         let (client, db, _) = web::tests::setup(vec![("/", super::super::routes())]);
// TODO:         (client, db)
// TODO:     }
// TODO: 
// TODO:     fn user_id_cookie(response: &Response) -> Option<Cookie<'static>> {
// TODO:         let cookie = response
// TODO:             .headers()
// TODO:             .get("Set-Cookie")
// TODO:             .find(|v| v.starts_with(COOKIE_EMAIL_KEY))
// TODO:             .and_then(|val| Cookie::parse_encoded(val).ok());
// TODO:         cookie.map(|c| c.into_owned())
// TODO:     }
// TODO: 
// TODO:     #[test]
// TODO:     fn get_login() {
// TODO:         let (client, _) = setup();
// TODO:         let mut res = client.get("/login").dispatch();
// TODO:         assert_eq!(res.status(), HttpStatus::Ok);
// TODO:         let body_str = res.body().and_then(|b| b.into_string()).unwrap();
// TODO:         assert!(body_str.contains("action=\"login\""));
// TODO:         assert!(user_id_cookie(&res).is_none());
// TODO:     }
// TODO: 
// TODO:     #[test]
// TODO:     fn post_login_fails() {
// TODO:         let (client, pool) = setup();
// TODO:         register_user(&pool, "foo@bar.com", "bazbaz", true);
// TODO:         let res = client
// TODO:             .post("/login")
// TODO:             .header(ContentType::Form)
// TODO:             .body("email=foo%40bar.com&password=invalid")
// TODO:             .dispatch();
// TODO:         assert_eq!(res.status(), HttpStatus::SeeOther);
// TODO:         for h in res.headers().iter() {
// TODO:             match h.name.as_str() {
// TODO:                 "Location" => assert_eq!(h.value, "/login"),
// TODO:                 "Content-Length" => assert_eq!(h.value.parse::<i32>().unwrap(), 0),
// TODO:                 _ => { /* let these through */ }
// TODO:             }
// TODO:         }
// TODO:     }
// TODO: 
// TODO:     #[test]
// TODO:     fn post_login_sucess() {
// TODO:         let (client, pool) = setup();
// TODO:         register_user(&pool, "foo@bar.com", "baz baz", true);
// TODO:         let res = client
// TODO:             .post("/login")
// TODO:             .header(ContentType::Form)
// TODO:             .body("email=foo%40bar.com&password=baz baz")
// TODO:             .dispatch();
// TODO:         assert_eq!(res.status(), HttpStatus::SeeOther);
// TODO:         assert!(user_id_cookie(&res).is_some());
// TODO:         //TODO: extract private cookie value to assert v == "foo@bar.com"
// TODO:         for h in res.headers().iter() {
// TODO:             match h.name.as_str() {
// TODO:                 "Location" => assert_eq!(h.value, "/"),
// TODO:                 "Content-Length" => assert_eq!(h.value.parse::<i32>().unwrap(), 0),
// TODO:                 _ => { /* let these through */ }
// TODO:             }
// TODO:         }
// TODO:     }
// TODO: }
