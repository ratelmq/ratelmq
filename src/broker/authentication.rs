use crate::broker::authentication::AuthenticationError::{EncryptionError, InvalidPassword};
use crate::broker::authentication::FileIdentityManagerError::InvalidEntry;
use std::collections::HashMap;
use std::io::Error;
use AuthenticationError::UserNotFound;

use argon2::{
    password_hash::{PasswordHash, PasswordVerifier},
    Argon2,
};

#[derive(Debug)]
pub enum AuthenticationError {
    // UserNotFound(UserNotFound),
    // InvalidPassword(InvalidPassword),
    UserNotFound,
    InvalidPassword,
    EncryptionError(argon2::password_hash::Error),
}

// #[derive(Debug, Clone)]
// pub struct UserNotFound {
//     // pub username: &'static str,
// }
//
// #[derive(Debug, Clone)]
// pub struct InvalidPassword;

pub trait IdentityProvider {
    fn authenticate(&self, username: &str, password: &str) -> Result<(), AuthenticationError>;
}

#[derive(Debug)]
pub enum FileIdentityManagerError {
    FileError(std::io::Error),
    InvalidEntry,
    InvalidPassword,
}

impl From<std::io::Error> for FileIdentityManagerError {
    fn from(error: Error) -> Self {
        FileIdentityManagerError::FileError(error)
    }
}

pub struct FileIdentityManager {
    passwords_by_username: HashMap<String, String>,
}

impl FileIdentityManager {
    pub fn new(filename: &str) -> Result<FileIdentityManager, FileIdentityManagerError> {
        let credentials = std::fs::read_to_string(filename)?;

        let mut passwords_by_username = HashMap::with_capacity(credentials.lines().count());

        for line in credentials.lines() {
            let credential = line.split_once(':').ok_or(InvalidEntry)?;
            let username = credential.0;
            let password = credential.1;

            passwords_by_username.insert(username.to_owned(), password.to_owned());
        }

        let manager = FileIdentityManager {
            passwords_by_username,
        };

        Ok(manager)
    }
}

impl IdentityProvider for FileIdentityManager {
    fn authenticate(&self, username: &str, password: &str) -> Result<(), AuthenticationError> {
        let password_hash = self
            .passwords_by_username
            .get(username)
            .ok_or(UserNotFound)?;

        let parsed_hash = PasswordHash::new(password_hash).map_err(|e| EncryptionError(e))?;

        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|e| InvalidPassword)
    }
}
