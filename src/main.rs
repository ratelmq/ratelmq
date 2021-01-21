use crate::application::Application;

mod application;
mod broker;
mod mqtt;

#[tokio::main]
async fn main() {
    Application::init();

    let app = Box::new(Application::new());
    app.run().await;
}
