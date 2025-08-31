use gloo_net::http::Method;
use leptos::{prelude::*, reactive::spawn_local};
use strum::IntoEnumIterator;

use crate::{
    product::{
        Celsius, Cents, FilamentDiameter, FilamentMaterial, Grams, Product, TemperatureSpec,
    },
    request::{Auth, request_json},
    session::Session,
};

#[component]
pub fn AdminPage() -> impl IntoView {
    let mut redirect = true;

    if let Some(session) = Session::load()
        && session.is_admin
    {
        redirect = false;
    }

    if redirect {
        web_sys::window()
            .expect("No global window")
            .location()
            .set_href("/login")
            .expect("Failed to redirect to login page");

        return ().into_any();
    }

    view! {
        <div class="container">
            <h1>"Admin"</h1>
            <ProductEditor product_id=None />
        </div>
    }
    .into_any()
}

#[derive(Clone, Debug)]
enum ResultMessage {
    Success(String),
    Error(String),
}

#[component]
pub fn ProductEditor(product_id: Option<String>) -> impl IntoView {
    let (uuid, set_uuid) = signal::<String>(product_id.unwrap_or_default());
    let (name, set_name) = signal::<String>(String::new());
    let (url, set_url) = signal::<String>(String::new());
    let (material, set_material) = signal::<FilamentMaterial>(FilamentMaterial::Unspecified);
    let (diameter, set_diameter) = signal::<FilamentDiameter>(FilamentDiameter::D175);
    let (weight, set_weight) = signal::<Grams>(Grams(0));
    let (nozzle_temp, set_nozzle_temp) = signal::<Option<TemperatureSpec>>(None);
    let (bed_temp, set_bed_temp) = signal::<Option<TemperatureSpec>>(None);
    let (price_dollars_string, set_price_dollars_string) = signal::<String>(String::new());
    let (result_message, set_result_message) = signal::<Option<ResultMessage>>(None);

    let params = leptos_router::hooks::use_query_map();
    let product_query = move || params.read().get("product");

    Effect::new(move |_| {
        if let Some(product_id) = product_query() {
            set_uuid.set(product_id.clone());

            spawn_local(async move {
                let path = format!("products/{}", product_id);

                let product =
                    request_json::<(), Product>(&path, Auth::Unauthorized, Method::GET, None).await;

                match product {
                    Ok(p) => {
                        set_name.set(p.name);
                        set_url.set(p.url);
                        set_material.set(p.material);
                        set_diameter.set(p.diameter);
                        set_weight.set(p.weight);
                        set_nozzle_temp.set(p.nozzle_temp);
                        set_bed_temp.set(p.bed_temp);
                        set_price_dollars_string.set(cents_to_dollars_string(p.price));
                    }
                    Err(e) => {
                        set_result_message.set(Some(ResultMessage::Error(format!(
                            "Failed to load product: ({}) {}",
                            e.status, e.message
                        ))));
                    }
                }
            });
        }
    });

    let dollars_string_to_cents = |s: String| -> Option<Cents> {
        let s = s.trim();

        if s.is_empty() {
            return None;
        }

        s.parse::<f32>().ok().map(Cents::from_dollars)
    };

    fn cents_to_dollars_string(cents: Cents) -> String {
        format!("{:.2}", cents.0 as f64 / 100.0)
    }

    let on_delete = {
        move |_| {
            let uuid = uuid.get();
            let path = format!("products/{}", uuid);

            spawn_local(async move {
                let res =
                    request_json::<(), ()>(&path, Auth::Authorized, Method::DELETE, None).await;

                match res {
                    Ok(_) => {
                        set_result_message.set(Some(ResultMessage::Success(format!(
                            "Product with UUID {} deleted successfully",
                            uuid
                        ))));
                        set_uuid.set(String::new());
                    }
                    Err(e) => {
                        set_result_message.set(Some(ResultMessage::Error(format!(
                            "Failed to delete product: ({}) {}",
                            e.status, e.message
                        ))));
                    }
                }
            });
        }
    };

    let on_update = {
        move |_| {
            let product = Product {
                uuid: String::new(),
                name: name.get().trim().to_string(),
                price: dollars_string_to_cents(price_dollars_string.get()).unwrap_or(Cents(0)),
                price_per_kg: Cents(0), // to be calculated server-side
                url: url.get(),
                material: material.get(),
                diameter: diameter.get(),
                weight: weight.get(),
                nozzle_temp: nozzle_temp.get(),
                bed_temp: bed_temp.get(),
            };

            enum ProductAction {
                Create,
                Update,
            }

            let action = if uuid.get().is_empty() {
                ProductAction::Create
            } else {
                ProductAction::Update
            };

            let path = if let ProductAction::Create = action {
                "products".to_string()
            } else {
                format!("products/{}", uuid.get())
            };

            spawn_local(async move {
                let product = request_json::<Product, Product>(
                    &path,
                    Auth::Authorized,
                    Method::POST,
                    Some(&product),
                )
                .await;

                let create_or_update_str = if let ProductAction::Create = action {
                    "create"
                } else {
                    "update"
                };

                match product {
                    Ok(p) => {
                        set_result_message.set(Some(ResultMessage::Success(format!(
                            "Product \"{}\" {}d successfully with UUID {}",
                            p.name, create_or_update_str, p.uuid
                        ))));
                        set_uuid.set(p.uuid);
                    }
                    Err(e) => {
                        set_result_message.set(Some(ResultMessage::Error(format!(
                            "Failed to {} product: ({}) {}",
                            create_or_update_str, e.status, e.message
                        ))));
                    }
                }
            });
        }
    };

    // helpers
    let select_value = move || match material.get() {
        FilamentMaterial::Other(_) => "Other".to_string(),
        m => m.to_string(),
    };

    let other_value = move || match material.get() {
        FilamentMaterial::Other(s) => s,
        _ => String::new(),
    };

    view! {
        <div class="container full-width">
            <h2>"Create/Update Product"</h2>
            <section style="display: grid; gap: 12px;">
                <div>
                    <label>"Product UUID"</label>
                    <input
                        class="input"
                        type="text"
                        placeholder="(leave blank to create new)"
                        prop:value=move || uuid.get()
                        on:input=move |e| set_uuid.set(event_target_value(&e))
                    />
                </div>
                <div>
                    <label>"Product Name"</label>
                    <input
                        class="input"
                        type="text"
                        placeholder="Product name"
                        prop:value=move || name.get()
                        on:input=move |e| set_name.set(event_target_value(&e))
                    />
                </div>
                <div class="filter-row">
                    <div class="filter-field">
                        <label>"Price"</label>
                        <input
                            class="input"
                            type="text"
                            placeholder="In USD (e.g. 19.99)"
                            prop:value=move || price_dollars_string.get()
                            on:input=move |e| set_price_dollars_string.set(event_target_value(&e))
                        />
                    </div>
                    <div>
                        <label>"Material"</label>
                        <select
                            class="input"
                            prop:value=select_value
                            on:change=move |e| {
                            let v = event_target_value(&e);

                            if v == "Other" {
                                set_material.update(|m| if !matches!(m, FilamentMaterial::Other(_)) {
                                *m = FilamentMaterial::Other(String::new());
                                });
                                return;
                            }

                            if let Some(m) = FilamentMaterial::iter()
                                .filter(|m| !matches!(m, FilamentMaterial::Other(_)))
                                .find(|m| m.to_string() == v)
                            {
                                set_material.set(m.clone());
                            } else {
                                set_material.set(FilamentMaterial::Other(v));
                            }
                            }
                        >
                            {
                                FilamentMaterial::iter()
                                    .filter(|m| !matches!(m, FilamentMaterial::Other(_)))
                                    .map(|m| {
                                        let label = m.to_string();
                                        view! { <option value=label.clone()>{ label.clone() }</option> }
                                    })
                                    .collect_view()
                            }
                            <option value="Other">"Other…"</option>
                        </select>

                        <Show when=move || matches!(material.get(), FilamentMaterial::Other(_))>
                            <input
                            class="input"
                            type="text"
                            placeholder="Material name"
                            prop:value=other_value
                            on:input=move |e| {
                                set_material.set(FilamentMaterial::Other(event_target_value(&e)));
                            }
                            />
                        </Show>
                    </div>
                    <div class="filter-field">
                        <label>"Diameter"</label>
                        <select
                            class="input"
                            on:change=move |e| {
                                let v = event_target_value(&e);
                                let all_diameters: Vec<_> = FilamentDiameter::iter().collect();

                                for d in &all_diameters {
                                    if let FilamentDiameter::Other(_) = d {
                                        continue;
                                    }

                                    if d.mm().to_string() == v {
                                        set_diameter.set(*d);
                                        break;
                                    } else {
                                        let v = v.parse::<f32>().unwrap_or(0.0);
                                        let v = (v * 100.0).round() as u16;
                                        set_diameter.set(FilamentDiameter::Other(v));
                                    }
                                }
                            }
                        >
                            {
                                FilamentDiameter::iter()
                                .filter(|m| !matches!(m, FilamentDiameter::Other(_)))
                                .map(|m| {
                                    let label = m.mm();
                                    view! { <option value=label>{ label }</option> }
                                })
                                .collect_view()
                            }
                            <option value="Other">"Other…"</option>
                        </select>
                        <Show when=move || matches!(diameter.get(), FilamentDiameter::Other(_))>
                            <input
                                class="input"
                                type="number"
                                inputmode="numeric"
                                placeholder="In mm (e.g. 1.75)"
                                on:input=move |e| {
                                    set_diameter.update(|df| {
                                        *df = FilamentDiameter::from_mm_string(&event_target_value(&e));
                                    });
                                }
                            />
                        </Show>
                    </div>
                    <div class="filter-field">
                        <label>"Spool Weight"</label>
                        <input
                            class="input"
                            type="number"
                            inputmode="numeric"
                            placeholder="In kg (e.g. 1.25)"
                            on:input=move |e| {
                                set_weight.update(|v| {
                                    let g = (event_target_value(&e).parse::<f32>().unwrap_or(0.0) * 1000.0).round() as u16;
                                    *v = Grams(g);
                                });
                            }
                        />
                    </div>
                    <div class="filter-field">
                            <TemperaturePicker label="Nozzle Temp" on_change=set_nozzle_temp />
                    </div>
                    <div class="filter-field">
                            <TemperaturePicker label="Bed Temp" on_change=set_bed_temp />
                    </div>
                </div>
                <div>
                    <label>"Product Page URL"</label>
                    <input
                        class="input"
                        type="text"
                        placeholder="https://example.com/product-page"
                        on:input=move |e| set_url.set(event_target_value(&e))
                    />
                </div>
                <div class="filter-row">
                    <button on:click=on_update>
                        {
                            move || if uuid.get().is_empty() {
                                "Create Product"
                            } else {
                                "Update Product"
                            }
                        }
                    </button>
                    <Show when=move || !uuid.get().is_empty()>
                        <button class="danger" on:click=on_delete>"Delete Product"</button>
                    </Show>
                </div>
                <Show when=move || result_message.get().is_some()>
                    {move || match result_message.get().unwrap() {
                        ResultMessage::Success(s) => view! { <p class="success">{s}</p> }.into_view(),
                        ResultMessage::Error(s)   => view! { <p class="error">{s}</p> }.into_view(),
                    }}
                </Show>
            </section>
        </div>
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TempMode {
    Unspecified,
    Exact,
    Range,
}

#[component]
fn TemperaturePicker(
    label: &'static str,
    on_change: WriteSignal<Option<TemperatureSpec>>,
) -> impl IntoView {
    let (mode, set_mode) = signal(TempMode::Unspecified);
    let (exact, set_exact) = signal(String::new());
    let (min_s, set_min_s) = signal(String::new());
    let (max_s, set_max_s) = signal(String::new());

    let refresh = {
        move || match mode.get() {
            TempMode::Unspecified => on_change.set(None),
            TempMode::Exact => {
                if let Ok(v) = exact.get().trim().parse::<u16>() {
                    on_change.set(Some(TemperatureSpec::Exact(Celsius(v))));
                } else {
                    on_change.set(None);
                }
            }
            TempMode::Range => {
                let a = min_s.get().trim().parse::<u16>().ok();
                let b = max_s.get().trim().parse::<u16>().ok();
                if let (Some(x), Some(y)) = (a, b) {
                    let (lo, hi) = if x <= y { (x, y) } else { (y, x) };
                    on_change.set(Some(TemperatureSpec::Range {
                        min: Celsius(lo),
                        max: Celsius(hi),
                    }));
                } else {
                    on_change.set(None);
                }
            }
        }
    };

    view! {
        <div class="filter-field">
            <label>{label}</label>
            <select
                class="input"
                on:change=move |e| {
                    match event_target_value(&e).as_str() {
                        "Exact" => set_mode.set(TempMode::Exact),
                        "Range" => set_mode.set(TempMode::Range),
                        _ => set_mode.set(TempMode::Unspecified),
                    }
                    refresh();
                }
            >
                <option value="Unspecified" selected>"Unspecified"</option>
                <option value="Exact">"Exact"</option>
                <option value="Range">"Range"</option>
            </select>

            <Show when=move || mode.get() == TempMode::Exact>
                <input
                    class="input mt-6"
                    type="number" inputmode="numeric"
                    placeholder="°C (e.g. 200)"
                    prop:value=move || exact.get()
                    on:input=move |e| { set_exact.set(event_target_value(&e)); refresh(); }
                />
            </Show>

            <Show when=move || mode.get() == TempMode::Range>
                <div class="row two mt-6">
                    <input
                        class="input" type="number" inputmode="numeric"
                        placeholder="Min °C"
                        prop:value=move || min_s.get()
                        on:input=move |e| { set_min_s.set(event_target_value(&e)); refresh(); }
                    />
                    <input
                        class="input" type="number" inputmode="numeric"
                        placeholder="Max °C"
                        prop:value=move || max_s.get()
                        on:input=move |e| { set_max_s.set(event_target_value(&e)); refresh(); }
                    />
                </div>
            </Show>
        </div>
    }
}
