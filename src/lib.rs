extern crate core;

pub mod broker;
pub mod config;
pub mod mqtt;
pub mod settings;

mod application;

pub async fn run(config_filename: &str) {
    application::run(config_filename).await;
}
