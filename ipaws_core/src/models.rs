use chrono::{DateTime, Utc};
use std::fmt;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Default)]
pub enum CapStatus { Actual, Exercise, System, #[default] Test, Draft }

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Default)]
pub enum CapMsgType { #[default] Alert, Update, Cancel, Ack, Error }

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Default)]
pub enum CapScope { #[default] Public, Restricted, Private }

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Default)]
pub enum CapCategory { Geo, #[default] Met, Safety, Security, Rescue, Fire,
                       Health, Env, Transport, Infra, CBRNE, Other }

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Default)]
pub enum CapUrgency { #[default] Immediate, Expected, Future, Past, Unknown }

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Default)]
pub enum CapSeverity { Extreme, #[default] Severe, Moderate, Minor, Unknown }

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Default)]
pub enum CapCertainty { #[default] Observed, Likely, Possible, Unlikely, Unknown }

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum CapResponseType { Shelter, Evacuate, Prepare, Execute, Avoid,
                           Monitor, Assess, AllClear, None }

macro_rules! display_debug {
    ($t:ty) => {
        impl fmt::Display for $t {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{:?}", self) }
        }
    };
}
display_debug!(CapStatus);
display_debug!(CapMsgType);
display_debug!(CapScope);
display_debug!(CapCategory);
display_debug!(CapUrgency);
display_debug!(CapSeverity);
display_debug!(CapCertainty);
display_debug!(CapResponseType);

#[derive(Debug, Clone, Default)]
pub struct CapParameter { pub name: String, pub value: String }

#[derive(Debug, Clone, Default)]
pub struct AlertArea {
    pub area_desc:  String,
    pub same_codes: Vec<String>,
    pub polygons:   Vec<String>,
    pub circles:    Vec<String>,
}

#[derive(Debug, Clone)]
pub struct AlertInfo {
    pub language:       String,
    pub categories:     Vec<CapCategory>,
    pub event:          String,
    pub response_types: Vec<CapResponseType>,
    pub urgency:        CapUrgency,
    pub severity:       CapSeverity,
    pub certainty:      CapCertainty,
    pub effective:      Option<DateTime<Utc>>,
    pub onset:          Option<DateTime<Utc>>,
    pub expires:        Option<DateTime<Utc>>,
    pub sender_name:    Option<String>,
    pub headline:       Option<String>,
    pub description:    Option<String>,
    pub instruction:    Option<String>,
    pub web:            Option<String>,
    pub parameters:     Vec<CapParameter>,
    pub areas:          Vec<AlertArea>,
}

impl Default for AlertInfo {
    fn default() -> Self {
        Self {
            language: "en-US".into(), categories: vec![CapCategory::Met],
            event: String::new(), response_types: Vec::new(),
            urgency: CapUrgency::default(), severity: CapSeverity::default(),
            certainty: CapCertainty::default(),
            effective: None, onset: None, expires: None,
            sender_name: None, headline: None, description: None,
            instruction: None, web: None,
            parameters: Vec::new(), areas: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CapAlert {
    pub identifier: String,
    pub sender:     String,
    pub sent:       DateTime<Utc>,
    pub status:     CapStatus,
    pub msg_type:   CapMsgType,
    pub scope:      CapScope,
    pub references: Vec<String>,
    pub codes:      Vec<String>,
    pub infos:      Vec<AlertInfo>,
}

impl Default for CapAlert {
    fn default() -> Self {
        Self {
            identifier: uuid::Uuid::new_v4().to_string(),
            sender:     String::new(),
            sent:       Utc::now(),
            status:     CapStatus::default(),
            msg_type:   CapMsgType::default(),
            scope:      CapScope::default(),
            references: Vec::new(),
            codes:      vec!["IPAWSv1.0".into()],
            infos:      Vec::new(),
        }
    }
}
