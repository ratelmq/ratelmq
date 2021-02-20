use crate::broker::session::Session;
use crate::mqtt::packets::ClientId;
use std::collections::HashMap;

pub trait SessionRepository {
    fn insert(&mut self, session: Session);
    fn exists(&self, client_id: &ClientId) -> bool;
    fn get(&self, client_id: &ClientId) -> Option<&Session>;
    fn delete(&mut self, client_id: &ClientId) -> Option<Session>;

    fn count(&self) -> usize;
}

pub struct InMemorySessionRepository {
    sessions: HashMap<ClientId, Session>,
}

impl InMemorySessionRepository {
    pub fn new(sessions: HashMap<ClientId, Session>) -> Self {
        InMemorySessionRepository { sessions }
    }
}

impl SessionRepository for InMemorySessionRepository {
    fn insert(&mut self, session: Session) {
        self.sessions.insert(session.client_id().clone(), session);
    }

    fn exists(&self, client_id: &ClientId) -> bool {
        self.sessions.contains_key(client_id)
    }

    fn get(&self, client_id: &ClientId) -> Option<&Session> {
        self.sessions.get(client_id)
    }

    fn delete(&mut self, client_id: &ClientId) -> Option<Session> {
        self.sessions.remove(client_id)
    }

    fn count(&self) -> usize {
        self.sessions.len()
    }
}

impl Default for InMemorySessionRepository {
    fn default() -> Self {
        InMemorySessionRepository {
            sessions: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};
    use tokio::sync::mpsc;

    fn create_session() -> Session {
        let (tx, rx) = mpsc::channel(32);
        Session::new(
            "client-1".to_string(),
            IpAddr::V4(Ipv4Addr::LOCALHOST),
            false,
            tx,
        )
    }

    #[test]
    fn test_in_memory_exists_true() {
        let mut sessions = HashMap::new();

        let session = create_session();
        let client_id = session.client_id().clone();
        sessions.insert(client_id.clone(), session);

        let repository = InMemorySessionRepository::new(sessions);

        assert_eq!(repository.exists(&client_id), true);
    }

    #[test]
    fn test_in_memory_exists_false() {
        let mut sessions = HashMap::new();
        let client_id = "client-1".to_string();

        let repository = InMemorySessionRepository::new(sessions);

        assert_eq!(repository.exists(&client_id), false);
    }
}
