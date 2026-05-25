use egui::{Color32, RichText, ScrollArea, Ui};
use crate::theme::*;

#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub alert_id:      String,
    pub event_type:    String,
    pub headline:      String,
    pub cap_status:    String,
    pub channels:      String,
    pub is_success:    bool,
    pub submitted_at:  chrono::DateTime<chrono::Local>,
    pub elapsed_ms:    u64,
    pub server_msg_id: Option<String>,
    pub error_summary: Option<String>,
    pub raw_cap_xml:   String,
}

impl HistoryEntry {
    pub fn elapsed_label(&self)   -> String { format!("{} ms", self.elapsed_ms) }
    pub fn submitted_label(&self) -> String { self.submitted_at.format("%m/%d/%Y %H:%M:%S").to_string() }
    pub fn status_label(&self)    -> &str   { if self.is_success { "✓ Accepted" } else { "✗ Failed" } }
    pub fn status_color(&self)    -> Color32 { if self.is_success { STATUS_SUCCESS } else { STATUS_ERROR } }
}

#[derive(Default)]
pub struct HistoryState {
    pub entries:      Vec<HistoryEntry>,
    pub selected_idx: Option<usize>,
}

impl HistoryState {
    pub fn add(&mut self, e: HistoryEntry) { self.entries.insert(0, e); self.selected_idx = Some(0); }
    pub fn selected(&self) -> Option<&HistoryEntry> { self.selected_idx.and_then(|i| self.entries.get(i)) }
}

pub fn show(ui: &mut Ui, state: &mut HistoryState) {
    egui::Frame::none().fill(BG_PANEL).inner_margin(egui::Margin::symmetric(20.0, 14.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.heading(RichText::new("Submission History").color(TEXT_PRIMARY).size(18.0));
                    ui.label(RichText::new(format!("{} alerts in session", state.entries.len()))
                        .color(TEXT_SECONDARY).size(11.0));
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Clear All").clicked() { state.entries.clear(); state.selected_idx = None; }
                    if let Some(e) = state.selected() {
                        if ui.button("Copy XML").clicked() {
                            ui.output_mut(|o| o.copied_text = e.raw_cap_xml.clone());
                        }
                    }
                });
            });
        });

    ui.separator();

    if state.entries.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(60.0);
            ui.label(RichText::new("No alerts sent yet").color(TEXT_MUTED).size(16.0));
            ui.label(RichText::new("Alerts you submit will appear here with their full CAP XML.")
                .color(TEXT_MUTED).size(12.0));
        });
        return;
    }

    ui.columns(2, |cols| {
        // Table
        ScrollArea::vertical().id_source("hist_scroll").show(&mut cols[0], |ui| {
            use egui_extras::{Column, TableBuilder};
            TableBuilder::new(ui).striped(true).resizable(true)
                .column(Column::initial(100.0))
                .column(Column::initial(75.0))
                .column(Column::initial(140.0))
                .column(Column::remainder())
                .column(Column::initial(75.0))
                .column(Column::initial(135.0))
                .column(Column::initial(65.0))
                .header(22.0, |mut h| {
                    for col in ["STATUS","CAP STATUS","EVENT TYPE","HEADLINE","CHANNELS","SUBMITTED","ELAPSED"] {
                        h.col(|ui| { ui.label(RichText::new(col).color(TEXT_MUTED).size(10.0).strong()); });
                    }
                })
                .body(|mut body| {
                    let sel = state.selected_idx;
                    for (i, entry) in state.entries.iter().enumerate() {
                        body.row(28.0, |mut row| {
                            row.set_selected(Some(i) == sel);
                            row.col(|ui| {
                                if ui.add(egui::SelectableLabel::new(
                                    Some(i) == sel,
                                    RichText::new(entry.status_label()).color(entry.status_color()).size(11.0),
                                )).clicked() { state.selected_idx = Some(i); }
                            });
                            row.col(|ui| { ui.label(RichText::new(&entry.cap_status).size(11.0)); });
                            row.col(|ui| { ui.label(RichText::new(&entry.event_type).size(11.0)); });
                            row.col(|ui| { ui.label(RichText::new(&entry.headline).size(11.0)); });
                            row.col(|ui| { ui.label(RichText::new(&entry.channels).size(11.0)); });
                            row.col(|ui| { ui.label(RichText::new(entry.submitted_label()).size(11.0)); });
                            row.col(|ui| { ui.label(RichText::new(entry.elapsed_label()).size(11.0)); });
                        });
                    }
                });
        });

        // Detail pane
        if let Some(entry) = state.selected() {
            cols[1].vertical(|ui| {
                egui::Frame::none().fill(BG_PANEL).inner_margin(egui::Margin::same(16.0))
                    .show(ui, |ui| {
                        ui.label(RichText::new(&entry.event_type).color(TEXT_PRIMARY).size(14.0).strong());
                        ui.label(RichText::new(&entry.alert_id).color(TEXT_MUTED).size(10.0).monospace());
                        if let Some(sid) = &entry.server_msg_id {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Server ID: ").color(TEXT_SECONDARY).size(10.0));
                                ui.label(RichText::new(sid).color(TEXT_PRIMARY).size(10.0).monospace());
                            });
                        }
                        if !entry.is_success {
                            if let Some(err) = &entry.error_summary {
                                ui.add_space(4.0);
                                ui.label(RichText::new(err).color(STATUS_ERROR).size(11.0));
                            }
                        }
                    });
                ui.separator();
                ScrollArea::both().id_source("xml_detail").show(ui, |ui| {
                    let mut text = entry.raw_cap_xml.as_str();
                    ui.add(egui::TextEdit::multiline(&mut text)
                        .desired_width(f32::INFINITY).font(egui::FontId::monospace(10.0)).interactive(false));
                });
            });
        }
    });
}
