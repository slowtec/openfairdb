use seed::prelude::*;

mod update;
mod view;

#[derive(Debug)]
pub struct Mdl {
    page: Page,
}

impl Default for Mdl {
    fn default() -> Self {
        Mdl { page: Page::Home }
    }
}

#[derive(Debug)]
enum Page {
    Home,
    Login,
    Events,
    NotFound,
}

impl Page {
    fn init(mut url: Url) -> Self {
        match url.next_path_part() {
            None => Self::Home,
            Some("login") => Page::Login,
            Some("events") => Page::Events,
            _ => Self::NotFound,
        }
    }
}

#[derive(Clone)]
pub enum Msg {
    UrlChanged(subs::UrlChanged),
}

fn init(_: Url, orders: &mut impl Orders<Msg>) -> Mdl {
    orders.subscribe(Msg::UrlChanged);
    Mdl::default()
}

#[wasm_bindgen(start)]
pub fn start() {
    App::start("app", init, update::update, view::view);
}
