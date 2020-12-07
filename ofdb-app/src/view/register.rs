use crate::{
    view::{page::page, FlashMessage},
    Mdl, Msg,
};
use seed::{prelude::*, *};

pub fn register(flash: Option<FlashMessage>) -> Node<Msg> {
    page(
        "Register",
        None,
        flash,
        None,
        form![
            class!["register"],
            attrs! {
                At::Action => "register";
                At::Method => "POST";
            },
            fieldset![
                label![
                    "eMail:",
                    br![],
                    input![attrs! {
                        At::Type => "email";
                        At::Name => "email";
                        At::Placeholder => "eMail address";
                    }],
                ],
                br![],
                label![
                    "Password:",
                    br![],
                    input![attrs! {
                        At::Type=>"password";
                        At::Name=>"password";
                        At::Placeholder=>"Password";
                    }],
                ],
                br![],
                input![attrs! {
                        At::Type =>"submit";
                        At::Value=>"register";
                }]
            ]
        ],
    )
}
