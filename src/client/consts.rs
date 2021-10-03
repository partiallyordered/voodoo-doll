use serde_json::json;
use lazy_static::lazy_static;

pub const LABEL_NAME: &'static str = "app.kubernetes.io/name";
pub const LABEL_VALUE: &'static str = "voodoo-doll";
pub const CONTAINER_NAME: &'static str = "app";
pub const CONTAINER_PORT: i32 = 3030;

lazy_static! {
    // static ref POD: Regex = Regex::new(r".*Settlement model.*already exists.*").unwrap();
    pub static ref POD: serde_json::Value = json!(
        {
          "apiVersion": "v1",
          "kind": "Pod",
          "metadata": {
            "name": "voodoo-doll",
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
// pub const POD: serde_json::Value = json!(
//     {
//       "apiVersion": "v1",
//       "kind": "Pod",
//       "metadata": {
//         "name": "voodoo-doll",
//         "labels": {
//           LABEL_NAME: LABEL_VALUE
//         }
//       },
//       "spec": {
//         "containers": [
//           {
//             "name": CONTAINER_NAME,
//             "image": format!("ghcr.io/partiallyordered/voodoo-doll:{}", env!("CARGO_PKG_VERSION")),
//             "ports": [
//               {
//                 "containerPort": CONTAINER_PORT
//               }
//             ],
//             "env": [
//               {
//                 "name": "HOST_IP",
//                 "valueFrom": {
//                   "fieldRef": {
//                     "fieldPath": "status.podIP"
//                   }
//                 }
//               }
//             ]
//           }
//         ]
//       }
//     }
// );
