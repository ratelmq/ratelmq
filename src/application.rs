use crate::broker::authentication::FileIdentityManager;
use crate::broker::client_packet_handler::ClientPacketHandler;
// use crate::broker::keepalive_checker::KeepAliveChecker;
use crate::broker::messaging::MessagingService;
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

    // let messaging_service = Arc::new(Mutex::new(MessagingService::new()));
    let messaging_service = MessagingService::new();
    let (messaging_tx, mut messaging_rx) = mpsc::channel(32);

    let manager = ClientPacketHandler::new(
        client_rx,
        ctrl_c_rx,
        &settings,
        messaging_tx,
        // Arc::clone(&messaging_service),
    );
    let manager_future = tokio::spawn(manager.run());

    let messaging_service_future = tokio::spawn(messaging_service.run(messaging_rx));

    // let keep_alive_checker =
    //     KeepAliveChecker::new(ctrl_c_tx.subscribe(), Arc::clone(&messaging_service));
    // let keep_alive_checker_future = tokio::spawn(keep_alive_checker.run());

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

    info!("Successfully initialized RatelMQ, ready to accept connections");

    signal::ctrl_c().await.unwrap();

    info!("Stopping RatelMQ...");
    ctrl_c_tx.send(()).unwrap();

    join_all(listeners).await;

    messaging_service_future.await.unwrap();
    // keep_alive_checker_future.await.unwrap();
    manager_future.await.unwrap();

    info!("RatelMQ stopped");
}
