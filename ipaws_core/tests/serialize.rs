//! Tests for CAP v1.2 XML serialization.

use chrono::{TimeZone, Utc};
use ipaws_core::*;

fn fixed_time() -> chrono::DateTime<chrono::Utc> {
    Utc.with_ymd_and_hms(2026, 5, 24, 18, 30, 0).unwrap()
}

fn sample_alert() -> CapAlert {
    let mut alert = CapAlertBuilder::new()
        .with_sender("alerts@example.gov")
        .with_status(CapStatus::Test)
        .with_msg_type(CapMsgType::Alert)
        .with_scope(CapScope::Public)
        .add_info(|info| info
            .with_event("Tornado Warning")
            .with_urgency(CapUrgency::Immediate)
            .with_severity(CapSeverity::Extreme)
            .with_certainty(CapCertainty::Observed)
            .with_headline("Tornado Warning until 6:00 PM")
            .with_description("A confirmed tornado is on the ground.")
            .with_instruction("Take shelter immediately.")
            .add_wea_routing(Some("Tornado Warning until 6PM"), None)
            .add_eas_routing()
            .add_area(|a| a
                .with_description("Lubbock County, TX")
                .add_same_code("048303")))
        .build();
    alert.identifier = "TEST-1234".into();
    alert.sent = fixed_time();
    alert
}

#[test]
fn includes_xml_prolog() {
    let xml = ipaws_core::serialize(&sample_alert(), false);
    assert!(xml.starts_with(r#"<?xml version="1.0" encoding="UTF-8"?>"#));
}

#[test]
fn declares_cap_1_2_namespace() {
    let xml = ipaws_core::serialize(&sample_alert(), false);
    assert!(xml.contains(r#"xmlns="urn:oasis:names:tc:emergency:cap:1.2""#));
}

#[test]
fn emits_required_top_level_elements_in_order() {
    let xml = ipaws_core::serialize(&sample_alert(), false);
    let order = ["<identifier>", "<sender>", "<sent>", "<status>", "<msgType>", "<scope>", "<code>", "<info>"];
    let mut last_idx = 0;
    for tag in order {
        let idx = xml.find(tag).unwrap_or_else(|| panic!("Missing tag {tag}"));
        assert!(idx >= last_idx, "Tag {tag} out of order");
        last_idx = idx;
    }
}

#[test]
fn includes_ipaws_v1_0_code() {
    let xml = ipaws_core::serialize(&sample_alert(), false);
    assert!(xml.contains("<code>IPAWSv1.0</code>"));
}

#[test]
fn formats_sent_timestamp_correctly() {
    let xml = ipaws_core::serialize(&sample_alert(), false);
    assert!(xml.contains("<sent>2026-05-24T18:30:00+00:00</sent>"));
}

#[test]
fn includes_event_and_severity() {
    let xml = ipaws_core::serialize(&sample_alert(), false);
    assert!(xml.contains("<event>Tornado Warning</event>"));
    assert!(xml.contains("<severity>Extreme</severity>"));
    assert!(xml.contains("<urgency>Immediate</urgency>"));
    assert!(xml.contains("<certainty>Observed</certainty>"));
}

#[test]
fn escapes_xml_special_characters() {
    let alert = CapAlertBuilder::new()
        .with_sender("a@b.gov")
        .add_info(|i| i
            .with_event("Test")
            .with_headline(r#"Floods <bad> & "danger" 'ahead'"#)
            .add_area(|a| a.with_description("Area")))
        .build();
    let xml = ipaws_core::serialize(&alert, false);
    assert!(xml.contains("&lt;bad&gt;"));
    assert!(xml.contains("&amp;"));
    assert!(xml.contains("&quot;"));
    assert!(xml.contains("&apos;"));
    // And the raw characters must NOT appear in the headline content
    assert!(!xml.contains("<bad>"));
}

#[test]
fn wea_routing_adds_expected_parameters() {
    let xml = ipaws_core::serialize(&sample_alert(), false);
    assert!(xml.contains("<valueName>WEAHandling</valueName>"));
    assert!(xml.contains("<valueName>CMAMtext</valueName>"));
}

#[test]
fn eas_routing_adds_expected_parameter() {
    let xml = ipaws_core::serialize(&sample_alert(), false);
    assert!(xml.contains("<valueName>EASHandling</valueName>"));
}

#[test]
fn same_code_renders_as_geocode_block() {
    let xml = ipaws_core::serialize(&sample_alert(), false);
    assert!(xml.contains("<geocode>"));
    assert!(xml.contains("<valueName>SAME</valueName>"));
    assert!(xml.contains("<value>048303</value>"));
}

#[test]
fn wea_short_text_truncated_to_90_chars() {
    let long = "X".repeat(200);
    let alert = CapAlertBuilder::new()
        .with_sender("a@b.gov")
        .add_info(|i| i
            .with_event("Test")
            .add_wea_routing(Some(&long), None)
            .add_area(|a| a.with_description("Area")))
        .build();
    let xml = ipaws_core::serialize(&alert, false);
    // Extract the CMAMtext value
    let idx = xml.find("<valueName>CMAMtext</valueName>").expect("CMAMtext present");
    let after = &xml[idx..];
    let val_start = after.find("<value>").expect("value tag") + "<value>".len();
    let val_end = after[val_start..].find("</value>").expect("end value tag");
    let value = &after[val_start..val_start + val_end];
    assert_eq!(value.len(), 90, "WEA short text not truncated; got {} chars", value.len());
}

#[test]
fn wea_long_text_truncated_to_360_chars() {
    let long = "Y".repeat(500);
    let alert = CapAlertBuilder::new()
        .with_sender("a@b.gov")
        .add_info(|i| i
            .with_event("Test")
            .add_wea_routing(None, Some(&long))
            .add_area(|a| a.with_description("Area")))
        .build();
    let xml = ipaws_core::serialize(&alert, false);
    let idx = xml.find("<valueName>CMAMlongtext</valueName>").expect("CMAMlongtext present");
    let after = &xml[idx..];
    let val_start = after.find("<value>").expect("value tag") + "<value>".len();
    let val_end = after[val_start..].find("</value>").expect("end value tag");
    let value = &after[val_start..val_start + val_end];
    assert_eq!(value.len(), 360);
}

#[test]
fn multiple_info_blocks_supported() {
    let alert = CapAlertBuilder::new()
        .with_sender("a@b.gov")
        .add_info(|i| i.with_event("Tornado Warning"))
        .add_info(|i| i.with_event("Advertencia de Tornado"))
        .build();
    let xml = ipaws_core::serialize(&alert, true);
    let count = xml.matches("<info>").count();
    assert_eq!(count, 2);
}

#[test]
fn pretty_and_compact_have_identical_content() {
    let alert = sample_alert();
    let pretty  = ipaws_core::serialize(&alert, true);
    let compact = ipaws_core::serialize(&alert, false);
    // Strip whitespace from pretty and compare to compact
    let strip_ws: fn(&str) -> String = |s| s.chars().filter(|c| !c.is_whitespace()).collect();
    assert_eq!(strip_ws(&pretty), strip_ws(&compact));
}
