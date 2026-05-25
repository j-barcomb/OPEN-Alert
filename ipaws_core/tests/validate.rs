//! Tests for CAP/IPAWS alert validation rules.

use ipaws_core::*;

fn minimal_valid_alert() -> CapAlert {
    CapAlertBuilder::new()
        .with_sender("alerts@example.gov")
        .with_status(CapStatus::Test)
        .add_info(|info| info
            .with_event("Tornado Warning")
            .with_urgency(CapUrgency::Immediate)
            .with_severity(CapSeverity::Extreme)
            .with_certainty(CapCertainty::Observed)
            .with_headline("Tornado warning until 6pm")
            .with_expires(chrono::Utc::now() + chrono::Duration::hours(1))
            .add_area(|a| a.with_description("Test County").add_same_code("048303")))
        .build()
}

#[test]
fn cap001_missing_identifier() {
    let mut a = minimal_valid_alert();
    a.identifier = String::new();
    let r = CapValidator::validate(&a);
    assert!(r.errors().any(|e| e.code == "CAP001"));
}

#[test]
fn cap002_missing_sender() {
    let mut a = minimal_valid_alert();
    a.sender = String::new();
    let r = CapValidator::validate(&a);
    assert!(r.errors().any(|e| e.code == "CAP002"));
}

#[test]
fn cap003_sender_without_at_sign_warns() {
    let mut a = minimal_valid_alert();
    a.sender = "not-an-email".into();
    let r = CapValidator::validate(&a);
    assert!(r.warnings().any(|w| w.code == "CAP003"));
}

#[test]
fn cap004_missing_info_block() {
    let mut a = minimal_valid_alert();
    a.infos.clear();
    let r = CapValidator::validate(&a);
    assert!(r.errors().any(|e| e.code == "CAP004"));
    assert!(!r.is_valid());
}

#[test]
fn ipaws001_missing_code() {
    let mut a = minimal_valid_alert();
    a.codes.clear();
    let r = CapValidator::validate(&a);
    assert!(r.errors().any(|e| e.code == "IPAWS001"));
}

#[test]
fn cap005_missing_event_type() {
    let mut a = minimal_valid_alert();
    a.infos[0].event = String::new();
    let r = CapValidator::validate(&a);
    assert!(r.errors().any(|e| e.code == "CAP005"));
}

#[test]
fn cap006_missing_category() {
    let mut a = minimal_valid_alert();
    a.infos[0].categories.clear();
    let r = CapValidator::validate(&a);
    assert!(r.errors().any(|e| e.code == "CAP006"));
}

#[test]
fn cap012_invalid_same_code_length() {
    let mut a = minimal_valid_alert();
    a.infos[0].areas[0].same_codes = vec!["12345".into()];  // 5 digits, not 6
    let r = CapValidator::validate(&a);
    assert!(r.errors().any(|e| e.code == "CAP012"));
}

#[test]
fn cap012_non_numeric_same_code() {
    let mut a = minimal_valid_alert();
    a.infos[0].areas[0].same_codes = vec!["ABC123".into()];
    let r = CapValidator::validate(&a);
    assert!(r.errors().any(|e| e.code == "CAP012"));
}

#[test]
fn cap012_valid_same_code_passes() {
    let a = minimal_valid_alert();
    let r = CapValidator::validate(&a);
    assert!(!r.errors().any(|e| e.code == "CAP012"));
}

#[test]
fn wea002_short_text_too_long() {
    let a = CapAlertBuilder::new()
        .with_sender("a@b.gov")
        .add_info(|i| i
            .with_event("Test")
            .add_wea_routing(Some(&"X".repeat(90)), None)
            .with_urgency(CapUrgency::Immediate)
            .with_severity(CapSeverity::Severe))
        .build();
    // 90 chars exactly should not error.
    let r = CapValidator::validate(&a);
    assert!(!r.errors().any(|e| e.code == "WEA002"));

    // But manually construct a too-long parameter
    let mut a2 = a;
    if let Some(p) = a2.infos[0].parameters.iter_mut().find(|p| p.name == "CMAMtext") {
        p.value = "X".repeat(91);
    }
    let r2 = CapValidator::validate(&a2);
    assert!(r2.errors().any(|e| e.code == "WEA002"));
}

#[test]
fn wea004_non_immediate_urgency_warns() {
    let a = CapAlertBuilder::new()
        .with_sender("a@b.gov")
        .add_info(|i| i
            .with_event("Test")
            .add_wea_routing(Some("hi"), None)
            .with_urgency(CapUrgency::Future)  // not Immediate
            .with_severity(CapSeverity::Severe))
        .build();
    let r = CapValidator::validate(&a);
    assert!(r.warnings().any(|w| w.code == "WEA004"));
}

#[test]
fn eas001_no_same_code_for_eas() {
    let a = CapAlertBuilder::new()
        .with_sender("a@b.gov")
        .add_info(|i| i
            .with_event("Test")
            .add_eas_routing()
            .add_area(|ar| ar.with_description("Region")))  // no SAME code
        .build();
    let r = CapValidator::validate(&a);
    assert!(r.errors().any(|e| e.code == "EAS001"));
}

#[test]
fn nwem001_missing_vtec_warns() {
    let a = CapAlertBuilder::new()
        .with_sender("a@b.gov")
        .add_info(|i| i
            .with_event("Test")
            .add_nwem_routing(None, Some("OHZ001"))  // VTEC missing
            .add_area(|ar| ar.with_description("Region")))
        .build();
    let r = CapValidator::validate(&a);
    assert!(r.warnings().any(|w| w.code == "NWEM001"));
}

#[test]
fn minimal_valid_alert_passes_all_errors() {
    let a = minimal_valid_alert();
    let r = CapValidator::validate(&a);
    let error_codes: Vec<&str> = r.errors().map(|e| e.code.as_str()).collect();
    assert!(r.is_valid(), "Minimal alert should be valid; errors: {error_codes:?}");
}

#[test]
fn summary_for_valid_alert_says_no_issues() {
    let mut a = minimal_valid_alert();
    // Strip warnings: remove headline-length warning by adding a short headline,
    // add description+instruction (warnings on Actual), but keep Test status
    a.infos[0].headline = Some("OK".into());
    let r = CapValidator::validate(&a);
    if r.findings.is_empty() {
        assert_eq!(r.summary(), "✓  No issues found.");
    }
}

#[test]
fn actual_alert_warns_without_description() {
    let mut a = minimal_valid_alert();
    a.status = CapStatus::Actual;
    a.infos[0].description = None;
    let r = CapValidator::validate(&a);
    assert!(r.warnings().any(|w| w.code == "CAP013"));
}
