use std::{fmt::Display, str::FromStr};

use gloo_net::http::Method;
use leptos::{prelude::*, reactive::spawn_local};
use serde::{Deserialize, Serialize};

use crate::{
    product::{
        Cents, FilamentColor, FilamentDiameter, FilamentMaterial, Grams, KNOWN_COLORS,
        KNOWN_MATERIALS, Product, Retailer,
    },
    request::{Auth, request_json},
};

#[derive(Clone, Debug, PartialEq)]
enum MaterialFilter {
    Any,
    Material(FilamentMaterial),
    Other(String),
    Unspecified,
}

impl FromStr for MaterialFilter {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "Any" {
            Ok(MaterialFilter::Any)
        } else if s == "Unspecified" {
            Ok(MaterialFilter::Unspecified)
        } else if s == "Other" {
            Ok(MaterialFilter::Other(String::new()))
        } else {
            let chosen = KNOWN_MATERIALS.iter().find(|m| m.to_string() == s).cloned();
            if let Some(m) = chosen {
                Ok(MaterialFilter::Material(m))
            } else {
                Err(())
            }
        }
    }
}

impl Display for MaterialFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MaterialFilter::Any => write!(f, "Any"),
            MaterialFilter::Material(m) => write!(f, "{}", m),
            MaterialFilter::Other(s) => write!(f, "Other: {}", s),
            MaterialFilter::Unspecified => write!(f, "Unspecified"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum ColorFilter {
    Any,
    Material(FilamentColor),
    Other(String),
    Unspecified,
}

impl FromStr for ColorFilter {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "Any" {
            Ok(ColorFilter::Any)
        } else if s == "Unspecified" {
            Ok(ColorFilter::Unspecified)
        } else if s == "Other" {
            Ok(ColorFilter::Other(String::new()))
        } else {
            let chosen = KNOWN_COLORS.iter().find(|c| c.to_string() == s).cloned();
            if let Some(c) = chosen {
                Ok(ColorFilter::Material(c))
            } else {
                Err(())
            }
        }
    }
}

impl Display for ColorFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColorFilter::Any => write!(f, "Any"),
            ColorFilter::Material(c) => write!(f, "{}", c),
            ColorFilter::Other(s) => write!(f, "Other: {}", s),
            ColorFilter::Unspecified => write!(f, "Unspecified"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum DiameterFilter {
    Any,
    D175,
    D285,
    Other(String),
}

impl Display for DiameterFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiameterFilter::Any => write!(f, "Any"),
            DiameterFilter::D175 => write!(f, "1.75"),
            DiameterFilter::D285 => write!(f, "2.85"),
            DiameterFilter::Other(s) => write!(f, "Other: {}", s),
        }
    }
}

impl FromStr for DiameterFilter {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Any" => Ok(DiameterFilter::Any),
            "1.75" => Ok(DiameterFilter::D175),
            "2.85" => Ok(DiameterFilter::D285),
            "Other" => Ok(DiameterFilter::Other(String::new())),
            _ => Err(()),
        }
    }
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

impl Display for WeightFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WeightFilter::Any => write!(f, "Any"),
            WeightFilter::G500 => write!(f, "500"),
            WeightFilter::G750 => write!(f, "750"),
            WeightFilter::G1000 => write!(f, "1000"),
            WeightFilter::G2000 => write!(f, "2000"),
            WeightFilter::Other(s) => write!(f, "Other: {}", s),
        }
    }
}

impl FromStr for WeightFilter {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Any" => Ok(WeightFilter::Any),
            "500" => Ok(WeightFilter::G500),
            "750" => Ok(WeightFilter::G750),
            "1000" => Ok(WeightFilter::G1000),
            "2000" => Ok(WeightFilter::G2000),
            "Other" => Ok(WeightFilter::Other(String::new())),
            _ => Err(()),
        }
    }
}

#[derive(Serialize)]
pub struct ProductSearchRequest {
    name: Option<String>,
    min_price: Option<Cents>,
    max_price: Option<Cents>,
    material: Option<FilamentMaterial>,
    diameter: Option<FilamentDiameter>,
    weight: Option<Grams>,
    color: Option<FilamentColor>,
    page: u32,
    per_page: u32,
}

const PER_PAGE: u32 = 50;

#[component]
pub fn ProductSearch() -> impl IntoView {
    let (seeking, set_seeking) = signal(true);
    let (results, set_results) = signal::<Vec<Product>>(vec![]);

    let (query, set_query) = signal(String::new());
    let (min_price, set_min_price) = signal::<Option<Cents>>(None);
    let (max_price, set_max_price) = signal::<Option<Cents>>(None);

    let (mat_filter, set_mat_filter) = signal::<MaterialFilter>(MaterialFilter::Any);
    let (col_filter, set_col_filter) = signal::<ColorFilter>(ColorFilter::Any);
    let (diam_filter, set_diam_filter) = signal::<DiameterFilter>(DiameterFilter::Any);
    let (weight_filter, set_weight_filter) = signal::<WeightFilter>(WeightFilter::Any);

    let (page, set_page) = signal(1u32);
    let (total_pages, set_total_pages) = signal(1u32);
    let (total_results, set_total_results) = signal(0u32);

    let is_admin = crate::session::Session::load()
        .map(|s| s.is_admin)
        .unwrap_or(false);

    let loc = leptos_router::hooks::use_location();
    let navigate = leptos_router::hooks::use_navigate();

    // Parse from URL
    Effect::new(move |_| {
        let search = loc.search.get_untracked();
        if let Ok(params) = web_sys::UrlSearchParams::new_with_str(&search) {
            if let Some(q) = params.get("q") {
                set_query.set(q);
            }
            if let Some(p) = params.get("page") {
                if let Ok(n) = p.parse::<u32>() {
                    set_page.set(n);
                }
            }
            if let Some(v) = params.get("min_price") {
                if let Ok(n) = v.parse::<u32>() {
                    set_min_price.set(Some(Cents(n)));
                }
            }
            if let Some(v) = params.get("max_price") {
                if let Ok(n) = v.parse::<u32>() {
                    set_max_price.set(Some(Cents(n)));
                }
            }
            if let Some(v) = params.get("mat") {
                if let Ok(m) = v.parse::<MaterialFilter>() {
                    set_mat_filter.set(m);
                }
            }
            if let Some(v) = params.get("col") {
                if let Ok(c) = v.parse::<ColorFilter>() {
                    set_col_filter.set(c);
                }
            }
            if let Some(v) = params.get("diam") {
                if let Ok(d) = v.parse::<DiameterFilter>() {
                    set_diam_filter.set(d);
                }
            }
            if let Some(v) = params.get("weight") {
                if let Ok(w) = v.parse::<WeightFilter>() {
                    set_weight_filter.set(w);
                }
            }
        }
    });

    // Write to URL
    Effect::new(move |_| {
        let params = web_sys::UrlSearchParams::new().unwrap();

        let q = query.get();
        if !q.is_empty() {
            params.set("q", &q);
        }

        if let Some(min) = min_price.get() {
            params.set("min_price", &min.0.to_string());
        }
        if let Some(max) = max_price.get() {
            params.set("max_price", &max.0.to_string());
        }

        if mat_filter.get() != MaterialFilter::Any {
            params.set("mat", &mat_filter.get().to_string());
        }
        if col_filter.get() != ColorFilter::Any {
            params.set("col", &col_filter.get().to_string());
        }
        if diam_filter.get() != DiameterFilter::Any {
            params.set("diam", &diam_filter.get().to_string());
        }
        if weight_filter.get() != WeightFilter::Any {
            params.set("weight", &weight_filter.get().to_string());
        }

        if page.get() != 1 {
            params.set("page", &page.get().to_string());
        }

        let _ = navigate(&format!("?{}", params.to_string()), Default::default());
    });

    let parse_dollars = |s: String| -> Option<Cents> {
        let s = s.trim();

        if s.is_empty() {
            return None;
        }

        s.parse::<f32>().ok().map(Cents::from_dollars)
    };

    let search = {
        let set_seeking = set_seeking;
        let set_results = set_results;
        let set_total_pages = set_total_pages;

        move || {
            let query = if query.get().trim().is_empty() {
                None
            } else {
                Some(query.get().trim().to_string())
            };

            let payload: ProductSearchRequest = ProductSearchRequest {
                name: query,
                min_price: min_price.get(),
                max_price: max_price.get(),
                material: match mat_filter.get() {
                    MaterialFilter::Any => None,
                    MaterialFilter::Material(m) => Some(m.clone()),
                    MaterialFilter::Other(s) => {
                        if s.trim().is_empty() {
                            None
                        } else {
                            Some(FilamentMaterial::Other(s.trim().to_string()))
                        }
                    }
                    MaterialFilter::Unspecified => Some(FilamentMaterial::Unspecified),
                },
                color: match col_filter.get() {
                    ColorFilter::Any => None,
                    ColorFilter::Material(c) => Some(c.clone()),
                    ColorFilter::Other(s) => {
                        if s.trim().is_empty() {
                            None
                        } else {
                            Some(FilamentColor::Other(s.trim().to_string()))
                        }
                    }
                    ColorFilter::Unspecified => Some(FilamentColor::Unspecified),
                },
                diameter: match diam_filter.get() {
                    DiameterFilter::Any => None,
                    DiameterFilter::D175 => Some(FilamentDiameter::D175),
                    DiameterFilter::D285 => Some(FilamentDiameter::D285),
                    DiameterFilter::Other(s) => {
                        if s.trim().is_empty() {
                            None
                        } else {
                            Some(FilamentDiameter::from_mm_string(&s))
                        }
                    }
                },
                weight: match weight_filter.get() {
                    WeightFilter::Any => None,
                    WeightFilter::G500 => Some(Grams(500)),
                    WeightFilter::G750 => Some(Grams(750)),
                    WeightFilter::G1000 => Some(Grams(1000)),
                    WeightFilter::G2000 => Some(Grams(2000)),
                    WeightFilter::Other(s) => {
                        if s.trim().is_empty() {
                            None
                        } else {
                            Some(Grams::from_kg_string(&s))
                        }
                    }
                },
                page: page.get(),
                per_page: PER_PAGE,
            };

            spawn_local(async move {
                set_seeking.set(true);
                let response = search_products(&payload).await;
                set_results.set(response.items);
                set_total_pages.set(response.total_pages as u32);
                set_total_results.set(response.total as u32);
                set_seeking.set(false);
            });
        }
    };

    let on_search = move |_| {
        set_page.set(1);
        search();
    };

    let prev_page = StoredValue::new(page.get());

    Effect::new(move |_| {
        let current = page.get();

        if current != prev_page.get_value() {
            prev_page.set_value(current);
            search();
        }
    });

    Effect::new(move |_| {
        spawn_local(async move {
            search();
        });
    });

    view! {
        <div class="container full-width">
            <section style="display: grid; gap: 12px;">
                <h3>
                    "FilamentSeek is in its initial development phase. Features, content, and design are still in progress."
                </h3>
                <input
                    class="input"
                    type="text"
                    placeholder="Search by name…"
                    on:input=move |e| set_query.set(event_target_value(&e))
                />
                <div class="options-row">
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
                                placeholder="Material name"
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
                    <div>
                        <label>"Color"</label>
                        <select
                            class="input"
                            on:change=move |e| {
                                let v = event_target_value(&e);

                                match v.as_str() {
                                    "Any" => set_col_filter.set(ColorFilter::Any),
                                    "Unspecified" => set_col_filter.set(ColorFilter::Unspecified),
                                    "Other" => set_col_filter.set(ColorFilter::Other(String::new())),
                                    _ => {
                                        let chosen = KNOWN_COLORS.iter()
                                            .find(|m| m.to_string() == v)
                                            .cloned();
                                        if let Some(m) = chosen {
                                            set_col_filter.set(ColorFilter::Material(m));
                                        } else {
                                            set_col_filter.set(ColorFilter::Any);
                                        }
                                    }
                                }
                            }
                        >
                            <option value="Any">"Any"</option>
                            { KNOWN_COLORS.iter()
                                .map(|m| {
                                    let label = m.to_string();
                                    view! { <option value=label.clone()>{ label.clone() }</option> }
                                })
                                .collect_view()
                            }
                            <option value="Unspecified">"Unspecified"</option>
                            <option value="Other">"Other…"</option>
                        </select>
                        <Show when=move || matches!(col_filter.get(), ColorFilter::Other(_))>
                            <input
                                class="input"
                                type="text"
                                placeholder="Color name"
                                on:input=move |e| {
                                    set_col_filter.update(|mf| {
                                        if let ColorFilter::Other(s) = mf {
                                            *s = event_target_value(&e);
                                        }
                                    });
                                }
                            />
                        </Show>
                    </div>
                    <div>
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
                                placeholder="Millimeters (e.g. 1.75)"
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
                    <div>
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
                                placeholder="Kilograms (e.g. 1.2)"
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

                <div class="options-row">
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
                {move || {
                    if seeking.get() {
                        view! { <div class="loading">"Seeking..."</div> }.into_any()
                    } else if results.get().is_empty() {
                        view! { <div class="empty">"No products match your filters."</div> }.into_any()
                    } else {
                        view! { <ProductTable
                            products=results
                            is_admin=is_admin
                            page=page
                            total_pages=total_pages
                            set_page=set_page
                            total_results=total_results
                        /> }.into_any()
                    }
                }}
            </section>
        </div>
    }
}

#[component]
fn ProductTable(
    products: ReadSignal<Vec<Product>>,
    is_admin: bool,
    set_page: WriteSignal<u32>,
    page: ReadSignal<u32>,
    total_pages: ReadSignal<u32>,
    total_results: ReadSignal<u32>,
) -> impl IntoView {
    view! {
        <div style="margin: 12px 0;">
            {total_results.get()} " results"
        </div>
        <Pagination page=page total_pages=total_pages set_page=set_page />
        <table class="product-table">
            <thead>
                <tr>
                    <th>"Name"</th>
                    <th>"Price"</th>
                    <th>"$ / kg"</th>
                    <th>"Material"</th>
                    <th>"Color"</th>
                    <th>"Diameter"</th>
                    <th>"Weight"</th>
                    <th>"Retailer"</th>
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
        <Pagination page=page total_pages=total_pages set_page=set_page />
    }
}

#[component]
fn ProductRow(product: Product, is_admin: bool) -> impl IntoView {
    let url_admin = product.url.clone();
    let url_user = product.url.clone();

    view! {
        <tr class="row-link-wrap">
            <td style="max-width: 200px">{product.name.clone()}</td>
            <td>{product.price.to_string()}</td>
            <td>{product.price_per_kg.to_string()}</td>
            <td>{product.material.to_string()}</td>
            <td style=format!("color: {}", product.color.to_string())>
                {product.color.to_string()}
            </td>
            <td>{product.diameter.to_string()}</td>
            <td>{product.weight.to_string()}</td>
            <td>
                {product.retailer.to_string()}
                {if product.retailer == Retailer::Amazon {
                    view! { <div>"(#ad)"</div> }.into_any()
                } else {
                    view! { <></> }.into_any()
                }}
            </td>

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

async fn search_products(request: &ProductSearchRequest) -> ProductSearchResponse {
    request_json::<ProductSearchRequest, ProductSearchResponse>(
        "products/search",
        Auth::Unauthorized,
        Method::POST,
        Some(&request),
    )
    .await
    .unwrap_or(ProductSearchResponse {
        items: vec![],
        total: 0,
        total_pages: 1,
    })
}

#[component]
pub fn Pagination(
    set_page: WriteSignal<u32>,
    page: ReadSignal<u32>,
    total_pages: ReadSignal<u32>,
) -> impl IntoView {
    let go = move |n: u32| set_page.set(n.clamp(1, total_pages.get()));
    let pages = move || (1..=total_pages.get()).collect::<Vec<u32>>();

    view! {
        <nav style="display: flex; flex-direction: row; justify-content: center;">
            <For
                each=pages
                key=|n| *n
                children=move |n| {
                    let is_current = move || page.get() == n;
                    view! {
                        <button
                            on:click=move |_| go(n)
                            disabled=is_current
                            style="width:30px; margin: 20px;"
                        >
                            {n}
                        </button>
                    }
                }
            />
        </nav>
    }
}

#[derive(Deserialize)]
pub struct ProductSearchResponse {
    pub items: Vec<Product>,
    pub total: u64,
    pub total_pages: u64,
}
