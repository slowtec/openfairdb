use crate::{Mdl, Msg, Page};
use ofdb_entities::{address::*, location::*, place::*};
use seed::{prelude::*, *};

pub struct FlashMessage;

impl FlashMessage {
    fn name(&self) -> &str {
        todo!()
    }
    fn msg(&self) -> &str {
        todo!()
    }
}

pub fn view(mdl: &Mdl) -> Node<Msg> {
    match mdl.page {
        Page::Home => index(None),
        Page::Login => login::login(None, "/reset-password"),
        Page::Events => event::events(None, &vec![]),
        Page::NotFound => index(None),
    }
}

// TODO: use num_traits::ToPrimitive;
// TODO: const MAP_JS_URL: &str = "/map.js";

mod dashboard;
mod entry;
mod event;
mod login;
mod page;
mod password;
mod place;
mod register;

// TODO: pub use dashboard::*;
// TODO: pub use entry::*;
// TODO: pub use event::*;
// TODO: pub use login::*;
// TODO: use page::*;
// TODO: pub use password::*;
// TODO: pub use place::*;
// TODO: pub use register::*;

fn index(email: Option<&str>) -> Node<Msg> {
    page::page(
        "OpenFairDB Search",
        email,
        None,
        None,
        div![
            class!["search"],
            h1!["OpenFairDB Search"],
            global_search_form(None)
        ],
    )
}

fn global_search_form(search_term: Option<&str>) -> Node<Msg> {
    div![
        class!["search-form"],
        form![
            attrs! {
                At::Action=> "search";
                At::Method=> "GET";
            },
            input![attrs! {
                                        At::Type=>"text";
                                        At::Name=>"q";
                                        At::Value => search_term.unwrap_or("");
                                        At::Size=> 50;
                                        At::MaxLength=>200;
                                        At::Placeholder=> "search term, empty = all";
            }],
            br![],
            input![
                class!["btn"],
                attrs! {
                    At::Type=>"submit";
                    At::Value=>"search";
                }
            ]
        ]
    ]
}

// TODO: pub fn search_results(email: Option<&str>, search_term: &str, entries: &[IndexedPlace]) -> Markup {
// TODO:     page(
// TODO:         "OpenFairDB Search Results",
// TODO:         email,
// TODO:         None,
// TODO:         None,
// TODO:         html! {
// TODO:             div class="search" {
// TODO:                 h1 {"OpenFairDB Search"}
// TODO:                 (global_search_form(Some(search_term)))
// TODO:             }
// TODO:             div class="results" {
// TODO:                 @if entries.is_empty(){
// TODO:                     p{ "We are so sorry but we could not find any entries
// TODO:                         that are related to your search term "
// TODO:                             em {
// TODO:                                 (format!("'{}'", search_term))
// TODO:                             }
// TODO:                     }
// TODO:                 } @else {
// TODO:                     p {
// TODO:                         ul class="result-list" {
// TODO:                             @for e in entries {
// TODO:                                 li{
// TODO:                                     (entry_result(e))
// TODO:                                 }
// TODO:                             }
// TODO:                         }
// TODO:                     }
// TODO:                 }
// TODO:             }
// TODO:         },
// TODO:     )
// TODO: }
// TODO:
// TODO: fn entry_result(e: &IndexedPlace) -> Markup {
// TODO:     html! {
// TODO:         h3 {
// TODO:             a href=(format!("entries/{}",e.id)) {(e.title)}
// TODO:         }
// TODO:         p {(e.description)}
// TODO:     }
// TODO: }

fn address_to_html(addr: &Address) -> Vec<Node<Msg>> {
    vec![
    // TODO: @if let Some(ref s) = addr.street {
    // TODO:     (s) br;
    // TODO: }
    // TODO: @if let Some(ref z) = addr.zip {
    // TODO:     (z)
    // TODO: }
    // TODO: @if let Some(ref z) = addr.city {
    // TODO:     (z) br;
    // TODO: }
    // TODO: @if let Some(ref c) = addr.country {
    // TODO:     (c)
    // TODO: }
    ]
}

struct MapPin {
    lat: f64,
    lng: f64,
}

impl MapPin {
    fn to_js_object_string(&self) -> String {
        format!("{{lat:{},lng:{}}}", self.lat, self.lng)
    }
}

impl From<Place> for MapPin {
    fn from(e: Place) -> Self {
        e.location.into()
    }
}

impl From<&Place> for MapPin {
    fn from(e: &Place) -> Self {
        (&e.location).into()
    }
}

impl From<Location> for MapPin {
    fn from(l: Location) -> Self {
        (&l).into()
    }
}

impl From<&Location> for MapPin {
    fn from(l: &Location) -> Self {
        MapPin {
            lat: l.pos.lat().to_deg(),
            lng: l.pos.lng().to_deg(),
        }
    }
}

// TODO: fn map_scripts(pins: &[MapPin]) -> Markup {
// TODO:     let (center, zoom) = match pins.len() {
// TODO:         1 => ((pins[0].lat, pins[0].lng), 13.0),
// TODO:         _ => {
// TODO:             //TODO: calculate center & zoom
// TODO:             ((48.720, 9.152), 6.0)
// TODO:         }
// TODO:     };
// TODO:
// TODO:     let center = format!("[{},{}]", center.0, center.1);
// TODO:
// TODO:     let pins: String = pins
// TODO:         .iter()
// TODO:         .map(MapPin::to_js_object_string)
// TODO:         .collect::<Vec<_>>()
// TODO:         .join(",");
// TODO:
// TODO:     html! {
// TODO:       script{
// TODO:         (format!("window.OFDB_MAP_PINS=[{}];window.OFDB_MAP_ZOOM={};OFDB_MAP_CENTER={};",
// TODO:                  pins,
// TODO:                  zoom,
// TODO:                  center))
// TODO:       }
// TODO:       script
// TODO:         src=(LEAFLET_JS_URL)
// TODO:         integrity=(LEAFLET_JS_SHA512)
// TODO:         crossorigin="anonymous" {}
// TODO:       script src=(MAP_JS_URL){}
// TODO:     }
// TODO: }
// TODO:
// TODO: pub fn user_search_result(admin_email: &str, users: &[User]) -> Markup {
// TODO:     page(
// TODO:         "Users",
// TODO:         Some(admin_email),
// TODO:         None,
// TODO:         None,
// TODO:         html! {
// TODO:             main {
// TODO:                 h3 { "Users" }
// TODO:                 (search_users_form())
// TODO:                 @if users.is_empty() {
// TODO:                     "No users were found :("
// TODO:                 } @else {
// TODO:                     table {
// TODO:                         thead {
// TODO:                             tr {
// TODO:                               th { "Username"        }
// TODO:                               th { "eMail"           }
// TODO:                               th { "eMail confirmed" }
// TODO:                               th { "Role"            }
// TODO:                               th { "Modify role"            }
// TODO:                             }
// TODO:                         }
// TODO:                         tbody {
// TODO:                             @for u in users {
// TODO:                                 tr {
// TODO:                                     td { (u.email) }
// TODO:                                     td { (if u.email_confirmed{"yes"}else{"no"}) }
// TODO:                                     td { (format!("{:?}",u.role)) }
// TODO:                                     td {
// TODO:                                         @if u.email != admin_email {
// TODO:                                             form action="change-user-role" method="POST" {
// TODO:                                                 select name = "role" required? {
// TODO:                                                     option value="-1" {"-- please select --"}
// TODO:                                                     option value=(Role::Guest.to_u8().unwrap()) { "Guest" }
// TODO:                                                     option value=(Role::User.to_u8().unwrap())  { "User" }
// TODO:                                                     option value=(Role::Scout.to_u8().unwrap()) { "Scout" }
// TODO:                                                 }
// TODO:                                                 input type="hidden" name="email" value=(u.email);
// TODO:                                                 input type="submit" value="change";
// TODO:                                             }
// TODO:                                         }
// TODO:                                     }
// TODO:                                 }
// TODO:                             }
// TODO:                         }
// TODO:                     }
// TODO:                 }
// TODO:             }
// TODO:         },
// TODO:     )
// TODO: }

pub fn search_users_form() -> Node<Msg> {
    form![
        attrs! {
            At::Action => "search-users";
            At::Method => "GET";
        },
        input![attrs! {
            At::Type=>"email";
            At::Name=>"email";
            At::Placeholder=>"email address";
        },],
        br![],
        input![attrs! {
            At::Type=>"submit";
            At::Value =>"search";
        },]
    ]
}
