// use log::{info, trace};
// use tokio::select;
// use tokio::sync::broadcast::Receiver;
// use tokio::time::Duration;
//
// use crate::broker::messaging::MessagingService;
// use crate::mqtt::events::ServerEvent;
//
// pub struct KeepAliveChecker {
//     ctrl_c_rx: Receiver<()>,
//     messaging: MessagingServiceSync,
// }
//
// impl KeepAliveChecker {
//     pub fn new(ctrl_c_rx: Receiver<()>, messaging: MessagingServiceSync) -> Self {
//         KeepAliveChecker {
//             ctrl_c_rx,
//             messaging,
//         }
//     }
//
//     pub async fn run(mut self) {
//         trace!("Starting keep alive checker process");
//
//         loop {
//             select! {
//                 _ = self.ctrl_c_rx.recv() => {
//                     trace!("Stopping keep alive checker");
//                     break;
//                 }
//                  _ = tokio::time::sleep(Duration::from_secs(1)) => {
//                     trace!("Checking for expired keep alive");
//                     self.check().await;
//                  }
//             }
//         }
//
//         trace!("Keep alive checker process stopped");
//     }
//
//     async fn check(&mut self) {
//         let mut senders = Vec::new();
//         let mut expired_client_ids = Vec::new();
//
//         {
//             let mut messaging = self.messaging.lock();
//
//             let expired_sessions = messaging.session_get_keep_alive_expired();
//             for expired_session in expired_sessions {
//                 senders.push(expired_session.sender().clone());
//                 expired_client_ids.push(expired_session.client_id().clone());
//             }
//         }
//
//         if senders.len() > 0 {
//             info!(
//                 "Found {} sessions with expired keep alive, client ids: {:?}",
//                 &senders.len(),
//                 expired_client_ids
//             );
//         } else {
//             trace!("Found no sessions with expired keep alive");
//         }
//
//         for sender in senders {
//             sender.send(ServerEvent::Disconnect).await;
//         }
//     }
// }
