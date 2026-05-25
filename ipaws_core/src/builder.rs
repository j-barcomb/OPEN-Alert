use chrono::{DateTime, Utc};
use crate::models::*;

pub struct CapAlertBuilder { alert: CapAlert }
impl Default for CapAlertBuilder { fn default() -> Self { Self::new() } }

impl CapAlertBuilder {
    pub fn new() -> Self { Self { alert: CapAlert::default() } }
    pub fn with_sender(mut self, s: impl Into<String>) -> Self { self.alert.sender = s.into(); self }
    pub fn with_status(mut self, s: CapStatus)   -> Self { self.alert.status   = s; self }
    pub fn with_msg_type(mut self, t: CapMsgType) -> Self { self.alert.msg_type = t; self }
    pub fn with_scope(mut self, s: CapScope)     -> Self { self.alert.scope    = s; self }
    pub fn add_code(mut self, code: impl Into<String>) -> Self {
        self.alert.codes.push(code.into());
        self
    }

    pub fn add_reference(mut self, sender: impl Into<String>,
                         id: impl Into<String>, sent: DateTime<Utc>) -> Self {
        self.alert.references.push(format!("{},{},{}",
            sender.into(), id.into(), sent.format("%Y-%m-%dT%H:%M:%S+00:00")));
        self
    }

    pub fn add_info<F: FnOnce(AlertInfoBuilder) -> AlertInfoBuilder>(mut self, f: F) -> Self {
        self.alert.infos.push(f(AlertInfoBuilder::new()).build());
        self
    }

    pub fn build(self) -> CapAlert { self.alert }
}

pub struct AlertInfoBuilder { info: AlertInfo }

impl AlertInfoBuilder {
    pub fn new() -> Self { Self { info: AlertInfo::default() } }
    pub fn with_event(mut self, e: impl Into<String>) -> Self { self.info.event = e.into(); self }
    pub fn with_urgency(mut self, u: CapUrgency)     -> Self { self.info.urgency    = u; self }
    pub fn with_severity(mut self, s: CapSeverity)   -> Self { self.info.severity   = s; self }
    pub fn with_certainty(mut self, c: CapCertainty) -> Self { self.info.certainty  = c; self }
    pub fn with_effective(mut self, dt: DateTime<Utc>) -> Self { self.info.effective = Some(dt); self }
    pub fn with_expires(mut self, dt: DateTime<Utc>)   -> Self { self.info.expires   = Some(dt); self }
    pub fn with_sender_name(mut self, n: impl Into<String>) -> Self { self.info.sender_name = Some(n.into()); self }
    pub fn with_headline(mut self, h: impl Into<String>)    -> Self { self.info.headline    = Some(h.into()); self }
    pub fn with_description(mut self, d: impl Into<String>) -> Self { self.info.description = Some(d.into()); self }
    pub fn with_instruction(mut self, i: impl Into<String>) -> Self { self.info.instruction = Some(i.into()); self }

    pub fn add_parameter(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.info.parameters.push(CapParameter { name: name.into(), value: value.into() });
        self
    }

    pub fn add_wea_routing(mut self, short: Option<&str>, long: Option<&str>) -> Self {
        self.info.parameters.push(CapParameter { name: "WEAHandling".into(), value: "Broadcast".into() });
        if let Some(t) = short {
            self.info.parameters.push(CapParameter { name: "CMAMtext".into(), value: t.chars().take(90).collect() });
        }
        if let Some(t) = long {
            self.info.parameters.push(CapParameter { name: "CMAMlongtext".into(), value: t.chars().take(360).collect() });
        }
        self
    }

    pub fn add_eas_routing(mut self) -> Self {
        self.info.parameters.push(CapParameter { name: "EASHandling".into(), value: "Broadcast".into() });
        self
    }

    pub fn add_nwem_routing(mut self, vtec: Option<&str>, ugc: Option<&str>) -> Self {
        self.info.parameters.push(CapParameter { name: "NWEMHandling".into(), value: "Broadcast".into() });
        if let Some(v) = vtec { self.info.parameters.push(CapParameter { name: "VTEC".into(), value: v.into() }); }
        if let Some(u) = ugc  { self.info.parameters.push(CapParameter { name: "UGC".into(),  value: u.into() }); }
        self
    }

    pub fn add_same_code(mut self, code: impl Into<String>) -> Self {
        if self.info.areas.is_empty() { self.info.areas.push(AlertArea::default()); }
        self.info.areas.last_mut().unwrap().same_codes.push(code.into());
        self
    }

    pub fn add_area<F: FnOnce(AlertAreaBuilder) -> AlertAreaBuilder>(mut self, f: F) -> Self {
        self.info.areas.push(f(AlertAreaBuilder::new()).build());
        self
    }

    pub fn build(self) -> AlertInfo { self.info }
}

pub struct AlertAreaBuilder { area: AlertArea }

impl AlertAreaBuilder {
    pub fn new() -> Self { Self { area: AlertArea::default() } }
    pub fn with_description(mut self, d: impl Into<String>) -> Self { self.area.area_desc = d.into(); self }
    pub fn add_same_code(mut self, c: impl Into<String>)    -> Self { self.area.same_codes.push(c.into()); self }
    pub fn add_polygon(mut self, p: impl Into<String>)      -> Self { self.area.polygons.push(p.into()); self }
    pub fn build(self) -> AlertArea { self.area }
}
