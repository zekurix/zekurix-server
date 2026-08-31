use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

fn bundle_openapi(output: &Path) {
    let workspace = env::current_dir().expect("failed to get current directory");
    let volume = format!("{}:/workspace", workspace.to_string_lossy());
    let relative_output = output
        .strip_prefix(&workspace)
        .expect("OUT_DIR must be inside workspace");
    let docker_output = format!("/workspace/{}", relative_output.display());

    let status = Command::new("docker")
        .arg("run")
        .arg("--rm")
        .arg("-v")
        .arg(&volume)
        .arg("-w")
        .arg("/workspace")
        .arg("redocly/cli")
        .arg("bundle")
        .arg("openapi/openapi.yaml")
        .arg("-o")
        .arg(&docker_output)
        .status()
        .expect("failed to start docker");

    assert!(
        status.success(),
        "Redocly failed to bundle OpenAPI specification"
    );
    assert!(
        output.exists(),
        "OpenAPI bundle was not generated: {}",
        output.display()
    );
}

fn minify_openapi(input: &Path, output: &Path) {
    let json = fs::read_to_string(input).expect("failed to read bundled OpenAPI");
    let minified: serde_json::Value = serde_json::from_str(&json).expect("invalid bundled OpenAPI");

    fs::write(
        output,
        serde_json::to_string(&minified).expect("failed to minify OpenAPI"),
    )
    .expect("failed to write minified OpenAPI");

    assert!(
        output.exists(),
        "OpenAPI file was not generated: {}",
        output.display()
    );
}

fn main() {
    println!("cargo:rerun-if-changed=openapi");

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
    let openapi_bundle_json = out_dir.join("openapi.bundle.json");
    let openapi_min_json = out_dir.join("openapi.min.json");

    bundle_openapi(&openapi_bundle_json);
    minify_openapi(&openapi_bundle_json, &openapi_min_json);

    println!(
        "cargo:rustc-env=OPENAPI_JSON_PATH={}",
        openapi_min_json.display()
    );
}
