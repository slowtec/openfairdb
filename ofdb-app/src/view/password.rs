use crate::{
    view::{page::page, FlashMessage},
    Mdl, Msg,
};
use seed::{prelude::*, *};

pub fn reset_password_request(flash: Option<FlashMessage>, action: &str) -> Node<Msg> {
    page(
        "Request password reset",
        None,
        flash,
        None,
        div![
            h2!["Password reset"],
            p!["Please enter your email address to reset your password."],
            form![
                class!["reset-pw-req"],
                attrs! {
                At::Action=>action;
                At::Method=>"POST";
                },
                fieldset![
                    label![
                        "eMail:",
                        br![],
                        input![attrs! {
                        At::Type=>"text";
                        At::Name=>"email";
                        At::Placeholder=>"eMail address";
                        }]
                    ],
                    br![],
                    input![attrs! {
                        At::Type=>"submit";
                        At::Value=>"reset";
                    }]
                ]
            ]
        ],
    )
}

pub fn reset_password_request_ack(flash: Option<FlashMessage>) -> Node<Msg> {
    page(
        "Request password reset",
        None,
        flash,
        None,
        div![
            h2!["Password reset"],
            p![
                "Your request to reset your password was successfully send.",
                br![],
                "Please look into your email inbox to continue."
            ]
        ],
    )
}

pub fn reset_password(flash: Option<FlashMessage>, action: &str, token: &str) -> Node<Msg> {
    page(
        "New password",
        None,
        flash,
        None,
        form![
            class!["reset-pw"],
            attrs! {
                At::Action=> action;
                At::Method=> "POST";
            },
            fieldset![
                label![
                    "New password:",
                    br![],
                    input![attrs! {
                        At::Type=>"password";
                        At::Name=> "new_password";
                        At::Placeholder=>"new password";
                    }]
                ],
                br![],
                label![
                    "New password (repeated):",
                    br![],
                    input![attrs! {
                        At::Type =>"password";
                        At::Name =>"new_password_repeated";
                        At::Placeholder=>"repeat new password";
                    }]
                ],
                br![],
                input![attrs! {
                    At::Type=>"hidden";
                    At::Name=>"token";
                    At::Value =>token;
                }],
                input![attrs! {
                    At::Type=>"submit";
                    At::Value =>"save";
                }]
            ]
        ],
    )
}

pub fn reset_password_ack(flash: Option<FlashMessage>) -> Node<Msg> {
    page(
        "New password",
        None,
        flash,
        None,
        div![
            h2!["New password"],
            p!["Your password changed successfully."],
            p![a![
                attrs! { At::Href => "/login"; },
                "Login with your new password."
            ]]
        ],
    )
}
