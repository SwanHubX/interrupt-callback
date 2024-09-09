mod config;
use log::{LevelFilter};
use env_logger::Builder;

fn main() {
    Builder::new().filter_level(LevelFilter::Info).init()
}
