mod config;
mod alert;
mod spot;
mod keepalive;

use std::env;
use log::{LevelFilter};
use env_logger::Builder;

fn main() {
    Builder::new().filter_level(LevelFilter::Info).init();
    let port: u16 = match env::var("SERVER_PORT") {
        Ok(s) => s.parse::<u16>().unwrap_or(9080),
        Err(_) => 9080
    };
}

