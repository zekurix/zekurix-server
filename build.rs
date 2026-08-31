fn main() {
    println!("cargo:rerun-if-changed=openapi/generated/openapi.min.json");
}
