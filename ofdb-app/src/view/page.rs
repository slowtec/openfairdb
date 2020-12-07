use crate::{view::FlashMessage, Mdl, Msg};
use seed::{prelude::*, *};

pub fn page(
    title: &str,
    email: Option<&str>,
    flash: Option<FlashMessage>,
    h: Option<Node<Msg>>,
    content: Node<Msg>,
) -> Node<Msg> {
    // TODO: Change title tag of page dynamically
    // TODO: Inject header content
    div![flash_msg(flash), header(email), content]
}

fn flash_msg(flash: Option<FlashMessage>) -> Node<Msg> {
    if let Some(msg) = flash {
        div![class!["flash", msg.name()], msg.msg()]
    } else {
        empty!()
    }
}

fn header(email: Option<&str>) -> Node<Msg> {
    header![
        if let Some(email) = email {
            div![
                class!["msg"],
                "Your are logged in as ",
                span![class!["email"], email]
            ]
        } else {
            empty!()
        },
        nav![
            a![attrs! {At::Href=> "/"; }, "places"],
            a![attrs! {At::Href=> "/events"}, "events"],
            if email.is_some() {
                vec![
                    a![attrs! {At::Href=>"/dashboard"; }, "dashboard"],
                    form![
                        class!["logout"],
                        attrs! {
                            At::Action=> "/logout";
                            At::Method=> "POST";
                        },
                        input![attrs! {
                            At::Type=>"submit";
                            At::Value=>"logout";
                        }]
                    ],
                ]
            } else {
                vec![
                    a![attrs! { At::Href=> "/login"; }, "login"],
                    a![attrs! { At::Href=> "/register"; }, "register"],
                ]
            }
        ]
    ]
}
