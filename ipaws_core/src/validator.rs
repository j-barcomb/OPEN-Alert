use crate::models::*;

#[derive(Debug, Clone, PartialEq)]
pub enum FindingSeverity { Error, Warning }

#[derive(Debug, Clone)]
pub struct ValidationFinding {
    pub severity: FindingSeverity,
    pub code:     String,
    pub message:  String,
}

impl ValidationFinding {
    fn err(code: &str, msg: impl Into<String>) -> Self {
        Self { severity: FindingSeverity::Error, code: code.into(), message: msg.into() }
    }
    fn warn(code: &str, msg: impl Into<String>) -> Self {
        Self { severity: FindingSeverity::Warning, code: code.into(), message: msg.into() }
    }
}

#[derive(Debug, Default)]
pub struct ValidationResult { pub findings: Vec<ValidationFinding> }

impl ValidationResult {
    pub fn is_valid(&self) -> bool {
        !self.findings.iter().any(|f| f.severity == FindingSeverity::Error)
    }
    pub fn has_warnings(&self) -> bool {
        self.findings.iter().any(|f| f.severity == FindingSeverity::Warning)
    }
    pub fn errors(&self) -> impl Iterator<Item = &ValidationFinding> {
        self.findings.iter().filter(|f| f.severity == FindingSeverity::Error)
    }
    pub fn warnings(&self) -> impl Iterator<Item = &ValidationFinding> {
        self.findings.iter().filter(|f| f.severity == FindingSeverity::Warning)
    }
    pub fn summary(&self) -> String {
        if self.findings.is_empty() { return "✓  No issues found.".into(); }
        self.findings.iter()
            .map(|f| format!("[{}] {}: {}",
                if f.severity == FindingSeverity::Error { "ERR" } else { "WARN" },
                f.code, f.message))
            .collect::<Vec<_>>().join("\n")
    }
}

pub struct CapValidator;

impl CapValidator {
    pub fn validate(alert: &CapAlert) -> ValidationResult {
        let mut r = ValidationResult::default();
        let f = &mut r.findings;

        if alert.identifier.is_empty()  { f.push(ValidationFinding::err("CAP001", "Alert identifier is required.")); }
        if alert.sender.is_empty()      { f.push(ValidationFinding::err("CAP002", "Sender address is required.")); }
        else if !alert.sender.contains('@') { f.push(ValidationFinding::warn("CAP003", "Sender should be a valid email address.")); }
        if alert.infos.is_empty()       { f.push(ValidationFinding::err("CAP004", "At least one <info> block is required.")); return r; }
        if !alert.codes.iter().any(|c| c == "IPAWSv1.0") {
            f.push(ValidationFinding::err("IPAWS001", "Alert must include the 'IPAWSv1.0' code element."));
        }

        for (i, info) in alert.infos.iter().enumerate() {
            let p = if alert.infos.len() > 1 { format!("Info[{i}]: ") } else { String::new() };

            if info.event.is_empty()       { f.push(ValidationFinding::err("CAP005", format!("{p}Event type is required."))); }
            if info.categories.is_empty()  { f.push(ValidationFinding::err("CAP006", format!("{p}At least one category is required."))); }
            if info.expires.is_none()      { f.push(ValidationFinding::warn("CAP007", format!("{p}Expiry time is recommended."))); }
            if info.headline.is_none()     { f.push(ValidationFinding::warn("CAP009", format!("{p}Headline is recommended."))); }
            else if let Some(h) = &info.headline {
                if h.len() > 160 { f.push(ValidationFinding::warn("CAP008", format!("{p}Headline exceeds 160 characters."))); }
            }
            if info.areas.is_empty() {
                f.push(ValidationFinding::warn("CAP010", format!("{p}At least one geographic area is recommended.")));
            } else {
                for area in &info.areas {
                    if area.area_desc.is_empty() { f.push(ValidationFinding::warn("CAP011", format!("{p}Area description is recommended."))); }
                    for code in &area.same_codes {
                        if code.len() != 6 || !code.chars().all(|c| c.is_ascii_digit()) {
                            f.push(ValidationFinding::err("CAP012", format!("{p}SAME code '{code}' must be exactly 6 digits.")));
                        }
                    }
                }
            }

            let has_wea  = info.parameters.iter().any(|p| p.name == "WEAHandling");
            let has_eas  = info.parameters.iter().any(|p| p.name == "EASHandling");
            let has_nwem = info.parameters.iter().any(|p| p.name == "NWEMHandling");

            if has_wea {
                let short = info.parameters.iter().find(|p| p.name == "CMAMtext");
                if short.is_none() { f.push(ValidationFinding::warn("WEA001", format!("{p}WEA short text (CMAMtext) is recommended."))); }
                else if let Some(s) = short { if s.value.len() > 90 { f.push(ValidationFinding::err("WEA002", format!("{p}WEA short text exceeds 90 chars ({} chars).", s.value.len()))); } }
                if let Some(l) = info.parameters.iter().find(|p| p.name == "CMAMlongtext") {
                    if l.value.len() > 360 { f.push(ValidationFinding::err("WEA003", format!("{p}WEA long text exceeds 360 chars ({} chars).", l.value.len()))); }
                }
                if info.urgency != CapUrgency::Immediate  { f.push(ValidationFinding::warn("WEA004", format!("{p}WEA alerts typically require Immediate urgency."))); }
                if info.severity == CapSeverity::Minor || info.severity == CapSeverity::Unknown {
                    f.push(ValidationFinding::warn("WEA005", format!("{p}WEA alerts typically require Extreme or Severe severity.")));
                }
            }
            if has_eas && info.areas.iter().all(|a| a.same_codes.is_empty()) {
                f.push(ValidationFinding::err("EAS001", format!("{p}EAS alerts require at least one SAME location code.")));
            }
            if has_nwem && !info.parameters.iter().any(|p| p.name == "VTEC") {
                f.push(ValidationFinding::warn("NWEM001", format!("{p}NWEM alerts should include a VTEC string.")));
            }
            if alert.status == CapStatus::Actual {
                if info.description.as_ref().map_or(true, |d| d.is_empty()) {
                    f.push(ValidationFinding::warn("CAP013", format!("{p}Description is strongly recommended for Actual alerts.")));
                }
                if info.instruction.as_ref().map_or(true, |i| i.is_empty()) {
                    f.push(ValidationFinding::warn("CAP014", format!("{p}Instruction is strongly recommended for Actual alerts.")));
                }
            }
        }
        r
    }
}
