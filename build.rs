use std::{env, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=openapi");

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));

    let openapi_json = out_dir.join("openapi.json");

    let yaml = fs::read_to_string("openapi/openapi.yaml").expect("failed to read openapi.yaml");

    let document: yaml_serde::Value = yaml_serde::from_str(&yaml).expect("invalid OpenAPI YAML");

    let json = serde_json::to_string(&document).expect("failed to convert OpenAPI to JSON");

    fs::write(&openapi_json, json).expect("failed to write openapi.json");

    println!(
        "cargo:rustc-env=OPENAPI_JSON_PATH={}",
        openapi_json.display()
    );
}
