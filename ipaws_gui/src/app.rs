use egui::{Color32, RichText};
use ipaws_core::{xml, CapAlert, IpawsClient};
use std::sync::{Arc, Mutex};
use crate::{
    compose::ComposeState,
    history::{HistoryEntry, HistoryState},
    settings_tab::SettingsState,
    theme::*,
};

#[derive(Clone, PartialEq)]
pub enum Tab { Compose, History, Settings }

struct SendResult { entry: HistoryEntry }

pub struct IpawsApp {
    tab:            Tab,
    compose:        ComposeState,
    history:        HistoryState,
    settings:       SettingsState,
    status:         String,
    pending_result: Arc<Mutex<Option<SendResult>>>,
    is_sending:     bool,
    /// Alert awaiting user confirmation before submission (Actual + confirm-on toggle).
    confirm_alert:  Option<CapAlert>,
}

impl IpawsApp {
    pub fn new(cc: &eframe::CreationContext) -> Self {
        crate::theme::apply(&cc.egui_ctx);
        Self {
            tab: Tab::Compose, compose: ComposeState::default(),
            history: HistoryState::default(), settings: SettingsState::load(),
            status: "Ready.".into(),
            pending_result: Arc::new(Mutex::new(None)),
            is_sending: false,
            confirm_alert: None,
        }
    }

    fn send_alert(&mut self, alert: CapAlert, ctx: egui::Context) {
        if self.is_sending { return; }

        let mut config        = self.settings.settings.config.clone();
        config.cert_password  = self.settings.cert_password.clone();

        let event_type = alert.infos.first().map(|i| i.event.clone()).unwrap_or_default();
        let headline   = alert.infos.first().and_then(|i| i.headline.clone()).unwrap_or_default();
        let cap_status = format!("{}", alert.status);
        let raw_xml    = xml::serialize(&alert, true);

        let mut channels = Vec::new();
        if let Some(info) = alert.infos.first() {
            if info.parameters.iter().any(|p| p.name == "WEAHandling")  { channels.push("WEA"); }
            if info.parameters.iter().any(|p| p.name == "EASHandling")  { channels.push("EAS"); }
            if info.parameters.iter().any(|p| p.name == "NWEMHandling") { channels.push("NWEM"); }
        }
        let channels_label = channels.join(", ");

        let pending     = Arc::clone(&self.pending_result);
        self.is_sending = true;
        self.status     = "Sending\u{2026}".into();

        std::thread::spawn(move || {
            let submitted_at = chrono::Local::now();
            let alert_id     = alert.identifier.clone();
            let client       = IpawsClient::new(config);
            let resp         = client.submit(&alert);

            let entry = HistoryEntry {
                alert_id, event_type, headline, cap_status,
                channels:      channels_label,
                is_success:    resp.is_success,
                submitted_at,
                elapsed_ms:    resp.elapsed.as_millis() as u64,
                server_msg_id: resp.server_message_id,
                error_summary: if resp.errors.is_empty() { None } else { Some(resp.errors.join("; ")) },
                raw_cap_xml:   raw_xml,
            };
            *pending.lock().unwrap() = Some(SendResult { entry });
            ctx.request_repaint();
        });
    }

    fn poll_results(&mut self) {
        let result = self.pending_result.lock().unwrap().take();
        if let Some(r) = result {
            self.status = if r.entry.is_success {
                format!("✓  Accepted  ({})", r.entry.server_msg_id.as_deref().unwrap_or("\u{2014}"))
            } else {
                format!("✗  Failed: {}", r.entry.error_summary.as_deref().unwrap_or("unknown error"))
            };
            self.history.add(r.entry);
            self.is_sending = false;
        }
    }
}

impl eframe::App for IpawsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_results();

        // Sidebar
        egui::SidePanel::left("sidebar").min_width(200.0).max_width(200.0)
            .frame(egui::Frame::none().fill(BG_SIDEBAR).stroke(egui::Stroke::new(1.0, BORDER)))
            .show(ctx, |ui| {
                // Branding
                egui::Frame::none()
                    .inner_margin(egui::Margin { left: 20.0, right: 20.0, top: 22.0, bottom: 18.0 })
                    .show(ui, |ui| {
                        ui.label(RichText::new("IPAWS").color(TEXT_PRIMARY).size(20.0).strong());
                        ui.label(RichText::new("ALERT CONSOLE").color(ACCENT_BLUE).size(9.0).strong());
                    });

                ui.add_space(12.0);
                ui.label(RichText::new("NAVIGATION").color(TEXT_MUTED).size(9.0).strong());
                ui.add_space(8.0);

                nav(ui, "⊕  Compose Alert",     &mut self.tab, Tab::Compose);
                ui.horizontal(|ui| {
                    nav(ui, "\u{2261}  History", &mut self.tab, Tab::History);
                    egui::Frame::none().fill(ACCENT_BLUE).rounding(egui::Rounding::same(8.0))
                        .inner_margin(egui::Margin::symmetric(5.0, 1.0))
                        .show(ui, |ui| {
                            ui.label(RichText::new(self.history.entries.len().to_string())
                                .color(Color32::WHITE).size(10.0).strong());
                        });
                });
                nav(ui, "\u{2699}  Settings", &mut self.tab, Tab::Settings);

                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    egui::Frame::none()
                        .inner_margin(egui::Margin::same(16.0))
                        .stroke(egui::Stroke::new(1.0, BORDER))
                        .show(ui, |ui| {
                            ui.label(RichText::new(&self.status).color(TEXT_SECONDARY).size(10.0));
                            ui.label(RichText::new(self.settings.settings.config.endpoint_label())
                                .color(TEXT_MUTED).size(9.0));
                            let (dot, label) = if self.is_sending {
                                (STATUS_TEST, "Sending\u{2026}")
                            } else {
                                (STATUS_SUCCESS, "Ready")
                            };
                            ui.horizontal(|ui| {
                                egui::Frame::none().fill(dot).rounding(egui::Rounding::same(4.0))
                                    .show(ui, |ui| { ui.add_space(7.0); });
                                ui.label(RichText::new(label).color(TEXT_SECONDARY).size(10.0).strong());
                            });
                        });
                });
            });

        // Main content
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(BG_SHELL))
            .show(ctx, |ui| {
                match self.tab {
                    Tab::Compose => {
                        let sender = self.settings.settings.config.sender.clone();
                        if let Some(alert) = crate::compose::show(ui, &mut self.compose, &sender, self.is_sending) {
                            // If Actual alert and confirmation is enabled, queue for modal.
                            let is_actual = alert.status == ipaws_core::CapStatus::Actual;
                            let confirm_on = self.settings.settings.config.confirm_before_send;
                            if is_actual && confirm_on {
                                self.confirm_alert = Some(alert);
                            } else {
                                self.send_alert(alert, ctx.clone());
                            }
                        }
                    }
                    Tab::History  => crate::history::show(ui, &mut self.history),
                    Tab::Settings => crate::settings_tab::show(ui, &mut self.settings),
                }
            });

        // Confirmation modal for Actual alerts.
        self.render_confirm_modal(ctx);
    }
}

impl IpawsApp {
    fn render_confirm_modal(&mut self, ctx: &egui::Context) {
        if self.confirm_alert.is_none() { return; }
        let mut keep_open = true;
        let mut do_send = false;
        let mut do_cancel = false;

        let event = self.confirm_alert.as_ref()
            .and_then(|a| a.infos.first())
            .map(|i| i.event.clone())
            .unwrap_or_default();
        let headline = self.confirm_alert.as_ref()
            .and_then(|a| a.infos.first())
            .and_then(|i| i.headline.clone())
            .unwrap_or_default();
        let endpoint = self.settings.settings.config.endpoint_label().to_string();

        egui::Window::new(RichText::new("⚠ Confirm LIVE Alert Submission").color(STATUS_ERROR).strong())
            .open(&mut keep_open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .frame(egui::Frame::window(&ctx.style()).fill(BG_PANEL))
            .show(ctx, |ui| {
                ui.add_space(6.0);
                ui.label(RichText::new(
                    "You are about to submit a LIVE alert to IPAWS-OPEN.\n\
                     This will broadcast to the public via WEA, EAS, and/or NWEM."
                ).color(TEXT_PRIMARY).size(13.0));
                ui.add_space(10.0);

                egui::Frame::none()
                    .fill(BG_CARD)
                    .stroke(egui::Stroke::new(1.0, BORDER))
                    .rounding(egui::Rounding::same(4.0))
                    .inner_margin(egui::Margin::same(12.0))
                    .show(ui, |ui| {
                        ui.label(RichText::new(format!("Event:    {event}")).color(TEXT_PRIMARY).size(12.0));
                        ui.label(RichText::new(format!("Headline: {headline}")).color(TEXT_PRIMARY).size(12.0));
                        ui.label(RichText::new(format!("Endpoint: {endpoint}")).color(TEXT_PRIMARY).size(12.0));
                    });

                ui.add_space(14.0);
                ui.horizontal(|ui| {
                    if ui.add(egui::Button::new(RichText::new("Cancel").color(TEXT_PRIMARY)))
                        .clicked() { do_cancel = true; }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(egui::Button::new(RichText::new("Send LIVE Alert").color(Color32::WHITE))
                            .fill(STATUS_ERROR)).clicked() { do_send = true; }
                    });
                });
            });

        if do_cancel || !keep_open {
            self.confirm_alert = None;
            self.status = "LIVE alert canceled.".into();
        } else if do_send {
            if let Some(alert) = self.confirm_alert.take() {
                self.send_alert(alert, ctx.clone());
            }
        }
    }
}

fn nav(ui: &mut egui::Ui, label: &str, current: &mut Tab, tab: Tab) {
    let active = *current == tab;
    let text   = RichText::new(label).size(13.0).color(if active { TEXT_PRIMARY } else { TEXT_SECONDARY });
    let fill   = if active { BG_CARD } else { Color32::TRANSPARENT };
    if ui.add_sized([180.0, 36.0], egui::Button::new(text).fill(fill)).clicked() {
        *current = tab;
    }
}
