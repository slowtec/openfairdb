use crate::{view::page::page, Mdl, Msg, Page};
use ofdb_entities::{event::Event, user::User};
use seed::{prelude::*, *};

pub fn event(user: Option<User>, ev: Event) -> Node<Msg> {
    page(
        &ev.title,
        user.as_ref().map(|u| &*u.email),
        None,
        None,
        div![
            class!["details", "event"],
            // TODO:                 div class="entity-type" { "Event"  }
            // TODO:                 h2{ (ev.title) }
            // TODO:                 p class="time" {
            // TODO:                     (ev.start.format("%d.%m.%Y %H:%M"))
            // TODO:                         @if let Some(end) = ev.end{
            // TODO:                             " - "
            // TODO:                             (end.format("%d.%m.%Y %H:%M"))
            // TODO:                         }
            // TODO:                 }
            // TODO:                 p class="description" { (ev.description.unwrap_or_default()) }
            // TODO:
            // TODO:                 @if let Some(ref location) = ev.location{
            // TODO:                         h4{ "Ort" }
            // TODO:                         @if let Some(ref addr) = location.address {
            // TODO:                             @if !addr.is_empty(){
            // TODO:                                 @if let Some(ref s) = addr.street {
            // TODO:                                     (s) br;
            // TODO:                                 }
            // TODO:                                 @if let Some(ref z) = addr.zip {
            // TODO:                                     (z) " "
            // TODO:                                 }
            // TODO:                                 @if let Some(ref c) = addr.city {
            // TODO:                                     (c) br;
            // TODO:                                 }
            // TODO:                                 @if let Some(ref c) = addr.country {
            // TODO:                                     (c)
            // TODO:                                 }
            // TODO:                             }
            // TODO:                         }
            // TODO:                         h4 { "Koordinaten" }
            // TODO:                         p{
            // TODO:                             (format!("{:.2} / {:.2}",location.pos.lat().to_deg(), location.pos.lng().to_deg()))
            // TODO:                         }
            // TODO:                 }
            // TODO:                 @if let Some(org) = ev.organizer {
            // TODO:                     h4{"Veranstalter"}
            // TODO:                     p{(org)}
            // TODO:                 }
            // TODO:                 @if let Some(contact) = ev.contact{
            // TODO:                     @if !contact.is_empty(){
            // TODO:                         h4{ "Kontakt" }
            // TODO:                         @if let Some(email) = contact.email{
            // TODO:                             (email)
            // TODO:                             br;
            // TODO:                         }
            // TODO:                         @if let Some(phone) = contact.phone{
            // TODO:                             (phone)
            // TODO:                         }
            // TODO:                     }
            // TODO:                 }
            // TODO:                 @if let Some(url) = ev.homepage{
            // TODO:                         h4{ "Webseite" }
            // TODO:                         p{
            // TODO:                             a href=(url) { (url) }
            // TODO:                         }
            // TODO:                 }
            // TODO:                 @if let Some(reg) = ev.registration{
            // TODO:                     h4{ "Anmeldung" }
            // TODO:                     p {
            // TODO:                         @match reg{
            // TODO:                             RegistrationType::Email => "eMail" ,
            // TODO:                             RegistrationType::Phone => "Telefon",
            // TODO:                             RegistrationType::Homepage => "Webseite",
            // TODO:                         }
            // TODO:                     }
            // TODO:                 }
            // TODO:                 p {
            // TODO:                     h4{ "Tags" }
            // TODO:                     ul class="tags" {
            // TODO:                         @for t in ev.tags{
            // TODO:                             li {(format!("#{}",t))}
            // TODO:                         }
            // TODO:                     }
            // TODO:                 }
            // TODO:                 @if let Some(user) = &user {
            // TODO:                     @match user.role {
            // TODO:                         Role::Admin | Role::Scout => {
            // TODO:                             form action=(format!("/events/{}/archive", ev.id)) method="POST" {
            // TODO:                                 input type="submit" value="archive event";
            // TODO:                             }
            // TODO:                         }
            // TODO:                         _=> {}
            // TODO:                     }
            // TODO:                 }
            // TODO:             }
            // TODO:             @if let Some(l) = ev.location {
            // TODO:                 div id="map" style="height:100vh;" { }
            // TODO:                 (map_scripts(&[l.into()]))
            // TODO:             }
        ],
    )
}

pub fn events(email: Option<&str>, events: &[Event]) -> Node<Msg> {
    // TODO: let locations: Vec<_> = events
    // TODO:     .iter()
    // TODO:     .filter_map(|e| e.location.as_ref())
    // TODO:     .map(|l| l.into())
    // TODO:     .collect();
    let event_lis = events.iter().map(|e| {
        li! [
        // TODO: a href=(format!("/events/{}", e.id)) {
        // TODO:     div {
        // TODO:         h4 {
        // TODO:             span class="title" { (e.title) }
        // TODO:             " "
        // TODO:             span class="date" {
        // TODO:                 (e.start.format("%d.%m.%y"))
        // TODO:             }
        // TODO:         }
        // TODO:         p {
        // TODO:             @if let Some(ref l) = e.location {
        // TODO:                 @if let Some(ref a) = l.address {
        // TODO:                     @if let Some(ref city) = a.city {
        // TODO:                         span class="city" { (city) }
        // TODO:                         br;
        // TODO:                     }
        // TODO:                 }
        // TODO:             }
        // TODO:             @if let Some(ref o) = e.organizer {
        // TODO:                 span class="organizer" { (o) }
        // TODO:             }
        // TODO:         }
        // TODO:     }
        // TODO: }
        ]
    });

    page(
        "List of Events",
        email,
        None,
        None,
        div![
            class!["events"],
            h3!["Events"],
            if events.is_empty() {
                p![
                    class!["no-results"],
                    "Es konnten keine Events gefunden werden."
                ]
            } else {
                ul![class!["event-list"], event_lis,]
            }
        ], // TODO: div id="map" style="height:100vh;" { }
           // TODO: (map_scripts(&locations))
    )
}
