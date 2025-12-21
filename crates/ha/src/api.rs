//! Types for the Home Assistant WebSocket API messages.
//!
//! These are serde-enabled structs and enums that mirror the JSON
//! message format described at https://developers.home-assistant.io/docs/api/websocket

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Error object included in `result` messages when `success` is false.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ErrorInfo {
    pub code: String,
    pub message: String,
    #[serde(rename = "translation_key", skip_serializing_if = "Option::is_none")]
    pub translation_key: Option<String>,
    #[serde(rename = "translation_domain", skip_serializing_if = "Option::is_none")]
    pub translation_domain: Option<String>,
    #[serde(
        rename = "translation_placeholders",
        skip_serializing_if = "Option::is_none"
    )]
    pub translation_placeholders: Option<Value>,
}

/// Context object included in events and some results.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Context {
    pub id: String,
    #[serde(rename = "parent_id", skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    #[serde(rename = "user_id", skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
}

/// Event payload as sent by Home Assistant in `event` messages.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Event {
    pub data: Value,
    #[serde(rename = "event_type")]
    pub event_type: String,
    #[serde(rename = "time_fired", skip_serializing_if = "Option::is_none")]
    pub time_fired: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<Context>,
}

/// Messages sent from server to client (tagged by the `type` field).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    /// Sent when authentication is required.
    AuthRequired {
        ha_version: Option<String>,
    },
    /// Sent when authentication succeeded.
    AuthOk {
        ha_version: Option<String>,
    },
    /// Sent when authentication failed.
    AuthInvalid {
        message: Option<String>,
    },
    /// Generic result for command responses.
    Result {
        id: i64,
        success: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        result: Option<Value>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        error: Option<ErrorInfo>,
    },
    /// An event emitted by the event bus.
    Event {
        id: Option<i64>,
        event: Event,
    },
    /// Ping/pong messages used for heartbeats.
    Ping {
        id: Option<i64>,
    },
    Pong {
        id: Option<i64>,
    },
    /// Fallback for message types we don't model explicitly.
    #[serde(other)]
    Unknown,
}

/// Messages sent from client to server (tagged by the `type` field).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    /// Authenticate with an access token.
    Auth { access_token: String },
    /// Subscribe to events. `id` is normally added by the client when sending.
    SubscribeEvents {
        id: i64,
        #[serde(skip_serializing_if = "Option::is_none")]
        event_type: Option<String>,
    },
    /// Unsubscribe from events, referencing the original subscription id.
    UnsubscribeEvents { id: i64, subscription: i64 },
    /// Call a service.
    CallService {
        id: i64,
        domain: String,
        service: String,
        #[serde(rename = "service_data", skip_serializing_if = "Option::is_none")]
        service_data: Option<Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        target: Option<Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        return_response: Option<bool>,
    },
    /// Fire a raw event on the HA event bus.
    FireEvent {
        id: i64,
        event_type: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        event_data: Option<Value>,
    },
    /// Fetch current states
    GetStates { id: i64 },
    /// Fetch current config
    GetConfig { id: i64 },
    /// Fetch available services
    GetServices { id: i64 },
    /// Fetch registered panels
    GetPanels { id: i64 },
    /// Ping
    Ping { id: i64 },
    /// Pong
    Pong { id: i64 },
    /// Validate config
    ValidateConfig {
        id: i64,
        #[serde(skip_serializing_if = "Option::is_none")]
        trigger: Option<Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        condition: Option<Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        action: Option<Value>,
    },
    /// Extract from target
    ExtractFromTarget {
        id: i64,
        target: Value,
        #[serde(skip_serializing_if = "Option::is_none")]
        expand_group: Option<bool>,
    },
    /// Subscribe to a trigger
    SubscribeTrigger { id: i64, trigger: Value },
    /// Subscribe to a generic feature enablement message
    SupportedFeatures { id: i64, features: Value },
    /// Fallback for unhandled message types.
    #[serde(other)]
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auth_messages_roundtrip() {
        let auth = ClientMessage::Auth {
            access_token: "abc".into(),
        };
        let s = serde_json::to_string(&auth).unwrap();
        assert!(s.contains("auth"));
        let de: ClientMessage = serde_json::from_str(&s).unwrap();
        assert_eq!(de, auth);
    }

    #[test]
    fn event_deserialize() {
        let json =
            r#"{"id":18,"type":"event","event":{"data":{"foo":1},"event_type":"test_event"}}"#;
        let msg: ServerMessage = serde_json::from_str(json).unwrap();
        if let ServerMessage::Event { id, event } = msg {
            assert_eq!(id, Some(18));
            assert_eq!(event.event_type, "test_event");
        } else {
            panic!("expected event");
        }
    }
}
