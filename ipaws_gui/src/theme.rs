use egui::{Color32, Rounding, Stroke, Visuals};

pub const BG_SHELL:   Color32 = Color32::from_rgb(0x0D, 0x11, 0x17);
pub const BG_PANEL:   Color32 = Color32::from_rgb(0x16, 0x1B, 0x22);
pub const BG_CARD:    Color32 = Color32::from_rgb(0x1C, 0x21, 0x28);
pub const BG_INPUT:   Color32 = Color32::from_rgb(0x0D, 0x11, 0x17);
pub const BG_SIDEBAR: Color32 = Color32::from_rgb(0x0A, 0x0E, 0x13);

pub const BORDER:       Color32 = Color32::from_rgb(0x30, 0x36, 0x3D);
pub const BORDER_FOCUS: Color32 = Color32::from_rgb(0x3B, 0x7D, 0xD8);

pub const TEXT_PRIMARY:   Color32 = Color32::from_rgb(0xE6, 0xED, 0xF3);
pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(0x8B, 0x94, 0x9E);
pub const TEXT_MUTED:     Color32 = Color32::from_rgb(0x48, 0x4F, 0x58);

pub const ACCENT_BLUE: Color32 = Color32::from_rgb(0x3B, 0x7D, 0xD8);

pub const SEV_EXTREME:  Color32 = Color32::from_rgb(0xFF, 0x3B, 0x30);
pub const SEV_SEVERE:   Color32 = Color32::from_rgb(0xFF, 0x95, 0x00);
pub const SEV_MODERATE: Color32 = Color32::from_rgb(0xFF, 0xD6, 0x0A);
pub const SEV_MINOR:    Color32 = Color32::from_rgb(0x34, 0xC7, 0x59);
pub const SEV_UNKNOWN:  Color32 = Color32::from_rgb(0x8B, 0x94, 0x9E);

pub const STATUS_SUCCESS: Color32 = Color32::from_rgb(0x34, 0xC7, 0x59);
pub const STATUS_ERROR:   Color32 = Color32::from_rgb(0xFF, 0x3B, 0x30);
pub const STATUS_TEST:    Color32 = Color32::from_rgb(0xFF, 0x95, 0x00);

pub fn apply(ctx: &egui::Context) {
    let mut v = Visuals::dark();
    v.panel_fill          = BG_PANEL;
    v.faint_bg_color      = BG_CARD;
    v.extreme_bg_color    = BG_INPUT;
    v.override_text_color = Some(TEXT_PRIMARY);
    v.window_rounding     = Rounding::same(6.0);
    v.window_stroke       = Stroke::new(1.0, BORDER);
    v.selection.bg_fill   = ACCENT_BLUE;

    macro_rules! set_widget {
        ($w:expr, $fill:expr, $fg:expr, $stroke:expr) => {
            $w.bg_fill   = $fill;
            $w.fg_stroke = Stroke::new(1.0, $fg);
            $w.bg_stroke = Stroke::new(1.0, $stroke);
            $w.rounding  = Rounding::same(4.0);
        };
    }
    set_widget!(v.widgets.noninteractive, BG_PANEL,  TEXT_SECONDARY, BORDER);
    set_widget!(v.widgets.inactive,       BG_INPUT,  TEXT_PRIMARY,   BORDER);
    set_widget!(v.widgets.hovered,        BG_PANEL,  TEXT_PRIMARY,   BORDER_FOCUS);
    set_widget!(v.widgets.active,         ACCENT_BLUE, Color32::WHITE, ACCENT_BLUE);
    set_widget!(v.widgets.open,           BG_CARD,   TEXT_PRIMARY,   BORDER);

    ctx.set_visuals(v);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing   = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(12.0, 6.0);
    ctx.set_style(style);
}

pub fn severity_color(s: &ipaws_core::CapSeverity) -> Color32 {
    match s {
        ipaws_core::CapSeverity::Extreme  => SEV_EXTREME,
        ipaws_core::CapSeverity::Severe   => SEV_SEVERE,
        ipaws_core::CapSeverity::Moderate => SEV_MODERATE,
        ipaws_core::CapSeverity::Minor    => SEV_MINOR,
        ipaws_core::CapSeverity::Unknown  => SEV_UNKNOWN,
    }
}

pub fn cap_status_color(s: &ipaws_core::CapStatus) -> Color32 {
    match s {
        ipaws_core::CapStatus::Actual   => SEV_EXTREME,
        ipaws_core::CapStatus::Test     => STATUS_TEST,
        ipaws_core::CapStatus::Exercise => ACCENT_BLUE,
        _                               => TEXT_MUTED,
    }
}
