use egui::{Color32, Stroke, Visuals};

pub const ACCENT: Color32 = Color32::from_rgb(196, 90, 44);
#[allow(dead_code)]
pub const ACCENT_HOVER: Color32 = Color32::from_rgb(169, 75, 36);
pub const BLUE: Color32 = Color32::from_rgb(58, 124, 191);
pub const SUCCESS: Color32 = Color32::from_rgb(58, 138, 92);
pub const WARNING: Color32 = Color32::from_rgb(184, 134, 11);
pub const ERROR: Color32 = Color32::from_rgb(199, 62, 58);
pub const NULL_COLOR: Color32 = Color32::from_rgb(168, 144, 128);

pub const CHART_COLORS: &[Color32] = &[
    Color32::from_rgb(196, 90, 44),
    Color32::from_rgb(58, 124, 191),
    Color32::from_rgb(58, 138, 92),
    Color32::from_rgb(184, 134, 11),
    Color32::from_rgb(123, 94, 167),
    Color32::from_rgb(199, 62, 58),
    Color32::from_rgb(46, 106, 168),
    Color32::from_rgb(224, 124, 62),
    Color32::from_rgb(91, 165, 130),
    Color32::from_rgb(156, 102, 68),
];

pub fn apply_rustora_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    if ctx.style().visuals.dark_mode {
        let mut visuals = Visuals::dark();
        visuals.override_text_color = Some(Color32::from_rgb(240, 235, 228));
        visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(35, 31, 27);
        visuals.widgets.inactive.bg_fill = Color32::from_rgb(42, 37, 32);
        visuals.widgets.hovered.bg_fill = Color32::from_rgb(51, 46, 40);
        visuals.widgets.active.bg_fill = Color32::from_rgb(61, 56, 48);
        visuals.extreme_bg_color = Color32::from_rgb(26, 22, 18);
        visuals.faint_bg_color = Color32::from_rgb(30, 26, 22);
        visuals.window_fill = Color32::from_rgb(35, 31, 27);
        visuals.panel_fill = Color32::from_rgb(35, 31, 27);
        visuals.selection.bg_fill = Color32::from_rgba_premultiplied(212, 114, 60, 30);
        visuals.selection.stroke = Stroke::new(1.0, Color32::from_rgb(212, 114, 60));
        style.visuals = visuals;
    } else {
        let mut visuals = Visuals::light();
        visuals.override_text_color = Some(Color32::from_rgb(44, 36, 24));
        visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(255, 255, 255);
        visuals.widgets.inactive.bg_fill = Color32::from_rgb(250, 248, 245);
        visuals.widgets.hovered.bg_fill = Color32::from_rgb(240, 235, 228);
        visuals.widgets.active.bg_fill = Color32::from_rgb(232, 224, 214);
        visuals.extreme_bg_color = Color32::from_rgb(245, 242, 238);
        visuals.faint_bg_color = Color32::from_rgb(250, 247, 243);
        visuals.window_fill = Color32::from_rgb(255, 255, 255);
        visuals.panel_fill = Color32::from_rgb(255, 255, 255);
        visuals.selection.bg_fill = Color32::from_rgba_premultiplied(196, 90, 44, 20);
        visuals.selection.stroke = Stroke::new(1.0, ACCENT);
        style.visuals = visuals;
    }

    style.spacing.button_padding = egui::vec2(8.0, 4.0);
    style.spacing.item_spacing = egui::vec2(6.0, 4.0);
    style.interaction.tooltip_delay = 0.3;

    ctx.set_style(style);
}
