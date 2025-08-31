use gloo_net::http::Method;
use leptos::web_sys;
use leptos::{prelude::*, reactive::spawn_local};
use serde::Serialize;

use crate::request::{Auth, TokenResponse, request_json};
use crate::session::Session;

#[component]
pub fn RegistrationPage() -> impl IntoView {
    view! {
        <div class="container">
            <h1>"Register"</h1>
            <div class="card">
                <RegistrationForm />
                <p>
                    <a href="/login">"Already have an account?"</a>
                </p>
            </div>
        </div>
    }
}

#[component]
pub fn RegistrationForm() -> impl IntoView {
    if Session::is_logged_in() {
        web_sys::window()
            .expect("No global window")
            .location()
            .set_href("/")
            .expect("Failed to redirect to home page");
        return ().into_any();
    }

    let (username, set_username) = signal(String::new());
    let (email, set_email) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (message, set_message) = signal(Option::<String>::None);
    let (loading, set_loading) = signal(false);

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_loading.set(true);

        #[derive(Serialize)]
        struct RegistrationBody {
            username: String,
            email: String,
            password: String,
        }

        let body = RegistrationBody {
            username: username.get(),
            email: email.get(),
            password: password.get(),
        };

        spawn_local(async move {
            match request_json::<RegistrationBody, TokenResponse>(
                "register_user",
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
            <form on:submit=on_submit>
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
                    <span>"Email"</span>
                    <input
                        type="email"
                        prop:value=move || email.get()
                        on:input=move |e| set_email.set(event_target_value(&e))
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
                    {move || if loading.get() { "Please wait…" } else { "Register" }}
                </button>

                <Show when=move || message.get().is_some()>
                    <p class="err">{move || message.get().unwrap_or_default()}</p>
                </Show>
            </form>
        </div>
    }
    .into_any()
}
