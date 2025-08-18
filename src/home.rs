use gloo_storage::Storage;
use leptos::prelude::*;

use crate::logout::LogoutButton;

#[component]
pub fn HomePage() -> impl IntoView {
    let username: Option<String> = gloo_storage::LocalStorage::get::<String>("username").ok();

    view! {
        <div class="container">
            <h1>"Homepage"</h1>
            <div class="card">
                <p>"Homepage goes here"</p>
                {
                    if let Some(u) = username {
                        view! { <p>{format!("Logged in as {u}")}</p><br /><LogoutButton /> }.into_any()
                    } else {
                        view! { <div><a href="/login">"Login"</a></div> }.into_any()
                    }
                }
            </div>
        </div>
    }
}
