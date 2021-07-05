pub mod protocol;
pub const MANIFEST_JSON: &str = include_str!("manifest.json");

// TODO: test, make sure this works, then users can unwrap() safely
pub fn pod<T: for<'de> serde::Deserialize<'de>>() -> Result<T, serde_json::Error> {
    serde_json::from_value(serde_json::from_str(MANIFEST_JSON)?)
}
