use crate::application::Application;

mod application;
mod broker;
mod mqtt;

#[tokio::main]
async fn main() {
    Application::init();

    let mut app = Box::new(Application::new());
    app.run().await;
}
