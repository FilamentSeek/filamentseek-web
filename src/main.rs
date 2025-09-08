#![allow(non_snake_case)] // Leptos components use PascalCase

use home::HomePage;
use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;
use login::LoginPage;
use register::RegistrationPage;

#[cfg(not(target_arch = "wasm32"))]
use rocket::fs::{FileServer, NamedFile, relative};
#[cfg(not(target_arch = "wasm32"))]
use rocket::{get, routes};
#[cfg(not(target_arch = "wasm32"))]
use std::path::{Path, PathBuf};

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

#[cfg(target_arch = "wasm32")]
fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() {
    if let Err(e) = rocket::build()
        .mount("/", FileServer::from(relative!("dist")).rank(10))
        .mount("/", routes![spa_fallback])
        .launch()
        .await
    {
        panic!("Rocket failed: {}", e);
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[get("/<__path..>", rank = 20)]
async fn spa_fallback(__path: PathBuf) -> Option<NamedFile> {
    let index = Path::new(relative!("dist")).join("index.html");
    NamedFile::open(index).await.ok()
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
