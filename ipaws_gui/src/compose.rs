use chrono::{DateTime, Local, NaiveDate, TimeZone, Utc};
use egui::{Color32, RichText, ScrollArea, Ui};
use ipaws_core::*;
use crate::theme::*;

pub const COMMON_EVENT_TYPES: &[&str] = &[
    "Tornado Warning", "Tornado Watch", "Severe Thunderstorm Warning",
    "Flash Flood Warning", "Flash Flood Watch", "Winter Storm Warning",
    "Blizzard Warning", "Ice Storm Warning", "High Wind Warning",
    "Hurricane Warning", "Hurricane Watch", "Tsunami Warning",
    "Earthquake Warning", "Volcanic Ashfall Advisory",
    "Shelter In Place Warning", "Evacuation Immediate",
    "Civil Emergency Message", "Law Enforcement Warning",
    "Hazardous Materials Warning", "Nuclear Power Plant Warning",
    "Administrative Message", "Custom Event\u{2026}",
];

pub struct ComposeState {
    pub status: CapStatus, pub msg_type: CapMsgType, pub scope: CapScope,
    pub ref_alert_id: String,
    pub event_type: String, pub custom_event: String,
    pub urgency: CapUrgency, pub severity: CapSeverity, pub certainty: CapCertainty,
    pub headline: String, pub description: String, pub instruction: String,
    pub effective_str: String, pub expiry_str: String,
    pub area_desc: String, pub same_input: String, pub same_codes: Vec<String>,
    pub polygon_text: String,
    pub wea_enabled: bool, pub wea_short: String, pub wea_long: String,
    pub eas_enabled: bool, pub eas_org: String, pub eas_callsign: String,
    pub nwem_enabled: bool, pub vtec: String, pub ugc: String,
    pub validation_summary: String, pub has_errors: bool, pub has_warnings: bool,
    pub xml_preview: String,
    /// Hash of state fields that affect the XML. The preview re-serializes only
    /// when this differs from the last serialized snapshot. Without this, we'd
    /// pay the cost of XML serialization every frame (~60×/sec while visible).
    prev_hash: u64,
}

impl Default for ComposeState {
    fn default() -> Self {
        let mut s = Self {
            status: CapStatus::Test, msg_type: CapMsgType::Alert, scope: CapScope::Public,
            ref_alert_id: String::new(),
            event_type: "Tornado Warning".into(), custom_event: String::new(),
            urgency: CapUrgency::Immediate, severity: CapSeverity::Severe, certainty: CapCertainty::Observed,
            headline: String::new(), description: String::new(), instruction: String::new(),
            effective_str: String::new(), expiry_str: String::new(),
            area_desc: String::new(), same_input: String::new(), same_codes: Vec::new(),
            polygon_text: String::new(),
            wea_enabled: false, wea_short: String::new(), wea_long: String::new(),
            eas_enabled: false, eas_org: String::new(), eas_callsign: String::new(),
            nwem_enabled: false, vtec: String::new(), ugc: String::new(),
            validation_summary: String::new(), has_errors: false, has_warnings: false,
            xml_preview: String::new(),
            prev_hash: 0,
        };
        s.refresh_xml();
        s
    }
}

impl ComposeState {
    /// Cheap hash of all XML-affecting fields.
    fn current_hash(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut h = DefaultHasher::new();
        format!("{:?}", self.status).hash(&mut h);
        format!("{:?}", self.msg_type).hash(&mut h);
        format!("{:?}", self.scope).hash(&mut h);
        format!("{:?}", self.urgency).hash(&mut h);
        format!("{:?}", self.severity).hash(&mut h);
        format!("{:?}", self.certainty).hash(&mut h);
        self.ref_alert_id.hash(&mut h);
        self.event_type.hash(&mut h);
        self.custom_event.hash(&mut h);
        self.headline.hash(&mut h);
        self.description.hash(&mut h);
        self.instruction.hash(&mut h);
        self.effective_str.hash(&mut h);
        self.expiry_str.hash(&mut h);
        self.area_desc.hash(&mut h);
        self.same_codes.hash(&mut h);
        self.polygon_text.hash(&mut h);
        self.wea_enabled.hash(&mut h);
        self.wea_short.hash(&mut h);
        self.wea_long.hash(&mut h);
        self.eas_enabled.hash(&mut h);
        self.eas_org.hash(&mut h);
        self.eas_callsign.hash(&mut h);
        self.nwem_enabled.hash(&mut h);
        self.vtec.hash(&mut h);
        self.ugc.hash(&mut h);
        h.finish()
    }

    /// Re-serialize the XML preview only if state has changed.
    pub fn refresh_xml_if_dirty(&mut self) {
        let h = self.current_hash();
        if h != self.prev_hash {
            self.prev_hash = h;
            self.refresh_xml();
        }
    }
}

impl ComposeState {
    pub fn build_alert(&self, sender: &str) -> CapAlert {
        let event = if self.event_type.ends_with('\u{2026}') { self.custom_event.clone() }
                    else { self.event_type.clone() };

        let mut b = CapAlertBuilder::new()
            .with_sender(sender)
            .with_status(self.status.clone())
            .with_msg_type(self.msg_type.clone())
            .with_scope(self.scope.clone());

        if !self.ref_alert_id.is_empty()
            && (self.msg_type == CapMsgType::Update || self.msg_type == CapMsgType::Cancel)
        {
            b = b.add_reference(sender, &self.ref_alert_id, Utc::now());
        }

        let wea_s = &self.wea_short; let wea_l = &self.wea_long;
        let vtec  = &self.vtec;      let ugc    = &self.ugc;
        let area_desc = &self.area_desc;
        let same_codes = &self.same_codes;
        let polygon_text = &self.polygon_text;
        let wea_enabled  = self.wea_enabled;
        let eas_enabled  = self.eas_enabled;
        let nwem_enabled = self.nwem_enabled;
        let eff = parse_date(&self.effective_str);
        let exp = parse_date(&self.expiry_str);
        let headline    = self.headline.clone();
        let description = self.description.clone();
        let instruction = self.instruction.clone();
        let urgency   = self.urgency.clone();
        let severity  = self.severity.clone();
        let certainty = self.certainty.clone();

        b = b.add_info(move |mut info| {
            info = info.with_event(&event).with_urgency(urgency).with_severity(severity).with_certainty(certainty);
            if !headline.is_empty()    { info = info.with_headline(&headline); }
            if !description.is_empty() { info = info.with_description(&description); }
            if !instruction.is_empty() { info = info.with_instruction(&instruction); }
            if let Some(dt) = eff { info = info.with_effective(dt); }
            if let Some(dt) = exp { info = info.with_expires(dt); }
            if wea_enabled {
                let s = if wea_s.is_empty() { None } else { Some(wea_s.as_str()) };
                let l = if wea_l.is_empty() { None } else { Some(wea_l.as_str()) };
                info = info.add_wea_routing(s, l);
            }
            if eas_enabled  { info = info.add_eas_routing(); }
            if nwem_enabled {
                let v = if vtec.is_empty() { None } else { Some(vtec.as_str()) };
                let u = if ugc.is_empty()  { None } else { Some(ugc.as_str()) };
                info = info.add_nwem_routing(v, u);
            }
            info = info.add_area(move |mut area| {
                area = area.with_description(area_desc);
                for c in same_codes { area = area.add_same_code(c); }
                if !polygon_text.is_empty() { area = area.add_polygon(polygon_text); }
                area
            });
            info
        });
        b.build()
    }

    pub fn refresh_xml(&mut self) {
        let a = self.build_alert("preview@local");
        self.xml_preview = xml::serialize(&a, true);
    }

    pub fn validate(&mut self, sender: &str) {
        let a = self.build_alert(sender);
        let r = CapValidator::validate(&a);
        self.has_errors   = !r.is_valid();
        self.has_warnings = r.has_warnings();
        self.validation_summary = r.summary();
    }

    pub fn clear(&mut self) { *self = Self::default(); }
}

fn parse_date(s: &str) -> Option<DateTime<Utc>> {
    let s = s.trim();
    if s.is_empty() { return None; }
    NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
        .and_then(|d| d.and_hms_opt(0, 0, 0))
        .and_then(|dt| Local.from_local_datetime(&dt).single())
        .map(|dt| dt.with_timezone(&Utc))
}

// ── UI ────────────────────────────────────────────────────────────────────────

pub fn show(ui: &mut Ui, state: &mut ComposeState, sender: &str, is_sending: bool) -> Option<CapAlert> {
    let mut to_send: Option<CapAlert> = None;
    let is_actual = state.status == CapStatus::Actual;

    egui::Frame::none().fill(BG_PANEL)
        .inner_margin(egui::Margin::symmetric(20.0, 12.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading(RichText::new("Compose Alert").color(TEXT_PRIMARY).size(18.0));
                egui::Frame::none().fill(cap_status_color(&state.status))
                    .rounding(egui::Rounding::same(10.0))
                    .inner_margin(egui::Margin::symmetric(8.0, 2.0))
                    .show(ui, |ui| {
                        ui.label(RichText::new(format!("{}", state.status))
                            .color(Color32::WHITE).size(10.0).strong());
                    });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let send_label = if is_sending {
                        "Sending\u{2026}"
                    } else if is_actual {
                        "⚠ Send LIVE Alert"
                    } else {
                        "Send Alert"
                    };
                    let send_btn = if is_actual {
                        egui::Button::new(RichText::new(send_label).color(Color32::WHITE)).fill(SEV_EXTREME)
                    } else {
                        egui::Button::new(RichText::new(send_label).color(Color32::WHITE)).fill(ACCENT_BLUE)
                    };
                    let can_send = !state.headline.is_empty() && !is_sending;
                    if ui.add_enabled(can_send, send_btn).clicked() {
                        to_send = Some(state.build_alert(sender));
                    }
                    if ui.add_enabled(!is_sending, egui::Button::new("Clear")).clicked()    { state.clear(); }
                    if ui.add_enabled(!is_sending, egui::Button::new("Validate")).clicked() { state.validate(sender); }
                });
            });
        });

    if !state.validation_summary.is_empty() {
        let bg = if state.has_errors { Color32::from_rgb(0x2A,0x1A,0x1A) }
                 else if state.has_warnings { Color32::from_rgb(0x2A,0x1E,0x0A) }
                 else { Color32::from_rgb(0x1A,0x2A,0x1A) };
        let fg = if state.has_errors { STATUS_ERROR } else if state.has_warnings { STATUS_TEST } else { STATUS_SUCCESS };
        egui::Frame::none().fill(bg).inner_margin(egui::Margin::symmetric(20.0, 10.0))
            .show(ui, |ui| { ui.label(RichText::new(&state.validation_summary).color(fg).size(11.0).monospace()); });
    }

    ui.columns(2, |cols| {
        // ── Left: form ───────────────────────────────────────────────────────
        ScrollArea::vertical().id_source("compose_scroll").show(&mut cols[0], |ui| {
            ui.add_space(8.0);

            section(ui, "ALERT CLASSIFICATION", |ui| {
                ui.columns(3, |c| {
                    c[0].label(lbl("Status"));
                    combo(&mut c[0], "status", &format!("{}", state.status), |ui| {
                        for s in [CapStatus::Test, CapStatus::Exercise, CapStatus::System,
                                  CapStatus::Actual, CapStatus::Draft] {
                            let l = format!("{s}"); ui.selectable_value(&mut state.status, s, &l);
                        }
                    });
                    c[1].label(lbl("Message Type"));
                    combo(&mut c[1], "msg_type", &format!("{}", state.msg_type), |ui| {
                        for t in [CapMsgType::Alert, CapMsgType::Update, CapMsgType::Cancel] {
                            let l = format!("{t}"); ui.selectable_value(&mut state.msg_type, t, &l);
                        }
                    });
                    c[2].label(lbl("Scope"));
                    combo(&mut c[2], "scope", &format!("{}", state.scope), |ui| {
                        for s in [CapScope::Public, CapScope::Restricted, CapScope::Private] {
                            let l = format!("{s}"); ui.selectable_value(&mut state.scope, s, &l);
                        }
                    });
                });
                if state.msg_type == CapMsgType::Update || state.msg_type == CapMsgType::Cancel {
                    ui.label(lbl("Reference Alert ID (required for Update / Cancel)"));
                    ui.text_edit_singleline(&mut state.ref_alert_id);
                }
            });

            section(ui, "EVENT INFORMATION", |ui| {
                ui.label(lbl("Event Type"));
                combo(ui, "event", &state.event_type, |ui| {
                    for &e in COMMON_EVENT_TYPES { ui.selectable_value(&mut state.event_type, e.to_string(), e); }
                });
                if state.event_type.ends_with('\u{2026}') { ui.text_edit_singleline(&mut state.custom_event); }
                ui.add_space(4.0);
                ui.columns(3, |c| {
                    c[0].label(lbl("Urgency"));
                    combo(&mut c[0], "urgency", &format!("{}", state.urgency), |ui| {
                        for u in [CapUrgency::Immediate, CapUrgency::Expected, CapUrgency::Future,
                                  CapUrgency::Past, CapUrgency::Unknown] {
                            let l = format!("{u}"); ui.selectable_value(&mut state.urgency, u, &l);
                        }
                    });
                    c[1].label(lbl("Severity"));
                    combo(&mut c[1], "severity", &format!("{}", state.severity), |ui| {
                        for s in [CapSeverity::Extreme, CapSeverity::Severe, CapSeverity::Moderate,
                                  CapSeverity::Minor, CapSeverity::Unknown] {
                            let l = format!("{s}"); let c = severity_color(&s);
                            ui.selectable_value(&mut state.severity, s, RichText::new(&l).color(c));
                        }
                    });
                    c[2].label(lbl("Certainty"));
                    combo(&mut c[2], "certainty", &format!("{}", state.certainty), |ui| {
                        for c in [CapCertainty::Observed, CapCertainty::Likely, CapCertainty::Possible,
                                  CapCertainty::Unlikely, CapCertainty::Unknown] {
                            let l = format!("{c}"); ui.selectable_value(&mut state.certainty, c, &l);
                        }
                    });
                });
            });

            section(ui, "MESSAGE CONTENT", |ui| {
                let cc = state.headline.len();
                ui.horizontal(|ui| {
                    ui.label(lbl("Headline"));
                    let col = if cc > 160 { STATUS_ERROR } else { TEXT_MUTED };
                    ui.label(RichText::new(format!("{cc} chars")).color(col).size(10.0));
                });
                ui.text_edit_singleline(&mut state.headline);
                ui.label(lbl("Description"));
                ui.add(egui::TextEdit::multiline(&mut state.description).desired_rows(4).desired_width(f32::INFINITY));
                ui.label(lbl("Instruction"));
                ui.add(egui::TextEdit::multiline(&mut state.instruction).desired_rows(3).desired_width(f32::INFINITY));
            });

            section(ui, "TIMING  (YYYY-MM-DD)", |ui| {
                ui.columns(2, |c| {
                    c[0].label(lbl("Effective")); c[0].text_edit_singleline(&mut state.effective_str);
                    c[1].label(lbl("Expires"));   c[1].text_edit_singleline(&mut state.expiry_str);
                });
            });

            section(ui, "GEOGRAPHIC AREA", |ui| {
                ui.label(lbl("Area Description"));
                ui.text_edit_singleline(&mut state.area_desc);
                ui.label(lbl("SAME Location Codes (6-digit FIPS)"));
                ui.horizontal(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut state.same_input).desired_width(120.0).hint_text("039049"));
                    if ui.button("Add").clicked() && state.same_input.len() == 6 {
                        let c = state.same_input.trim().to_string();
                        if c.chars().all(|ch| ch.is_ascii_digit()) && !state.same_codes.contains(&c) {
                            state.same_codes.push(c);
                        }
                        state.same_input.clear();
                    }
                });
                let mut rm: Option<usize> = None;
                ui.horizontal_wrapped(|ui| {
                    for (i, code) in state.same_codes.iter().enumerate() {
                        egui::Frame::none().fill(BG_CARD).stroke(egui::Stroke::new(1.0, BORDER))
                            .rounding(egui::Rounding::same(4.0)).inner_margin(egui::Margin::symmetric(8.0, 3.0))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(code).monospace().size(11.0));
                                    if ui.small_button("x").clicked() { rm = Some(i); }
                                });
                            });
                    }
                });
                if let Some(i) = rm { state.same_codes.remove(i); }
                ui.label(lbl("Polygon (lat,lon pairs — space-separated)"));
                ui.add(egui::TextEdit::multiline(&mut state.polygon_text).desired_rows(2).desired_width(f32::INFINITY));
            });

            section(ui, "DISSEMINATION CHANNELS", |ui| {
                chan(ui, "WEA — Wireless Emergency Alerts", &mut state.wea_enabled, |ui| {
                    let sc = state.wea_short.len(); let lc = state.wea_long.len();
                    ui.horizontal(|ui| {
                        ui.label(lbl("Short Text (WEA 2.0, ≤90 chars)"));
                        let c = if sc > 90 { STATUS_ERROR } else { TEXT_MUTED };
                        ui.label(RichText::new(format!("{sc} / 90")).color(c).size(10.0));
                    });
                    ui.add(egui::TextEdit::multiline(&mut state.wea_short).desired_rows(2).desired_width(f32::INFINITY));
                    ui.horizontal(|ui| {
                        ui.label(lbl("Long Text (WEA 3.0, ≤360 chars)"));
                        let c = if lc > 360 { STATUS_ERROR } else { TEXT_MUTED };
                        ui.label(RichText::new(format!("{lc} / 360")).color(c).size(10.0));
                    });
                    ui.add(egui::TextEdit::multiline(&mut state.wea_long).desired_rows(3).desired_width(f32::INFINITY));
                });
                chan(ui, "EAS — Emergency Alert System", &mut state.eas_enabled, |ui| {
                    ui.columns(2, |c| {
                        c[0].label(lbl("Org Code")); c[0].add(egui::TextEdit::singleline(&mut state.eas_org).desired_width(f32::INFINITY));
                        c[1].label(lbl("Callsign")); c[1].add(egui::TextEdit::singleline(&mut state.eas_callsign).desired_width(f32::INFINITY));
                    });
                });
                chan(ui, "NWEM — National Weather Emergency Messages", &mut state.nwem_enabled, |ui| {
                    ui.label(lbl("VTEC String"));
                    ui.add(egui::TextEdit::singleline(&mut state.vtec).desired_width(f32::INFINITY));
                    ui.label(lbl("UGC Codes (comma-separated)"));
                    ui.add(egui::TextEdit::singleline(&mut state.ugc).desired_width(f32::INFINITY));
                });
            });
            ui.add_space(20.0);
        });

        // ── Right: XML preview ───────────────────────────────────────────────
        cols[1].vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("CAP XML PREVIEW").color(TEXT_SECONDARY).size(11.0).strong());
                if ui.small_button("Copy").clicked() {
                    ui.output_mut(|o| o.copied_text = state.xml_preview.clone());
                }
                if ui.small_button("Refresh").clicked() { state.refresh_xml(); }
            });
            ui.separator();
            ScrollArea::both().id_source("xml_scroll").show(ui, |ui| {
                let mut text = state.xml_preview.as_str();
                ui.add(egui::TextEdit::multiline(&mut text)
                    .desired_width(f32::INFINITY)
                    .font(egui::FontId::monospace(10.0))
                    .interactive(false));
            });
        });
    });

    state.refresh_xml_if_dirty();
    to_send
}

fn lbl(t: &str) -> RichText { RichText::new(t).color(TEXT_SECONDARY).size(11.0) }

fn section(ui: &mut Ui, title: &str, f: impl FnOnce(&mut Ui)) {
    egui::Frame::none().fill(BG_CARD).stroke(egui::Stroke::new(1.0, BORDER))
        .rounding(egui::Rounding::same(4.0)).inner_margin(egui::Margin::same(16.0))
        .outer_margin(egui::Margin { bottom: 12.0, ..Default::default() })
        .show(ui, |ui| {
            ui.label(RichText::new(title).color(TEXT_MUTED).size(10.0).strong());
            ui.add_space(8.0);
            f(ui);
        });
}

fn chan(ui: &mut Ui, label: &str, enabled: &mut bool, f: impl FnOnce(&mut Ui)) {
    egui::Frame::none().fill(BG_PANEL).stroke(egui::Stroke::new(1.0, BORDER))
        .rounding(egui::Rounding::same(4.0)).inner_margin(egui::Margin::same(14.0))
        .outer_margin(egui::Margin { bottom: 8.0, ..Default::default() })
        .show(ui, |ui| {
            ui.checkbox(enabled, RichText::new(label).strong());
            if *enabled { ui.add_space(8.0); f(ui); }
        });
}

fn combo(ui: &mut Ui, id: &str, selected: &str, f: impl FnOnce(&mut Ui)) {
    egui::ComboBox::from_id_source(id).selected_text(selected)
        .show_ui(ui, |ui| { f(ui); });
}
