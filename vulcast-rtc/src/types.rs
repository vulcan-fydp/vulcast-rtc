//! Wrappers around signalling types that should be passed opaquely to WebRTC.
//! Can be used as types for GraphQL client as well.

use std::str::FromStr;

use derive_more::{From, Into};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, From, Into)]
pub struct TransportId(String);

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, From, Into)]
pub struct ProducerId(String);

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, From, Into)]
pub struct DataProducerId(String);
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, From, Into)]
pub struct DataConsumerId(String);

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MediaKind {
    Audio,
    Video,
}
impl FromStr for MediaKind {
    type Err = ();
    fn from_str(input: &str) -> Result<MediaKind, Self::Err> {
        match input {
            "audio" => Ok(MediaKind::Audio),
            "video" => Ok(MediaKind::Video),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, From, Into)]
pub struct RtpParameters(serde_json::Value);
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, From, Into)]
pub struct RtpCapabilities(serde_json::Value);
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, From, Into)]
pub struct RtpCapabilitiesFinalized(serde_json::Value);
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, From, Into)]
pub struct WebRtcTransportOptions(serde_json::Value);
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, From, Into)]
pub struct DtlsParameters(serde_json::Value);

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, From, Into)]
#[serde(rename_all = "camelCase")]
pub struct DataConsumerOptions {
    pub id: DataConsumerId,
    pub data_producer_id: DataProducerId,
    pub sctp_stream_parameters: serde_json::Value,
}
