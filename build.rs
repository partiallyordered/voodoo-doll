use std::fs;
use std::path::Path;
#[path = "src/client/consts.rs"] mod client_consts;

fn main() {
    let deploy_manifest = client_consts::POD_JSON.to_string();
    let serviceaccount_manifest = client_consts::SERVICEACCOUNT_JSON.to_string();
    let clusterrole_manifest = client_consts::CLUSTERROLE_JSON.to_string();
    let clusterrolebinding_manifest = client_consts::CLUSTERROLEBINDING_JSON.to_string();

    let deploy_fname = Path::new("kubernetes/deploy.json");
    let serviceaccount_fname = Path::new("kubernetes/serviceaccount.json");
    let clusterrole_fname = Path::new("kubernetes/clusterrole.json");
    let clusterrolebinding_fname = Path::new("kubernetes/clusterrolebinding.json");

    fs::write(&deploy_fname, deploy_manifest).unwrap();
    fs::write(&serviceaccount_fname, serviceaccount_manifest).unwrap();
    fs::write(&clusterrole_fname, clusterrole_manifest).unwrap();
    fs::write(&clusterrolebinding_fname, clusterrolebinding_manifest).unwrap();
}
