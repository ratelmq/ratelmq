use crate::broker::manager::Manager;
use crate::mqtt::listener::MqttListener;
use futures::future::join_all;
use log::info;
use tokio::signal;
use tokio::sync::broadcast;
use tokio::sync::mpsc;

pub async fn run() {
    info!("Initializing RatelMQ...");

    let (ctrl_c_tx, ctrl_c_rx) = broadcast::channel(5);

    let (client_tx, client_rx) = mpsc::channel(32);

    let manager = Manager::new(client_rx, ctrl_c_rx);
    let manager_future = tokio::spawn(manager.run());

    let listener = MqttListener::bind("0.0.0.0:1883", client_tx.clone(), ctrl_c_tx.subscribe())
        .await
        .unwrap();

    let mut listeners = Vec::new();
    listeners.push(tokio::spawn(listener.start_accepting()));

    info!("Initialized RatelMQ");

    signal::ctrl_c().await.unwrap();

    info!("Stopping RatelMQ");
    ctrl_c_tx.send(()).unwrap();

    join_all(listeners).await;

    manager_future.await.unwrap();

    info!("RatelMQ stopped");
}
