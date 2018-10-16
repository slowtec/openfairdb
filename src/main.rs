// Copyright (c) 2015 - 2018 Markus Kohlhase <mail@markus-kohlhase.de>

#![feature(plugin, custom_derive, test)]
#![plugin(rocket_codegen)]
#![recursion_limit = "256"]

extern crate chrono;
extern crate clap;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
extern crate dotenv;
extern crate env_logger;
#[macro_use]
extern crate failure;
extern crate fast_chemail;
#[macro_use]
extern crate lazy_static;
extern crate lettre;
extern crate lettre_email;
#[macro_use]
extern crate log;
extern crate pwhash;
#[macro_use]
extern crate quick_error;
extern crate regex;
extern crate rocket;
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;
extern crate csv;
extern crate serde_json;
#[cfg(test)]
extern crate test;
extern crate toml;
extern crate url;
extern crate uuid;

mod adapters;
mod core;
mod infrastructure;
mod ports;

fn main() {
    env_logger::init();
    ports::cli::run();
}
