// TODO: use super::login::LoginCredentials;
// TODO: use super::view;
// TODO: use crate::{
// TODO:     core::{prelude::*, usecases},
// TODO:     ports::web::{notify::*, sqlite::Connections},
// TODO: };
// TODO: use maud::Markup;
// TODO: use rocket::{
// TODO:     self,
// TODO:     http::RawStr,
// TODO:     request::{FlashMessage, Form},
// TODO:     response::{Flash, Redirect},
// TODO: };
// TODO: 
// TODO: #[get("/register")]
// TODO: pub fn get_register(flash: Option<FlashMessage>) -> Markup {
// TODO:     view::register(flash)
// TODO: }
// TODO: 
// TODO: #[post("/register", data = "<credentials>")]
// TODO: pub fn post_register(
// TODO:     db: Connections,
// TODO:     notify: Notify,
// TODO:     credentials: Form<LoginCredentials>,
// TODO: ) -> std::result::Result<Flash<Redirect>, Flash<Redirect>> {
// TODO:     match db.exclusive() {
// TODO:         Err(_) => Err(Flash::error(
// TODO:             Redirect::to(uri!(get_register)),
// TODO:             "We are so sorry! An internal server error has occurred. Please try again later.",
// TODO:         )),
// TODO:         //TODO: move into flow layer
// TODO:         Ok(mut db) => {
// TODO:             let credentials = credentials.into_inner();
// TODO:             match usecases::register_with_email(&mut *db, &credentials.as_login()) {
// TODO:                 Err(err) => {
// TODO:                     let msg = match err {
// TODO:                         Error::Parameter(ParameterError::UserExists) => {
// TODO:                             "A user with your email address already exists."
// TODO:                         }
// TODO:                         Error::Parameter(ParameterError::Credentials) => {
// TODO:                             "Invalid email or password."
// TODO:                         }
// TODO:                         _ => "We are so sorry, something went wrong :(",
// TODO:                     };
// TODO:                     Err(Flash::error(Redirect::to(uri!(get_register)), msg))
// TODO:                 }
// TODO:                 Ok(()) => {
// TODO:                     if let Ok(user) = db.get_user_by_email(&credentials.email) {
// TODO:                         debug_assert_eq!(user.email, credentials.email);
// TODO:                         notify.user_registered_ofdb(&user);
// TODO: 
// TODO:                         let msg = "Registered sucessfully. Please confirm your email address.";
// TODO:                         return Ok(Flash::success(
// TODO:                             Redirect::to(uri!(super::login::get_login)),
// TODO:                             msg,
// TODO:                         ));
// TODO:                     }
// TODO:                     Err(Flash::error(
// TODO:                         Redirect::to(uri!(get_register)),
// TODO:                         "We are so sorry, something went wrong :(",
// TODO:                     ))
// TODO:                 }
// TODO:             }
// TODO:         }
// TODO:     }
// TODO: }
// TODO: 
// TODO: #[get("/register/confirm/<token>")]
// TODO: pub fn get_email_confirmation(
// TODO:     db: Connections,
// TODO:     token: &RawStr,
// TODO: ) -> std::result::Result<Flash<Redirect>, Flash<Redirect>> {
// TODO:     match db.exclusive() {
// TODO:         Err(_) => Err(Flash::error(
// TODO:             Redirect::to(uri!(get_email_confirmation: token)),
// TODO:             "We are so sorry! An internal server error has occurred. Please try again later.",
// TODO:         )),
// TODO:         Ok(db) => match usecases::confirm_email_address(&*db, token.as_str()) {
// TODO:             Ok(_) => Ok(Flash::success(
// TODO:                 Redirect::to(uri!(super::login::get_login)),
// TODO:                 "Your email address is now confirmed :)",
// TODO:             )),
// TODO:             Err(_) => Err(Flash::error(
// TODO:                 Redirect::to(uri!(get_email_confirmation: token)),
// TODO:                 "We are sorry but seems to be something wrong.",
// TODO:             )),
// TODO:         },
// TODO:     }
// TODO: }
