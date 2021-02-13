use crate::broker::manager::Manager;
use crate::mqtt::listener::MqttListener;
use futures::future::join_all;
use log::info;
use tokio::sync::mpsc;

pub async fn run() {
    info!("Initializing RatelMQ...");

    let (tx, rx) = mpsc::channel(32);

    let manager = Manager::new(rx);
    let manager_future = tokio::spawn(manager.run());

    let listener = MqttListener::bind("127.0.0.1:1883", tx.clone())
        .await
        .unwrap();

    let mut listeners = Vec::new();
    listeners.push(tokio::spawn(listener.start_accepting()));

    info!("Initialized RatelMQ");

    join_all(listeners).await;

    manager_future.await.unwrap();

    info!("RatelMQ stopped");
}
