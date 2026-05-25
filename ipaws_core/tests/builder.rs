//! Tests for the fluent builder API and error-response parsers.

use ipaws_core::*;

#[test]
fn defaults_produce_valid_uuid_identifier() {
    let alert = CapAlertBuilder::new().build();
    assert_eq!(alert.identifier.len(), 36);  // standard UUID length
    assert_eq!(alert.identifier.matches('-').count(), 4);
}

#[test]
fn each_alert_gets_unique_identifier() {
    let a = CapAlertBuilder::new().build();
    let b = CapAlertBuilder::new().build();
    assert_ne!(a.identifier, b.identifier);
}

#[test]
fn defaults_to_ipaws_v1_0_code() {
    let alert = CapAlertBuilder::new().build();
    assert!(alert.codes.contains(&"IPAWSv1.0".to_string()));
}

#[test]
fn additional_codes_can_be_added() {
    let alert = CapAlertBuilder::new().add_code("MY-COG-001").build();
    assert!(alert.codes.contains(&"IPAWSv1.0".to_string()));
    assert!(alert.codes.contains(&"MY-COG-001".to_string()));
}

#[test]
fn references_use_cap_csv_format() {
    use chrono::{TimeZone, Utc};
    let sent = Utc.with_ymd_and_hms(2026, 5, 24, 18, 0, 0).unwrap();
    let alert = CapAlertBuilder::new()
        .add_reference("sender@example.gov", "ALERT-001", sent)
        .build();
    assert_eq!(alert.references.len(), 1);
    let r = &alert.references[0];
    let parts: Vec<&str> = r.split(',').collect();
    assert_eq!(parts.len(), 3);
    assert_eq!(parts[0], "sender@example.gov");
    assert_eq!(parts[1], "ALERT-001");
    assert!(parts[2].starts_with("2026-05-24T18:00:00"));
}

#[test]
fn add_same_code_creates_default_area_if_none_exists() {
    let alert = CapAlertBuilder::new()
        .add_info(|i| i.with_event("Test").add_same_code("048303"))
        .build();
    assert_eq!(alert.infos[0].areas.len(), 1);
    assert_eq!(alert.infos[0].areas[0].same_codes, vec!["048303"]);
}

#[test]
fn channel_routing_calls_are_additive() {
    let alert = CapAlertBuilder::new()
        .add_info(|i| i
            .with_event("Test")
            .add_wea_routing(Some("hi"), Some("longer"))
            .add_eas_routing()
            .add_nwem_routing(Some("/X.NEW.KOAX/"), Some("OHC001")))
        .build();
    let params = &alert.infos[0].parameters;
    assert!(params.iter().any(|p| p.name == "WEAHandling"));
    assert!(params.iter().any(|p| p.name == "EASHandling"));
    assert!(params.iter().any(|p| p.name == "NWEMHandling"));
    assert!(params.iter().any(|p| p.name == "CMAMtext"));
    assert!(params.iter().any(|p| p.name == "CMAMlongtext"));
    assert!(params.iter().any(|p| p.name == "VTEC"));
    assert!(params.iter().any(|p| p.name == "UGC"));
}
