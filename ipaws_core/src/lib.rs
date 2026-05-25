pub mod builder;
pub mod channels;
#[cfg(feature = "client")]
pub mod client;
pub mod models;
pub mod validator;
pub mod xml;
#[cfg(feature = "client")]
pub mod winstore;

// Convenient re-exports
pub use builder::{AlertAreaBuilder, AlertInfoBuilder, CapAlertBuilder};
pub use channels::IpawsConfig;
#[cfg(feature = "client")]
pub use client::{ClientError, IpawsClient, IpawsResponse};
pub use models::*;
pub use validator::{CapValidator, FindingSeverity, ValidationFinding, ValidationResult};
pub use xml::serialize;
