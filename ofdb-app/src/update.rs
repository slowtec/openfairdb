use crate::{Mdl, Msg, Page};
use seed::{prelude::*, *};

pub fn update(msg: Msg, mdl: &mut Mdl, _: &mut impl Orders<Msg>) {
    match msg {
        Msg::UrlChanged(subs::UrlChanged(url)) => {
            log!(url);
            mdl.page = Page::init(url);
        }
        _ => todo!(),
    }
}
