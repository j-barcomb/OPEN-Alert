use egui::{RichText, Ui};
use crate::{settings_store::AppSettings, theme::*};

pub struct SettingsState {
    pub settings:      AppSettings,
    pub cert_password: String,
    pub save_status:   String,
}

impl SettingsState {
    pub fn load() -> Self {
        Self { settings: AppSettings::load(), cert_password: String::new(), save_status: String::new() }
    }
}

pub fn show(ui: &mut Ui, state: &mut SettingsState) {
    egui::Frame::none().fill(BG_PANEL).inner_margin(egui::Margin::symmetric(20.0, 14.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading(RichText::new("Settings").color(TEXT_PRIMARY).size(18.0));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Save Settings").clicked() {
                        state.settings.config.cert_password = state.cert_password.clone();
                        state.save_status = match state.settings.save() {
                            Ok(_)  => "✓  Settings saved.".into(),
                            Err(e) => format!("✗  {e}"),
                        };
                    }
                    if !state.save_status.is_empty() {
                        let c = if state.save_status.starts_with('\u{2713}') { STATUS_SUCCESS } else { STATUS_ERROR };
                        ui.label(RichText::new(&state.save_status).color(c).size(11.0));
                    }
                });
            });
        });

    ui.separator();

    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(8.0);

        sec(ui, "CONNECTION", |ui| {
            let config = &mut state.settings.config;  // ← scoped inside closure, no conflict
            ui.columns(2, |c| {
                c[0].label(lbl("COG ID"));   c[0].text_edit_singleline(&mut config.cog_id);
                c[1].label(lbl("Sender"));   c[1].text_edit_singleline(&mut config.sender);
            });
            ui.label(lbl("Sender Name"));
            ui.text_edit_singleline(&mut config.sender_name);
            ui.add_space(8.0);
            ui.checkbox(&mut config.use_test_endpoint, "Use JITC Test Endpoint");
            if !config.use_test_endpoint {
                // ... warning frame
            }
        });

        sec(ui, "mTLS CERTIFICATE", |ui| {
            let config = &mut state.settings.config;  // ← fresh borrow per section
            ui.radio_value(&mut config.use_file_cert, true,  "Load from .p12 / .pfx file");
            ui.radio_value(&mut config.use_file_cert, false, "Load from Windows Certificate Store (thumbprint)");
            ui.add_space(8.0);
            if config.use_file_cert {
                ui.label(lbl("Certificate Path (.p12 / .pfx)"));
                ui.horizontal(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut config.cert_path).desired_width(f32::INFINITY));
                    if ui.button("Browse\u{2026}").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("PKCS#12", &["p12", "pfx"]).pick_file()
                        {
                            config.cert_path = path.display().to_string();
                        }
                    }
                });
                ui.label(lbl("Certificate Password"));
                ui.add(egui::TextEdit::singleline(&mut state.cert_password)  // ← needs state directly
                    .password(true).desired_width(f32::INFINITY));
            } else {
                ui.add(egui::TextEdit::singleline(&mut config.cert_thumbprint)
                    .font(egui::FontId::monospace(12.0)).desired_width(f32::INFINITY));
            }
        });

        sec(ui, "UI PREFERENCES", |ui| {
            ui.checkbox(&mut state.settings.config.confirm_before_send, "Confirm before sending Actual alerts");
        });

        ui.add_space(20.0);
    });
}

fn lbl(t: &str) -> RichText { RichText::new(t).color(TEXT_SECONDARY).size(11.0) }

fn sec(ui: &mut Ui, title: &str, f: impl FnOnce(&mut Ui)) {
    egui::Frame::none().fill(BG_CARD).stroke(egui::Stroke::new(1.0, BORDER))
        .rounding(egui::Rounding::same(4.0)).inner_margin(egui::Margin::same(16.0))
        .outer_margin(egui::Margin { bottom: 12.0, left: 20.0, right: 20.0, ..Default::default() })
        .show(ui, |ui| {
            ui.label(RichText::new(title).color(TEXT_MUTED).size(10.0).strong());
            ui.add_space(10.0);
            f(ui);
        });
}
