use crate::state::AppState;
use crate::ui::theme::{ERROR, SUCCESS, WARNING};
use egui::Ui;

pub fn show_status_bar(ui: &mut Ui, state: &mut AppState) {
    ui.horizontal(|ui| {
        if let Some(ref err) = state.error.clone() {
            ui.label(egui::RichText::new("Error:").color(ERROR).size(11.0));
            ui.label(
                egui::RichText::new(truncate(err, 100))
                    .color(ERROR)
                    .size(11.0),
            );
            if state.has_retry() && ui.small_button("Retry").clicked() {
                state.retry_last_action();
            }
            if ui.small_button("Dismiss").clicked() {
                state.dismiss_error();
            }
        } else if state.loading {
            ui.spinner();
            ui.label(egui::RichText::new("Processing...").color(WARNING).size(11.0));
        } else if state.name.is_some() {
            ui.label(
                egui::RichText::new(format!("Rows: {}", format_number(state.total_rows)))
                    .size(11.0),
            );
            ui.separator();
            ui.label(
                egui::RichText::new(format!("Cols: {}", state.columns.len())).size(11.0),
            );
            if let Some(bytes) = state.size_bytes {
                ui.separator();
                ui.label(egui::RichText::new(format_bytes(bytes)).size(11.0));
            }
            if state.persistent {
                ui.separator();
                ui.label(egui::RichText::new("Persistent").color(SUCCESS).size(11.0).strong());
            }
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new("egui | offline")
                    .size(9.0)
                    .monospace()
                    .weak(),
            );
            ui.separator();
            if let Some(ref proj) = state.project {
                ui.label(
                    egui::RichText::new(&proj.path)
                        .size(10.0)
                        .monospace()
                        .weak(),
                );
            }
            if let Some(ref name) = state.name {
                ui.label(
                    egui::RichText::new(format!("[{}]", name))
                        .size(10.0)
                        .monospace()
                        .weak(),
                );
                ui.separator();
                ui.label(
                    egui::RichText::new(format!(
                        "Page {}-{} of {}",
                        state.offset + 1,
                        (state.offset + state.page_size).min(state.total_rows),
                        state.total_rows
                    ))
                    .size(10.0)
                    .weak(),
                );
            }
        });
    });
}

fn format_number(n: usize) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max {
        s
    } else {
        &s[..max]
    }
}
