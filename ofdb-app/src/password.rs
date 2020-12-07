// TODO: use super::view;
// TODO: use crate::{
// TODO:     core::prelude::*,
// TODO:     infrastructure::flows::prelude::*,
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
// TODO: #[get("/reset-password?<token>&<success>")]
// TODO: pub fn get_reset_password(
// TODO:     flash: Option<FlashMessage>,
// TODO:     token: Option<&RawStr>,
// TODO:     success: Option<&RawStr>,
// TODO: ) -> Markup {
// TODO:     let success = success
// TODO:         .map(|raw| raw.as_str())
// TODO:         .map(|s| s == "true" || s == "1");
// TODO: 
// TODO:     if let Some(token) = token {
// TODO:         if let Some(true) = success {
// TODO:             view::reset_password_ack(flash)
// TODO:         } else {
// TODO:             view::reset_password(flash, "/users/actions/reset-password", token.as_str())
// TODO:         }
// TODO:     } else if let Some(true) = success {
// TODO:         view::reset_password_request_ack(flash)
// TODO:     } else {
// TODO:         view::reset_password_request(flash, "/users/actions/reset-password-request")
// TODO:     }
// TODO: }
// TODO: 
// TODO: #[derive(FromForm)]
// TODO: pub struct ResetPasswordRequest {
// TODO:     email: String,
// TODO: }
// TODO: 
// TODO: #[post("/users/actions/reset-password-request", data = "<data>")]
// TODO: pub fn post_reset_password_request(
// TODO:     db: Connections,
// TODO:     notify: Notify,
// TODO:     data: Form<ResetPasswordRequest>,
// TODO: ) -> std::result::Result<Redirect, Flash<Redirect>> {
// TODO:     let ResetPasswordRequest { email } = data.into_inner();
// TODO:     match reset_password_request(&db, &*notify, &email) {
// TODO:         Err(_) => Err(Flash::error(
// TODO:             Redirect::to(uri!(get_reset_password:token = _, success = _)),
// TODO:             "Failed to request a password reset.",
// TODO:         )),
// TODO:         Ok(_) => Ok(Redirect::to(
// TODO:             uri!(get_reset_password: token=_, success = "true"),
// TODO:         )),
// TODO:     }
// TODO: }
// TODO: 
// TODO: #[derive(FromForm)]
// TODO: pub struct ResetPassword {
// TODO:     token: String,
// TODO:     new_password: String,
// TODO:     new_password_repeated: String,
// TODO: }
// TODO: 
// TODO: #[post("/users/actions/reset-password", data = "<data>")]
// TODO: pub fn post_reset_password(
// TODO:     db: Connections,
// TODO:     data: Form<ResetPassword>,
// TODO: ) -> std::result::Result<Redirect, Flash<Redirect>> {
// TODO:     let req = data.into_inner();
// TODO: 
// TODO:     if req.new_password != req.new_password_repeated {
// TODO:         return Err(Flash::error(
// TODO:             Redirect::to(uri!(get_reset_password: token = req.token, success =_)),
// TODO:             "Your passwords do not match.",
// TODO:         ));
// TODO:     }
// TODO:     match req.new_password.parse::<Password>() {
// TODO:         Err(_) => Err(Flash::error(
// TODO:             Redirect::to(uri!(get_reset_password: token=req.token, success=_)),
// TODO:             "Your new password is not allowed.",
// TODO:         )),
// TODO:         Ok(new_password) => match EmailNonce::decode_from_str(&req.token) {
// TODO:             Err(_) => Err(Flash::error(
// TODO:                 Redirect::to(uri!(get_reset_password: token = req.token, success = _)),
// TODO:                 "Resetting your password is not possible (invalid token).",
// TODO:             )),
// TODO:             Ok(email_nonce) => {
// TODO:                 match reset_password_with_email_nonce(&db, email_nonce, new_password) {
// TODO:                     Err(_) => Err(Flash::error(
// TODO:                         Redirect::to(uri!(
// TODO:                                 get_reset_password: token = req.token, success = _)),
// TODO:                         "Failed to request a password reset.",
// TODO:                     )),
// TODO:                     Ok(_) => Ok(Redirect::to(uri!(
// TODO:                         get_reset_password: token = req.token,
// TODO:                         success = "true"
// TODO:                     ))),
// TODO:                 }
// TODO:             }
// TODO:         },
// TODO:     }
// TODO: }
