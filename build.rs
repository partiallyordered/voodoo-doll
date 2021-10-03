
use serde_json::json;
use std::fs;
use std::path::Path;
#[path = "src/client/consts.rs"] mod client_consts;

fn main() {
    let pod_str = client_consts::POD.to_string();

    let manifest_json = Path::new("kubernetes/pod.json");

    fs::write(
        &manifest_json,
        pod_str,
    ).unwrap();
}
