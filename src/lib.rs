pub mod broker;
pub mod mqtt;

mod application;

pub async fn run() {
    application::run().await;
}
