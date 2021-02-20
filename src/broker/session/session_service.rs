use crate::broker::session::session_repository::SessionRepository;
use crate::broker::session::Session;
use crate::mqtt::packets::ClientId;

#[derive(Default)]
pub struct SessionService<T>
where
    T: SessionRepository,
{
    repository: T,
}

impl<T> SessionService<T>
where
    T: SessionRepository,
{
    pub fn new(repository: T) -> Self {
        SessionService { repository }
    }

    pub fn exists(&self, client_id: &ClientId) -> bool {
        self.repository.exists(client_id)
    }

    pub fn insert(&mut self, session: Session) {
        self.repository.insert(session)
    }

    pub fn get(&self, client_id: &ClientId) -> Option<&Session> {
        self.repository.get(client_id)
    }

    pub fn delete(&mut self, client_id: &ClientId) -> Option<Session> {
        self.repository.delete(client_id)
    }

    pub fn count(&self) -> usize {
        self.repository.count()
    }
}
