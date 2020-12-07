use crate::{
    view::{page::page, FlashMessage},
    Mdl, Msg,
};
use ofdb_entities::{place::*, review::*, revision::*, user::*};
use seed::{prelude::*, *};

pub fn place_history(user: &User, h: &PlaceHistory) -> Node<Msg> {
    page(
        "Place History",
        Some(&user.email),
        None,
        None,
        div![
            class!["revisions"],
            table![
                thead![tr![
                    th!["Revision"],
                    th!["Log"],
                    th!["Title"],
                    th!["Description"],
                    th!["Position"],
                    th!["Street"],
                    th!["ZIP"],
                    th!["City"],
                    th!["Country"],
                    th!["E-Mail"],
                    th!["Phone"],
                    th!["Homepage"],
                    th!["Image"],
                    th!["Image Link"],
                    th!["Tags"],
                ]],
                tbody![
                    // TODO: @for (r,logs) in &h.revisions {
                    // TODO:     tr {
                    // TODO:         td{ (u64::from(r.revision)) }
                    // TODO:         td{
                    // TODO:             ul class="log" {
                    // TODO:                 @for l in logs {
                    // TODO:                     li { (review_status_log(r.revision, &l)) }
                    // TODO:                 }
                    // TODO:             }
                    // TODO:         }
                    // TODO:
                    // TODO:         td{ (r.title) }
                    // TODO:         td{ (r.description) }
                    // TODO:
                    // TODO:         td{
                    // TODO:             (format!("{:.2}/{:0.2}",
                    // TODO:                 r.location.pos.lat().to_deg(),
                    // TODO:                 r.location.pos.lng().to_deg()
                    // TODO:             ))
                    // TODO:         }
                    // TODO:         @if let Some(a) = &r.location.address {
                    // TODO:             td{ @if let Some(x) = &a.street  { (x) } }
                    // TODO:             td{ @if let Some(x) = &a.zip     { (x) } }
                    // TODO:             td{ @if let Some(x) = &a.city    { (x) } }
                    // TODO:             td{ @if let Some(x) = &a.country { (x) } }
                    // TODO:         } @else {
                    // TODO:             td{ }
                    // TODO:             td{ }
                    // TODO:             td{ }
                    // TODO:             td{ }
                    // TODO:         }
                    // TODO:         @if let Some(c) = &r.contact {
                    // TODO:             td{ @if let Some(x) = &c.email  { a href=(format!("mailto:{}",x)) { (x) } } }
                    // TODO:             td{ @if let Some(x) = &c.phone  { (x) } }
                    // TODO:         } @else {
                    // TODO:             td {}
                    // TODO:             td {}
                    // TODO:         }
                    // TODO:         @if let Some(l) = &r.links {
                    // TODO:             td{ @if let Some(x) = &l.homepage   { a href=(x) { (x) } } }
                    // TODO:             td{ @if let Some(x) = &l.image      { img src=(x); } }
                    // TODO:             td{ @if let Some(x) = &l.image_href { a href=(x) { (x) } } }
                    // TODO:         } @else {
                    // TODO:             td {}
                    // TODO:             td {}
                    // TODO:             td {}
                    // TODO:         }
                    // TODO:         td{
                    // TODO:             ul class="tags" {
                    // TODO:                 @for t in &r.tags {
                    // TODO:                     li { (t) }
                    // TODO:                 }
                    // TODO:             }
                    // TODO:         }
                    // TODO:     }
                    // TODO: }
                ]
            ]
        ],
    )
}

fn review_status_log(place_rev: Revision, l: &ReviewStatusLog) -> Node<Msg> {
    use ReviewStatus as S;
    let status = match l.status {
        S::Rejected => "Rejected",
        S::Archived => "Archived",
        S::Created => {
            if place_rev.is_initial() {
                "Created"
            } else {
                "Modified"
            }
        }
        S::Confirmed => "Confirmed",
    };
    div![
        span![class!["status"], status],
        " at ",
        // TODO: (l.activity.activity.at)
        " by ",
        if let Some(email) = &l.activity.activity.by {
            email
        } else {
            "anonymous visitor"
        },
        if let Some(c) = &l.activity.comment {
            span![class!["comment"], format!(" \"{}\"", c)]
        } else {
            empty!()
        },
        if let Some(c) = &l.activity.context {
            span![class!["context"], format!(" ({})", c)]
        } else {
            empty!()
        }
    ]
}

pub fn place_review(email: &str, place: &Place, status: ReviewStatus) -> Node<Msg> {
    use ReviewStatus as S;
    let options = [
        (S::Rejected, "reject"),
        (S::Archived, "archive"),
        (S::Confirmed, "confirm"),
    ];
    page(
        "Place Review",
        Some(&email),
        None,
        None,
        div![
            class!["review"],
            h2![format!("Add Review for \"{}\"", place.title)],
            form![
                attrs! {
                   At::Action=> format!("/places/{}/review", place.id);
                   At::Method=>"POST";
                },
                fieldset![
                    label![
                        "Comment:",
                        br![],
                        input![attrs! {
                            At::Required => true.as_at_value(),
                            At::Name=>"comment";
                            At::Placeholder => "Comment";
                        }]
                    ],
                    br![],
                    label![
                        "Action:",
                        br![],
                        select![
                            attrs! {
                                At::Name=>"status";
                            },
                            options.iter().map(|(o, label)| option![
                                attrs! {
                                    At::Value => i16::from(*o);
                                    At::Disabled => (*o==status).as_at_value();
                                },
                                label
                            ])
                        ]
                    ]
                ],
                input![attrs! {
                    At::Type=>"submit";
                    At::Value=>"change";
                }]
            ]
        ],
    )
}
