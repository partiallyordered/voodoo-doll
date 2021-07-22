pub mod protocol;
pub const MANIFEST_JSON: &str = include_str!("../kubernetes/pod.json");

// TODO:
// - test, make sure this works, then users can unwrap() safely
// - Consider instead incorporating the manifest here with serde_json! as in the example here:
//   https://docs.rs/kube/0.58.0/kube/#example
//   This would let us use the Cargo version directly in the manifest.
pub fn pod<T: for<'de> serde::Deserialize<'de>>() -> Result<T, serde_json::Error> {
    serde_json::from_value(serde_json::from_str(MANIFEST_JSON)?)
}
