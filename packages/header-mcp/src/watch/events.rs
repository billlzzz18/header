use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum ClientMessage {
    #[serde(rename = "join")]
    Join {
        path: String,
        #[serde(default)]
        is_wsl: bool,
    },
}

#[derive(Debug, Serialize, Clone)]
#[serde(tag = "type")]
pub enum ServerMessage {
    #[serde(rename = "ok")]
    Ok { path: String },

    #[serde(rename = "created")]
    Created { path: String },

    #[serde(rename = "modified")]
    Modified { path: String },

    #[serde(rename = "deleted")]
    Deleted { path: String },

    #[serde(rename = "renamed")]
    Renamed { from: String, to: String },

    #[serde(rename = "warning")]
    Warning { message: String },

    #[serde(rename = "error")]
    Error { message: String },

    #[serde(rename = "terminated")]
    Terminated { reason: String },
}

impl ServerMessage {
    pub fn to_json_string(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| {
            r#"{\"type\":\"error\",\"message\":\"Serialization failed\"}"#.to_string()
        })
    }
}
