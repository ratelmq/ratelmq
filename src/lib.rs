use crate::application::Application;

pub mod broker;
pub mod mqtt;

mod application;

pub async fn run() {
    let app = Box::new(Application::new());
    app.run().await;
}
