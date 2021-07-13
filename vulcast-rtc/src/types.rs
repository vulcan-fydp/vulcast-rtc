//! Wrappers around signalling types that should be passed opaquely to WebRTC.
//! Can be used as types for GraphQL client as well.

use std::str::FromStr;

use derive_more::{From, Into};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize, From, Into)]
pub struct TransportId(String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize, From, Into)]
pub struct ProducerId(String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize, From, Into)]
pub struct DataProducerId(String);
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize, From, Into)]
pub struct DataConsumerId(String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub enum MediaKind {
    #[serde(rename = "audio")]
    Audio,
    #[serde(rename = "video")]
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize, From, Into)]
pub struct RtpParameters(String);
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize, From, Into)]
pub struct RtpCapabilities(String);
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize, From, Into)]
pub struct RtpCapabilitiesFinalized(String);
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize, From, Into)]
pub struct WebRtcTransportOptions(String);
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize, From, Into)]
pub struct DtlsParameters(String);
