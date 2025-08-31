use gloo_net::http::{Method, Request, RequestBuilder};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::{env::API_BASE_URL, session::Session};

#[derive(PartialEq)]
pub enum Auth {
    Authorized,
    Unauthorized,
    Ephemeral { access_token: String },
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
    pub refresh_token: String,
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
        auth: &Auth,
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

        match auth {
            Auth::Authorized => {
                let session = Session::load().ok_or(ErrorResponse {
                    message: "No session in storage".to_string(),
                    status: 0,
                })?;

                req = req.header("Authorization", &format!("Bearer {}", session.access_token));
            }
            Auth::Ephemeral { access_token } => {
                req = req.header("Authorization", &format!("Bearer {}", access_token));
            }
            Auth::Unauthorized => (),
        }

        let req = if let Some(body) = body {
            req.json(&body).map_err(|e| ErrorResponse {
                message: format!("Bad JSON: {e}"),
                status: 0,
            })?
        } else {
            req.build().map_err(|e| ErrorResponse {
                message: format!("Request build error: {e}"),
                status: 0,
            })?
        };

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

    match send_once::<B, R>(path, &auth, method.clone(), body).await? {
        Ok(ok) => Ok(ok),
        Err(err) if err.status == 401 && auth == Auth::Authorized => {
            if refresh_access_token().await.is_err() {
                crate::console_warn(format!(
                    "Token refresh failed (Logging out): ({}) {}",
                    err.status, err.message
                ));

                Session::clear();

                web_sys::window()
                    .expect("No global window")
                    .location()
                    .set_href("/login")
                    .expect("Failed to redirect to login page");

                return Err(err);
            } else {
                crate::console_log("Access token refreshed");
            }

            match send_once::<B, R>(path, &auth, method, body).await? {
                Ok(r) => Ok(r),
                Err(_) => Err(err),
            }
        }
        Err(err) => Err(err),
    }
}

async fn refresh_access_token() -> Result<(), ErrorResponse> {
    let mut session = Session::load().ok_or(ErrorResponse {
        message: "No session in storage".to_string(),
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
        username: session.username,
        refresh_token: session.refresh_token,
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

    session.access_token = response.access_token;
    session.refresh_token = response.refresh_token;
    Ok(())
}
