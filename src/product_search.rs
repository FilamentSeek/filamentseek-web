use gloo_net::http::Method;
use leptos::{prelude::*, reactive::spawn_local};
use serde::Serialize;

use crate::{
    product::{Cents, FilamentMaterial, KNOWN_MATERIALS, Product},
    request::{Auth, request_json},
};

#[derive(Clone, Debug, PartialEq)]
enum MaterialFilter {
    Any,
    Material(FilamentMaterial),
    Other(String),
    Unspecified,
}

#[derive(Clone, Debug, PartialEq)]
enum DiameterFilter {
    Any,
    D175,
    D285,
    Other(String),
}

#[derive(Clone, Debug, PartialEq)]
enum WeightFilter {
    Any,
    G500,
    G750,
    G1000,
    G2000,
    Other(String),
}

#[derive(Serialize)]
pub struct ProductSearchRequest {
    name: Option<String>,
    // etc
}

#[component]
pub fn ProductSearch() -> impl IntoView {
    let (results, set_results) = signal::<Vec<Product>>(vec![]);

    let (query, set_query) = signal(String::new());
    let (min_price, set_min_price) = signal::<Option<Cents>>(None);
    let (max_price, set_max_price) = signal::<Option<Cents>>(None);

    let (mat_filter, set_mat_filter) = signal::<MaterialFilter>(MaterialFilter::Any);
    let (diam_filter, set_diam_filter) = signal::<DiameterFilter>(DiameterFilter::Any);
    let (weight_filter, set_weight_filter) = signal::<WeightFilter>(WeightFilter::Any);

    let is_admin = crate::session::Session::load()
        .map(|s| s.is_admin)
        .unwrap_or(false);

    let parse_dollars = |s: String| -> Option<Cents> {
        let s = s.trim();

        if s.is_empty() {
            return None;
        }

        s.parse::<f32>().ok().map(Cents::from_dollars)
    };

    let on_search = {
        move |_| {
            let query = if query.get().trim().is_empty() {
                None
            } else {
                Some(query.get().trim().to_string())
            };

            let payload: ProductSearchRequest = ProductSearchRequest { name: query };

            spawn_local(async move {
                let products = request_json::<ProductSearchRequest, Vec<Product>>(
                    "products/search",
                    Auth::Unauthorized,
                    Method::POST,
                    Some(&payload),
                )
                .await
                .unwrap_or_else(|_| vec![]);

                set_results.set(products);
            });
        }
    };

    view! {
        <div class="container full-width">
            <section style="display: grid; gap: 12px;">
                <input
                    class="input"
                    type="text"
                    placeholder="Search by name…"
                    on:input=move |e| set_query.set(event_target_value(&e))
                />
                <div class="filter-row">
                    <div>
                        <label>"Material"</label>
                        <select
                            class="input"
                            on:change=move |e| {
                                let v = event_target_value(&e);

                                match v.as_str() {
                                    "Any" => set_mat_filter.set(MaterialFilter::Any),
                                    "Unspecified" => set_mat_filter.set(MaterialFilter::Unspecified),
                                    "Other" => set_mat_filter.set(MaterialFilter::Other(String::new())),
                                    _ => {
                                        let chosen = KNOWN_MATERIALS.iter()
                                            .find(|m| m.to_string() == v)
                                            .cloned();
                                        if let Some(m) = chosen {
                                            set_mat_filter.set(MaterialFilter::Material(m));
                                        } else {
                                            set_mat_filter.set(MaterialFilter::Any);
                                        }
                                    }
                                }
                            }
                        >
                            <option value="Any">"Any"</option>
                            { KNOWN_MATERIALS.iter()
                                .map(|m| {
                                    let label = m.to_string();
                                    view! { <option value=label.clone()>{ label.clone() }</option> }
                                })
                                .collect_view()
                            }
                            <option value="Unspecified">"Unspecified"</option>
                            <option value="Other">"Other…"</option>
                        </select>
                        <Show when=move || matches!(mat_filter.get(), MaterialFilter::Other(_))>
                            <input
                                class="input"
                                type="text"
                                placeholder="Enter material…"
                                on:input=move |e| {
                                    set_mat_filter.update(|mf| {
                                        if let MaterialFilter::Other(s) = mf {
                                            *s = event_target_value(&e);
                                        }
                                    });
                                }
                            />
                        </Show>
                    </div>
                    <div class="filter-field">
                        <label>"Diameter"</label>
                        <select
                            class="input"
                            on:change=move |e| {
                                match event_target_value(&e).as_str() {
                                    "Any" => set_diam_filter.set(DiameterFilter::Any),
                                    "1.75" => set_diam_filter.set(DiameterFilter::D175),
                                    "2.85" => set_diam_filter.set(DiameterFilter::D285),
                                    "Other" => set_diam_filter.set(DiameterFilter::Other(String::new())),
                                    _ => set_diam_filter.set(DiameterFilter::Any),
                                }
                            }
                        >
                            <option value="Any">"Any"</option>
                            <option value="1.75">"1.75 mm"</option>
                            <option value="2.85">"2.85 mm"</option>
                            <option value="Other">"Other…"</option>
                        </select>
                        <Show when=move || matches!(diam_filter.get(), DiameterFilter::Other(_))>
                            <input
                                class="input"
                                type="number"
                                inputmode="numeric"
                                placeholder="Hundredths of mm (e.g. 175)"
                                on:input=move |e| {
                                    set_diam_filter.update(|df| {
                                        if let DiameterFilter::Other(s) = df {
                                            *s = event_target_value(&e);
                                        }
                                    });
                                }
                            />
                        </Show>
                    </div>

                    {/* Weight */}
                    <div class="filter-field">
                        <label>"Spool Weight"</label>
                        <select
                            class="input"
                            on:change=move |e| {
                                match event_target_value(&e).as_str() {
                                    "Any" => set_weight_filter.set(WeightFilter::Any),
                                    "500" => set_weight_filter.set(WeightFilter::G500),
                                    "750" => set_weight_filter.set(WeightFilter::G750),
                                    "1000" => set_weight_filter.set(WeightFilter::G1000),
                                    "2000" => set_weight_filter.set(WeightFilter::G2000),
                                    "Other" => set_weight_filter.set(WeightFilter::Other(String::new())),
                                    _ => set_weight_filter.set(WeightFilter::Any),
                                }
                            }
                        >
                            <option value="Any">"Any"</option>
                            <option value="500">"500 g"</option>
                            <option value="750">"750 g"</option>
                            <option value="1000">"1 kg"</option>
                            <option value="2000">"2 kg"</option>
                            <option value="Other">"Other…"</option>
                        </select>

                        <Show when=move || matches!(weight_filter.get(), WeightFilter::Other(_))>
                            <input
                                class="input"
                                type="number"
                                inputmode="numeric"
                                placeholder="Grams (e.g. 1200)"
                                on:input=move |e| {
                                    set_weight_filter.update(|wf| {
                                        if let WeightFilter::Other(s) = wf {
                                            *s = event_target_value(&e);
                                        }
                                    });
                                }
                            />
                        </Show>
                    </div>
                </div>

                <div class="filter-row">
                    <div>
                        <label>"Min $"</label>
                        <input
                            class="input"
                            inputmode="decimal"
                            placeholder="e.g. 12.99"
                            on:input=move |e| set_min_price.set(parse_dollars(event_target_value(&e)))
                        />
                    </div>
                    <div>
                        <label>"Max $"</label>
                        <input
                            class="input"
                            inputmode="decimal"
                            placeholder="e.g. 30"
                            on:input=move |e| set_max_price.set(parse_dollars(event_target_value(&e)))
                        />
                    </div>
                    <div style="justify-content: end;">
                        <button class="" on:click=on_search>
                            "Seek"
                        </button>
                    </div>
                </div>
            </section>

            <section class="results">
                <Show
                    when=move || !results.get().is_empty()
                    fallback=|| view! { <div class="empty">"No products match your filters."</div> }
                >
                    <ProductTable products=results is_admin />
                </Show>
            </section>
        </div>
    }
}

#[component]
fn ProductTable(products: ReadSignal<Vec<Product>>, is_admin: bool) -> impl IntoView {
    view! {
        <table class="product-table">
            <thead>
                <tr>
                    <th>"Name"</th>
                    <th>"Price"</th>
                    <th>"Material"</th>
                    <th>"Diameter"</th>
                    <th>"Weight"</th>
                    <Show when=move || is_admin>
                        <th>"Admin"</th>
                    </Show>
                </tr>
            </thead>
            <tbody>
                <For
                    each=move || products.get()
                    key=|p| p.uuid.clone()
                    children=move |p: Product| view! { <ProductRow product=p is_admin /> }
                />
            </tbody>
        </table>
    }
}

#[component]
fn ProductRow(product: Product, is_admin: bool) -> impl IntoView {
    let url_admin = product.url.clone();
    let url_user = product.url.clone();

    view! {
        <tr class="row-link-wrap">
            <td>{product.name.clone()}</td>
            <td>{product.price.to_string()}</td>
            <td>{product.material.to_string()}</td>
            <td>{product.diameter.to_string()}</td>
            <td>{product.weight.to_string()}</td>

            <Show when=move || is_admin>
                <td>
                    <a href={format!("/admin?product={}", product.uuid)} target="_blank">"Edit"</a>
                    <a href={url_admin.clone()} target="_blank">"Product page"</a>
                </td>
            </Show>

            <Show when=move || !is_admin>
                <td class="overlay-cell">
                    <a class="row-overlay" href={url_user.clone()} target="_blank"></a>
                </td>
            </Show>
        </tr>
    }
}
