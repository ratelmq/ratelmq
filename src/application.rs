use crate::broker::authentication::FileIdentityManager;
use crate::broker::manager::Manager;
use crate::config::build_info::BUILD_INFO;
use crate::mqtt::listener::MqttListener;
use crate::settings::Settings;
use futures::future::join_all;
use log::{debug, info};
use tokio::signal;
use tokio::sync::broadcast;
use tokio::sync::mpsc;

pub async fn run(config_filename: &str) {
    info!(
        "Initializing RatelMQ v{} ({})...",
        BUILD_INFO.version,
        &BUILD_INFO.commit_hash[..10]
    );

    debug!("Using configuration file {}", config_filename);
    let settings = Settings::new(config_filename).unwrap();
    debug!("Effective settings: {:?}", &settings);

    let (ctrl_c_tx, ctrl_c_rx) = broadcast::channel(5);

    let (client_tx, client_rx) = mpsc::channel(32);

    let manager = Manager::new(client_rx, ctrl_c_rx, &settings);
    let manager_future = tokio::spawn(manager.run());

    let mut listeners = Vec::new();

    for bind_address in settings.mqtt.listeners_tcp {
        let listener = MqttListener::bind(
            bind_address.as_str(),
            client_tx.clone(),
            ctrl_c_tx.subscribe(),
        )
        .await
        .unwrap();

        listeners.push(tokio::spawn(listener.start_accepting()));
    }

    info!("Initialized RatelMQ");

    signal::ctrl_c().await.unwrap();

    info!("Stopping RatelMQ");
    ctrl_c_tx.send(()).unwrap();

    join_all(listeners).await;

    manager_future.await.unwrap();

    info!("RatelMQ stopped");
}
