use serde_json::json;
use lazy_static::lazy_static;

pub const LABEL_NAME: &'static str = "app.kubernetes.io/name";
pub const LABEL_VALUE: &'static str = "voodoo-doll";
pub const CONTAINER_NAME: &'static str = "app";
pub const CONTAINER_PORT: i32 = 3030;
pub const POD_NAME: &'static str = "voodoo-doll";

lazy_static! {
    pub static ref POD_JSON: serde_json::Value = json!(
        {
          "apiVersion": "v1",
          "kind": "Pod",
          "metadata": {
            "name": POD_NAME,
            "labels": {
              LABEL_NAME: LABEL_VALUE
            }
          },
          "spec": {
            "containers": [
              {
                "name": CONTAINER_NAME,
                "image": format!("ghcr.io/partiallyordered/voodoo-doll:{}", env!("CARGO_PKG_VERSION")),
                "ports": [
                  {
                    "containerPort": CONTAINER_PORT
                  }
                ],
                "env": [
                  {
                    "name": "HOST_IP",
                    "valueFrom": {
                      "fieldRef": {
                        "fieldPath": "status.podIP"
                      }
                    }
                  }
                ]
              }
            ]
          }
        }
    );
}
