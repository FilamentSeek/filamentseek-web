use gloo_storage::LocalStorage;
use gloo_storage::Storage;
use leptos::IntoView;
use leptos::component;
use leptos::ev::MouseEvent;
use leptos::prelude::*;
use leptos::view;

#[component]
pub fn LogoutButton() -> impl IntoView {
    let on_click = move |ev: MouseEvent| {
        ev.prevent_default();
        LocalStorage::delete("username");
        LocalStorage::delete("access_token");
        LocalStorage::delete("refresh_token");

        web_sys::window()
            .expect("No global window")
            .location()
            .set_href("/")
            .expect("Failed to redirect to home page");
    };

    view! {
        <button on:click=on_click>
            "Logout"
        </button>
    }
}
