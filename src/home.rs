use leptos::prelude::*;

use crate::{logout::LogoutButton, product_search::ProductSearch, session::Session};

#[component]
pub fn HomePage() -> impl IntoView {
    let username: Option<String> = Session::load().map(|s| s.username);

    view! {
        <div class="container">
            <h1>"FilamentSeek"</h1>
            <div class="card">
                <ProductSearch />
                {
                    if let Some(u) = username {
                        view! { <p>{format!("Logged in as {u}")}</p><br /><LogoutButton /> }.into_any()
                    } else {
                        //view! { <div><a href="/login">"Login"</a></div> }.into_any()
                        ().into_any()
                    }
                }
            </div>
        </div>
    }
}
