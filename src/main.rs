#![allow(non_snake_case)] // Leptos components use PascalCase

use home::HomePage;
use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;
use login::LoginPage;
use register::RegistrationPage;

mod env;
mod home;
mod login;
mod logout;
mod register;
mod request;

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    view! {
        <Router>
            <Routes fallback=|| view! { <h1>"Not Found"</h1> }>
                <Route path=path!("/") view=HomePage />
                <Route path=path!("/register") view=RegistrationPage />
                <Route path=path!("/login") view=LoginPage />
            </Routes>
        </Router>
    }
}
