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

const MAX_PRICE_CAP: u32 = 100;
const MAX_PAGE_SIZE: u32 = 50;

#[derive(Clone, Debug, PartialEq)]
enum MaterialFilter {
    Any,
    Material(FilamentMaterial),
    Other(String),
    Unspecified,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SortBy {
    Price,
    PricePerKg,
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
    sort_by: Option<SortBy>,
}

const PER_PAGE: u32 = 50;

#[component]
pub fn ProductSearch() -> impl IntoView {
    let (seeking, set_seeking) = signal(true);
    let (results, set_results) = signal::<Vec<Product>>(vec![]);
    let (query, set_query) = signal(String::new());
    let (mat_filter, set_mat_filter) = signal::<MaterialFilter>(MaterialFilter::Any);
    let (col_filter, set_col_filter) = signal::<ColorFilter>(ColorFilter::Any);
    let (diam_filter, set_diam_filter) = signal::<DiameterFilter>(DiameterFilter::Any);
    let (weight_filter, set_weight_filter) = signal::<WeightFilter>(WeightFilter::Any);
    let (sortby, set_sortby) = signal::<SortBy>(SortBy::PricePerKg);

    let (page, set_page) = signal(1u32);
    let (total_pages, set_total_pages) = signal(1u32);
    let (total_results, set_total_results) = signal(0u32);

    let (min_price_int, set_min_price_int) = signal(0u32);
    let (max_price_int, set_max_price_int) = signal(100u32);
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
            if let Some(p) = params.get("page")
                && let Ok(n) = p.parse::<u32>()
            {
                set_page.set(n);
            }
            if let Some(v) = params.get("min_price") {
                set_min_price_int.set(v.parse::<u32>().unwrap_or(0).clamp(0, MAX_PRICE_CAP));
            }
            if let Some(v) = params.get("max_price") {
                set_max_price_int.set(
                    v.parse::<u32>()
                        .unwrap_or(MAX_PRICE_CAP)
                        .clamp(0, MAX_PRICE_CAP),
                );
            }
            if let Some(v) = params.get("mat")
                && let Ok(m) = v.parse::<MaterialFilter>()
            {
                set_mat_filter.set(m);
            }
            if let Some(v) = params.get("col")
                && let Ok(c) = v.parse::<ColorFilter>()
            {
                set_col_filter.set(c);
            }
            if let Some(v) = params.get("diam")
                && let Ok(d) = v.parse::<DiameterFilter>()
            {
                set_diam_filter.set(d);
            }
            if let Some(v) = params.get("weight")
                && let Ok(w) = v.parse::<WeightFilter>()
            {
                set_weight_filter.set(w);
            }
            if let Some(v) = params.get("sortby") {
                if let Ok(s) = serde_json::from_str::<SortBy>(&format!("\"{}\"", v)) {
                    set_sortby.set(s);
                }
            }
        }
    });

    // Write to URL
    Effect::new(move |_| {
        let params = web_sys::UrlSearchParams::new().unwrap();

        let query = query.get_untracked();
        let query = query.trim();
        if !query.is_empty() {
            params.set("q", query);
        }

        let min = min_price_int.get_untracked();
        if min != 0 {
            params.set("min_price", &min.to_string());
        }

        let max = max_price_int.get_untracked();
        if max != MAX_PRICE_CAP {
            params.set("max_price", &max.to_string());
        }

        let mat_filter = mat_filter.get_untracked();
        if mat_filter != MaterialFilter::Any {
            params.set("mat", &mat_filter.to_string());
        }

        let col_filter = col_filter.get_untracked();
        if col_filter != ColorFilter::Any {
            params.set("col", &col_filter.to_string());
        }

        let diam_filter = diam_filter.get_untracked();
        if diam_filter != DiameterFilter::Any {
            params.set("diam", &diam_filter.to_string());
        }

        let weight_filter = weight_filter.get_untracked();
        if weight_filter != WeightFilter::Any {
            params.set("weight", &weight_filter.to_string());
        }

        let page = page.get();
        if page != 1 {
            params.set("page", &page.to_string());
        }

        let sortby = sortby.get();
        if sortby != SortBy::PricePerKg {
            if let Ok(s) = serde_json::to_string(&sortby) {
                params.set("sortby", s.trim_matches('"'));
            }
        }
        navigate(&format!("?{}", params.to_string()), Default::default());
    });

    let search = {
        move || {
            let query = if query.get_untracked().trim().is_empty() {
                None
            } else {
                Some(query.get_untracked().trim().to_string())
            };

            let payload: ProductSearchRequest = ProductSearchRequest {
                name: query,
                min_price: Some(Cents(min_price_int.get_untracked() * 100)),
                max_price: Some(Cents(max_price_int.get_untracked() * 100)),
                material: match mat_filter.get_untracked() {
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
                color: match col_filter.get_untracked() {
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
                diameter: match diam_filter.get_untracked() {
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
                weight: match weight_filter.get_untracked() {
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
                page: page.get_untracked(),
                per_page: PER_PAGE,
                sort_by: Some(sortby.get_untracked()),
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

    let prev_page = StoredValue::new(page.get_untracked());

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

    Effect::new(move |_| {
        let _ = sortby.get();
        search();
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
                    prop:value=move || query.get()
                    on:input=move |e| set_query.set(event_target_value(&e))
                />
                <div class="options-row">
                    <div>
                        <label>"Material"</label>
                        <select
                            class="input"
                            prop:value=move || match mat_filter.get() {
                                MaterialFilter::Any => "Any".to_string(),
                                MaterialFilter::Unspecified => "Unspecified".to_string(),
                                MaterialFilter::Other(_) => "Other".to_string(),
                                MaterialFilter::Material(m) => m.to_string(),
                            }
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
                            prop:value=move || match col_filter.get() {
                                ColorFilter::Any => "Any".to_string(),
                                ColorFilter::Unspecified => "Unspecified".to_string(),
                                ColorFilter::Other(_) => "Other".to_string(),
                                ColorFilter::Material(c) => c.to_string(),
                            }
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
                            prop:value=move || match diam_filter.get() {
                                DiameterFilter::Any => "Any".to_string(),
                                DiameterFilter::D175 => "1.75".to_string(),
                                DiameterFilter::D285 => "2.85".to_string(),
                                DiameterFilter::Other(_) => "Other".to_string(),
                            }
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
                            prop:value=move || match weight_filter.get() {
                                WeightFilter::Any => "Any".to_string(),
                                WeightFilter::G500 => "500".to_string(),
                                WeightFilter::G750 => "750".to_string(),
                                WeightFilter::G1000 => "1000".to_string(),
                                WeightFilter::G2000 => "2000".to_string(),
                                WeightFilter::Other(_) => "Other".to_string(),
                            }
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

                <div class="options-row seek-row">
                    <RangeSlider
                        min_value=min_price_int
                        set_min_value=set_min_price_int
                        max_value=max_price_int
                        set_max_value=set_max_price_int
                        min_limit=0
                        max_limit=100
                        step=1
                        gap=1
                    />
                    <div style="justify-content: center;">
                        <button on:click=on_search>
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
                            sortby=sortby
                            set_sortby=set_sortby
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
    sortby: ReadSignal<SortBy>,
    set_sortby: WriteSignal<SortBy>,
) -> impl IntoView {
    let p = page.get_untracked();
    let total = total_results.get_untracked();

    let start = if total == 0 {
        0
    } else {
        (p.saturating_sub(1)) * MAX_PAGE_SIZE + 1
    };
    let end = (p * MAX_PAGE_SIZE).min(total);

    let summary = if total_pages.get_untracked() == 1 {
        format!("{total} results")
    } else {
        format!("{start}-{end} of {total} results")
    };

    view! {
        <Pagination page=page total_pages=total_pages set_page=set_page />
        <div style="text-align: right;">
            {summary.clone()}
        </div>
        <table class="product-table">
            <thead>
                <tr>
                    <th>"Name"</th>
                    <th class="wide-col">
                        <button
                            disabled={move || matches!(sortby.get(), SortBy::Price)}
                            on:click=move |_| {
                                set_sortby.set(SortBy::Price);
                            }>
                            "Price"
                        </button>
                    </th>
                    <th class="wide-col">
                        <button
                            disabled={move || matches!(sortby.get(), SortBy::PricePerKg)}
                            on:click=move |_| {
                                set_sortby.set(SortBy::PricePerKg);
                            }>
                            "$ / kg"
                        </button>
                    </th>
                    <th class="compact-col">
                        <button
                            style="margin-bottom: 8px;"
                            disabled={move || matches!(sortby.get(), SortBy::Price)}
                            on:click=move |_| {
                                set_sortby.set(SortBy::Price);
                            }>
                            "Price"
                        </button>
                        <button
                            disabled={move || matches!(sortby.get(), SortBy::PricePerKg)}
                            on:click=move |_| {
                                set_sortby.set(SortBy::PricePerKg);
                            }>
                            "$ / kg"
                        </button>
                    </th>
                    <th class="wide-col">"Material"</th>
                    <th class="wide-col">"Color"</th>
                    <th class="wide-col">"Diameter"</th>
                    <th class="wide-col">"Weight"</th>
                    <th class="compact-col">"Specs"</th>
                    <th class="wide-col">"Retailer"</th>
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
        <div style="text-align: center;">
            {summary}
        </div>
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
            <td class="wide-col">{product.price.to_string()}</td>
            <td class="wide-col">{product.price_per_kg.to_string()}</td>

            <td class="compact-col">
                {product.price.to_string()}
                <br />
                <br />
                {product.price_per_kg.to_string()}"/kg"
            </td>

            <td class="wide-col">{product.material.to_string()}</td>

            <td class="wide-col" style=format!("color: {}", product.color.hex())>
                {product.color.to_string()}
            </td>

            <td class="wide-col">{product.diameter.to_string()}</td>
            <td class="wide-col">{product.weight.to_string()}</td>

            <td class="compact-col">
                <div>{product.material.to_string()}</div>
                <div style=format!("color: {}", product.color.hex())>
                {product.color.to_string()}
                </div>
                <div>{product.diameter.to_string()}</div>
                <div>{product.weight.to_string()}</div>
                <div>
                    {product.retailer.to_string()}
                    {if product.retailer == Retailer::Amazon {
                        view! { <div>"(#ad)"</div> }.into_any()
                    } else {
                        let _: () = view! { <></> };
                        ().into_any()
                    }}
                </div>
            </td>

            <td class="wide-col">
                {product.retailer.to_string()}
                {if product.retailer == Retailer::Amazon {
                    view! { <div>"(#ad)"</div> }.into_any()
                } else {
                    let _: () = view! { <></> };
                    ().into_any()
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
        Some(request),
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
                            style="width:30px; margin: 20px 5px;"
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

#[component]
pub fn RangeSlider(
    min_value: ReadSignal<u32>,
    set_min_value: WriteSignal<u32>,
    max_value: ReadSignal<u32>,
    set_max_value: WriteSignal<u32>,
    min_limit: u32,
    max_limit: u32,
    step: u32,
    gap: u32,
) -> impl IntoView {
    let on_min_input = move |ev: web_sys::Event| {
        if let Ok(mut v) = event_target_value(&ev).parse::<u32>() {
            v = v.clamp(min_limit, max_limit);
            if v > max_value.get() - gap {
                v = (max_value.get() - gap).max(min_limit);
            }
            set_min_value.set(v);
        }
    };

    let on_max_input = move |ev: web_sys::Event| {
        if let Ok(mut v) = event_target_value(&ev).parse::<u32>() {
            v = v.clamp(min_limit, max_limit);
            if v < min_value.get() + gap {
                v = (min_value.get() + gap).min(max_limit);
            }
            set_max_value.set(v);
        }
    };

    view! {
        <div>
            <div class="input-box">
                <div class="min-box">
                    <div>
                        "Min $"
                    </div>
                    <input
                        type="number"
                        class="min-input"
                        prop:value=move || min_value.get().to_string()
                        min=min_limit
                        prop:max=move || (max_value.get() - gap).to_string()
                        on:input=on_min_input
                    />
                </div>
                <div class="max-box">
                    <div>
                        "Max $"
                    </div>
                    <input
                        type="number"
                        class="max-input"
                        prop:value=move || max_value.get().to_string()
                        prop:min=move || (min_value.get() + gap).to_string()
                        max=max_limit
                        on:input=on_max_input
                    />
                </div>
            </div>

            <div class="range-slider">
                <input
                    type="range"
                    min=min_limit
                    max=max_limit
                    step=step
                    prop:value=move || min_value.get().to_string()
                    on:input=move |ev| {
                        if let Ok(mut v) = event_target_value(&ev).parse::<u32>() {
                            if v > max_value.get() - gap {
                                v = max_value.get() - gap;
                            }
                            set_min_value.set(v);
                        }
                    }
                />
                <input
                    type="range"
                    min=min_limit
                    max=max_limit
                    step=step
                    prop:value=move || max_value.get().to_string()
                    on:input=move |ev| {
                        if let Ok(mut v) = event_target_value(&ev).parse::<u32>() {
                            if v < min_value.get() + gap {
                                v = min_value.get() + gap;
                            }
                            set_max_value.set(v);
                        }
                    }
                />
            </div>
        </div>
    }
}
