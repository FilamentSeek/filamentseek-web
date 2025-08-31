use gloo_net::http::Method;
use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};

use crate::request::request_json;

const SESSION_KEY: &str = "session_v1";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Session {
    pub uuid: String,
    pub username: String,
    pub email: String,
    pub is_admin: bool,
    pub access_token: String,
    pub refresh_token: String,
}

impl Session {
    pub fn save(&self) -> Result<(), gloo_storage::errors::StorageError> {
        LocalStorage::set(SESSION_KEY, self)
    }

    pub fn load() -> Option<Self> {
        LocalStorage::get(SESSION_KEY).ok()
    }

    pub fn clear() {
        LocalStorage::delete(SESSION_KEY);
    }

    pub fn is_logged_in() -> bool {
        Session::load().is_some()
    }

    pub async fn log_in(access_token: String, refresh_token: String) -> Result<Self, String> {
        #[derive(Serialize, Deserialize)]
        struct UserResponse {
            pub uuid: String,
            pub username: String,
            pub email: String,
            pub is_admin: bool,
        }

        let user_response = match request_json::<(), UserResponse>(
            "users/me",
            crate::request::Auth::Ephemeral {
                access_token: access_token.clone(),
            },
            Method::GET,
            None,
        )
        .await
        {
            Ok(user) => user,
            Err(err) => return Err(format!("{}: {}", err.status, err.message)),
        };

        let session = Session {
            uuid: user_response.uuid,
            username: user_response.username,
            email: user_response.email,
            is_admin: user_response.is_admin,
            access_token,
            refresh_token,
        };

        session.save().map_err(|e| e.to_string())?;
        Ok(session)
    }
}
