use std::env;
use std::fs;

fn main() {
    println!("cargo:rerun-if-changed=.env");
    dotenvy::dotenv().ok();
    let url = env::var("API_BASE_URL").expect("API_BASE_URL must be set");

    let content = format!(
        "// auto-generated\npub const API_BASE_URL: &str = \"{}\";\n",
        url.replace('"', "\\\"")
    );

    fs::write("src/env.rs", content).unwrap();
}
