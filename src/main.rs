#![allow(non_snake_case)] // Leptos components use PascalCase

use home::HomePage;
use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;
use login::LoginPage;
use register::RegistrationPage;

use crate::admin::AdminPage;

mod admin;
mod env;
mod home;
mod login;
mod logout;
mod product;
mod product_search;
mod register;
mod request;
mod session;

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
                <Route path=path!("/admin") view=AdminPage />
            </Routes>
        </Router>
    }
}

pub fn console_log<T: Into<web_sys::wasm_bindgen::JsValue>>(msg: T) {
    web_sys::console::log_1(&msg.into());
}

pub fn console_error<T: Into<web_sys::wasm_bindgen::JsValue>>(msg: T) {
    web_sys::console::error_1(&msg.into());
}

pub fn console_warn<T: Into<web_sys::wasm_bindgen::JsValue>>(msg: T) {
    web_sys::console::warn_1(&msg.into());
}
