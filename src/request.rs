use gloo_net::http::{Method, Request, RequestBuilder};
use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::env::API_BASE_URL;

#[derive(PartialEq)]
pub enum Auth {
    Authorized,
    Unauthorized,
}

impl Auth {
    pub fn is_authorized(&self) -> bool {
        self == &Auth::Authorized
    }
}

pub struct ErrorResponse {
    pub message: String,
    pub status: u16,
}

#[derive(Deserialize)]
struct GenericError {
    error: String,
}

#[derive(Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
}

pub async fn request_json<B, R>(
    path: &str,
    auth: Auth,
    method: Method,
    body: Option<&B>,
) -> Result<R, ErrorResponse>
where
    R: DeserializeOwned,
    B: Serialize,
{
    async fn send_once<B, R>(
        path: &str,
        authorized: bool,
        method: Method,
        body: Option<&B>,
    ) -> Result<Result<R, ErrorResponse>, ErrorResponse>
    where
        R: DeserializeOwned,
        B: Serialize,
    {
        let mut req = RequestBuilder::new(&format!("{API_BASE_URL}/{path}"))
            .method(method)
            .header("Content-Type", "application/json");

        if authorized {
            if let Ok(token) = LocalStorage::get::<String>("auth_token") {
                req = req.header("Authorization", &format!("Bearer {token}"));
            }
        }

        let req = req.json(&body).map_err(|e| ErrorResponse {
            message: format!("Bad JSON: {e}"),
            status: 0,
        })?;

        let resp = req.send().await.map_err(|e| ErrorResponse {
            message: format!("Network error: {e}"),
            status: 0,
        })?;

        if resp.ok() {
            let out = resp.json::<R>().await.map_err(|e| ErrorResponse {
                message: format!("Bad JSON: {e}"),
                status: resp.status(),
            })?;
            Ok(Ok(out))
        } else {
            let status = resp.status();

            let err = match resp.json::<GenericError>().await {
                Ok(e) => ErrorResponse {
                    message: e.error,
                    status,
                },
                Err(e) => ErrorResponse {
                    message: format!("Bad JSON: {e}"),
                    status,
                },
            };
            Ok(Err(err))
        }
    }

    match send_once::<B, R>(path, auth.is_authorized(), method.clone(), body).await? {
        Ok(ok) => Ok(ok),
        Err(err) if err.status == 401 && auth.is_authorized() => {
            if refresh_access_token().await.is_err() {
                return Err(err);
            }

            match send_once::<B, R>(path, true, method, body).await? {
                Ok(r) => Ok(r),
                Err(_) => Err(err),
            }
        }
        Err(err) => Err(err),
    }
}

async fn refresh_access_token() -> Result<(), ErrorResponse> {
    let username = LocalStorage::get::<String>("username").map_err(|_| ErrorResponse {
        message: "No username in storage".to_string(),
        status: 0,
    })?;

    let refresh_token =
        LocalStorage::get::<String>("refresh_token").map_err(|_| ErrorResponse {
            message: "No refresh token in storage:".to_string(),
            status: 0,
        })?;

    #[derive(Serialize)]
    struct RefreshBody {
        grant_type: String,
        username: String,
        refresh_token: String,
    }

    let body = RefreshBody {
        grant_type: "refresh_token".to_string(),
        username,
        refresh_token,
    };

    let response = Request::post(&format!("{API_BASE_URL}/auth/token"))
        .header("Content-Type", "application/json")
        .json(&body)
        .map_err(|e| ErrorResponse {
            message: format!("Bad JSON: {e}"),
            status: 0,
        })?
        .send()
        .await
        .map_err(|e| ErrorResponse {
            message: format!("Network error: {e}"),
            status: 0,
        })?;

    if !response.ok() {
        let status = response.status();

        let message = match response.json::<GenericError>().await {
            Ok(e) => e.error,
            Err(e) => format!("Bad JSON: {e}"),
        };

        return Err(ErrorResponse { message, status });
    }

    let response = response
        .json::<TokenResponse>()
        .await
        .map_err(|e| ErrorResponse {
            message: format!("Bad JSON: {e}"),
            status: 0,
        })?;

    LocalStorage::set("access_token", response.access_token).map_err(|e| ErrorResponse {
        message: format!("Failed to save access_token: {e}"),
        status: 0,
    })?;

    if let Some(rt) = response.refresh_token {
        LocalStorage::set("refresh_token", rt).map_err(|e| ErrorResponse {
            message: format!("Failed to save refresh_token: {e}"),
            status: 0,
        })?;
    }

    Ok(())
}
