use crate::{
    view::{page::page, FlashMessage},
    Mdl, Msg,
};
use seed::{prelude::*, *};

pub fn login(flash: Option<FlashMessage>, reset_pw_link: &str) -> Node<Msg> {
    page(
        "Login",
        None,
        flash,
        None,
        form![
            class!["login"],
            attrs! {
                At::Action=> "login";
                At::Method=> "POST" ;
            },
            fieldset![
                label![
                    "eMail:",
                    br![],
                    input![attrs! {
                     At::Type=>"email";
                     At::Name=>"email";
                     At::Placeholder=>"eMail address";
                    }]
                ],
                br![],
                label![
                    "Password:",
                    br![],
                    input![attrs! {
                        At::Type=>"password";
                        At::Name=>"password";
                        At::Placeholder=>"Password";
                    }]
                ],
                br![],
                input![attrs! {
                    At::Type=>"submit";
                    At::Value=>"login";
                }],
                p![
                    "Did you forget your password? Don't worry you can ",
                    a![attrs! {At::Href=> reset_pw_link; }, "reset your password"],
                    " :-)"
                ]
            ]
        ],
    )
}
