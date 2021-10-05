use serde_json::json;
use lazy_static::lazy_static;

pub const LABEL_NAME: &'static str = "app.kubernetes.io/name";
pub const LABEL_VALUE: &'static str = "voodoo-doll";
pub const CONTAINER_NAME: &'static str = "app";
pub const CONTAINER_PORT: i32 = 3030;
pub const POD_NAME: &'static str = "voodoo-doll";
pub const BINDING_NAME: &'static str = "voodoo-doll-pod-access";

lazy_static! {
    pub static ref SERVICEACCOUNT_JSON: serde_json::Value = json!(
        {
          "apiVersion": "v1",
          "kind": "ServiceAccount",
          "metadata": {
            "name": POD_NAME
          }
        }
    );

    pub static ref CLUSTERROLEBINDING_JSON: serde_json::Value = json!(
        {
          "apiVersion": "rbac.authorization.k8s.io/v1",
          "kind": "ClusterRoleBinding",
          "metadata": {
            "name": BINDING_NAME
          },
          "subjects": [
            {
              "kind": "ServiceAccount",
              "name": POD_NAME,
              "namespace": "default"
            }
          ],
          "roleRef": {
            "kind": "ClusterRole",
            "name": BINDING_NAME,
            "apiGroup": "rbac.authorization.k8s.io"
          }
        }
    );

    pub static ref CLUSTERROLE_JSON: serde_json::Value = json!(
        {
          "apiVersion": "rbac.authorization.k8s.io/v1",
          "kind": "ClusterRole",
          "metadata": {
            "name": BINDING_NAME
          },
          "rules": [
            {
              "apiGroups": [ "" ],
              "resources": [ "pods" ],
              "verbs": [ "get", "watch", "list" ]
            },
            {
              "apiGroups": [ "" ],
              "resources": [ "pods/portforward" ],
              "verbs": [ "create", "update", "delete", "watch", "get", "list" ]
            }
          ]
        }
    );

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
            "serviceAccountName": "voodoo-doll",
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
