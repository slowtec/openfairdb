use crate::{
    view::{
        address_to_html,
        // TODO: leaflet_css_link,
        // TODO: map_scripts
        page::page,
        FlashMessage,
    },
    Mdl, Msg,
};
use ofdb_entities::{comment::*, place::*, rating::*, user::*};
use seed::{prelude::*, *};
use std::collections::HashMap;

type Ratings = Vec<(Rating, Vec<Comment>)>;

pub struct EntryPresenter {
    pub place: Place,
    pub ratings: HashMap<RatingContext, Ratings>,
    pub allow_archiving: bool,
}

impl From<(Place, Vec<(Rating, Vec<Comment>)>, Role)> for EntryPresenter {
    fn from((place, rtngs, role): (Place, Vec<(Rating, Vec<Comment>)>, Role)) -> EntryPresenter {
        let mut p: EntryPresenter = (place, rtngs).into();
        p.allow_archiving = match role {
            Role::Admin | Role::Scout => true,
            _ => false,
        };
        p
    }
}

impl From<(Place, Vec<(Rating, Vec<Comment>)>)> for EntryPresenter {
    fn from((place, rtngs): (Place, Vec<(Rating, Vec<Comment>)>)) -> EntryPresenter {
        let mut ratings: HashMap<RatingContext, Ratings> = HashMap::new();

        for (r, comments) in rtngs {
            if let Some(x) = ratings.get_mut(&r.context) {
                x.push((r, comments));
            } else {
                ratings.insert(r.context, vec![(r, comments)]);
            }
        }
        let allow_archiving = false;
        EntryPresenter {
            place,
            ratings,
            allow_archiving,
        }
    }
}

pub fn entry(email: Option<&str>, e: EntryPresenter) -> Node<Msg> {
    page(
        &format!("{} | OpenFairDB", e.place.title),
        email,
        None,
        None, // TODO: Some(leaflet_css_link()),
        entry_detail(e),
    )
}

fn entry_detail(e: EntryPresenter) -> Node<Msg> {
    let rev = format!("v{}", u64::from(e.place.revision));
    div![
        h3![
            &e.place.title,
            " ",
            span![
                class!["rev"],
                "(",
                if e.allow_archiving {
                    span![
                        a![
                            attrs! {
                                At::Href => format!("/places/{}/history", e.place.id)
                            },
                            rev
                        ],
                        " | ",
                        a![
                            attrs! {At::Href=> format!("/places/{}/review", e.place.id) },
                            "review"
                        ]
                    ]
                } else {
                    span![rev]
                },
                ")"
            ]
        ],
        p![&e.place.description],
        p![table![
            if let Some(ref l) = e.place.links {
                if let Some(ref h) = l.homepage {
                    tr![
                        td!["Homepage"],
                        td![a![
                            attrs! {
                            At::Href=>h.as_str();
                            },
                            h.as_str()
                        ]]
                    ]
                } else {
                    empty!()
                }
            } else {
                empty!()
            },
            if let Some(ref c) = e.place.contact {
                vec![
                    if let Some(ref m) = c.email {
                        tr![
                            td!["eMail"],
                            td![a![
                                attrs! {
                                    At::Href=> format!("mailto:{}",m);
                                },
                                m.as_str()
                            ]]
                        ]
                    } else {
                        empty!()
                    },
                    if let Some(ref t) = c.phone {
                        tr![
                            td!["Phone"],
                            td![a![
                                attrs! {
                                    At::Href=> format!("tel:{}",t);
                                },
                                t
                            ]]
                        ]
                    } else {
                        empty!()
                    },
                ]
            } else {
                vec![empty!()]
            },
            if let Some(ref a) = e.place.location.address {
                if !a.is_empty() {
                    tr![td!["Address"], td![address_to_html(&a)]]
                } else {
                    empty!()
                }
            } else {
                empty!()
            }
        ]],
        p![ul![e.place.tags.iter().map(|t| li![format!("#{}", t)])]],
        h3!["Ratings"],
        e.ratings.iter().map(|(ctx, ratings)| div![
            h4![format!("{:?}", ctx)],
            ul![ratings.iter().map(|(r, comments)| li![rating(
                e.place.id.as_ref(),
                e.allow_archiving,
                &r,
                &comments
            )])]
        ]),
        div![attrs! {
           At::Id=>"map";
           At::Style=> "height:300px;";
        }],
        // TODO: (map_scripts(&[e.place.into()]))
    ]
}

fn rating(place_id: &str, archive: bool, r: &Rating, comments: &[Comment]) -> Node<Msg> {
    div![
        h5![&r.title, " ", span![format!("({})", i8::from(r.value))]],
        if archive {
            form![
                attrs! {
                    At::Action => "/ratings/actions/archive";
                    At::Method => "POST";
                },
                input![attrs! {
                At::Type=>"hidden";At::Name=>"ids"; At::Value=> r.id.as_str();
                }],
                input![attrs! { At::Type=>"hidden";At::Name=>"place_id" At::Value=>place_id;   }],
                input![attrs! { At::Type=>"submit";At::Value=>"archive rating";
                }]
            ]
        } else {
            empty!()
        },
        if let Some(ref src) = r.source {
            p![format!("source: {}", src)]
        } else {
            empty!()
        },
        ul![comments.iter().map(|c| li![
            p![&c.text],
            if archive {
                form![
                    attrs! { At::Action => "/comments/actions/archive"; At::Method => "POST"; },
                    input![attrs! { At::Type=>"hidden";At::Name=>"ids";At::Value=>c.id.as_str();}],
                    input![
                        attrs! { At::Type=>"hidden";At::Name=>"place_id";At::Value=>place_id;   }
                    ],
                    input![attrs! { At::Type=>"submit";At::Value=>"archive comment";            }]
                ]
            } else {
                empty!()
            }
        ])]
    ]
}
