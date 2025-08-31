use gloo_net::http::Method;
use leptos::{prelude::*, reactive::spawn_local};
use serde::Serialize;

use crate::{
    request::{Auth, TokenResponse, request_json},
    session::Session,
};

#[component]
pub fn LoginPage() -> impl IntoView {
    view! {
        <div class="container">
            <h1>"Login"</h1>
            <LoginForm />
            <p>
                <a href="/">"Homepage"</a>
            </p>
        </div>
    }
}

#[component]
pub fn LoginForm() -> impl IntoView {
    if Session::is_logged_in() {
        web_sys::window()
            .expect("No global window")
            .location()
            .set_href("/")
            .expect("Failed to redirect to home page");
        return ().into_any();
    }

    let (username, set_username) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (message, set_message) = signal(Option::<String>::None);
    let (loading, set_loading) = signal(false);

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_loading.set(true);

        #[derive(Serialize)]
        struct LoginBody {
            username: String,
            password: String,
            grant_type: String,
        }

        let body = LoginBody {
            username: username.get(),
            password: password.get(),
            grant_type: "password".to_string(),
        };

        spawn_local(async move {
            match request_json::<LoginBody, TokenResponse>(
                "auth/token",
                Auth::Unauthorized,
                Method::POST,
                Some(&body),
            )
            .await
            {
                Ok(response) => {
                    if let Err(e) =
                        Session::log_in(response.access_token, response.refresh_token).await
                    {
                        set_message.set(Some(e));
                        set_loading.set(false);
                        return;
                    }

                    web_sys::window()
                        .expect("No global window")
                        .location()
                        .set_href("/")
                        .expect("Failed to redirect to login page");

                    return;
                }
                Err(err) => {
                    set_message.set(Some(err.message));
                }
            }

            set_loading.set(false);
        });
    };

    view! {
        <div>
            <form class="card" on:submit=on_submit>
                <label>
                    <span>"Username"</span>
                    <input
                        type="username"
                        prop:value=move || username.get()
                        on:input=move |e| set_username.set(event_target_value(&e))
                        required
                    />
                </label>

                <label>
                    <span>"Password"</span>
                    <input
                        type="password"
                        prop:value=move || password.get()
                        on:input=move |e| set_password.set(event_target_value(&e))
                        required
                    />
                </label>

                <button type="submit" disabled=move || loading.get()>
                    {move || if loading.get() { "Please wait…" } else { "Sign in" }}
                </button>

                <Show when=move || message.get().is_some()>
                    <p class="err">{move || message.get().unwrap_or_default()}</p>
                </Show>

                <p style="margin-top:.6rem;">
                    <a href="/register">"Register"</a>
                </p>
            </form>
        </div>
    }
    .into_any()
}
