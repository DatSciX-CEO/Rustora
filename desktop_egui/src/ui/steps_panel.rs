use crate::state::AppState;
use crate::ui::theme::{ACCENT, BLUE};

pub fn show_steps_panel(ui: &mut egui::Ui, state: &mut AppState) {
    ui.vertical(|ui| {
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new("APPLIED STEPS")
                .size(10.0)
                .strong()
                .monospace()
                .weak(),
        );
        ui.add_space(4.0);
        ui.separator();

        if state.steps_entries.is_empty() {
            ui.label(egui::RichText::new("No steps recorded").weak().size(11.0));
            return;
        }

        let entries = state.steps_entries.clone();
        let active_idx = state
            .steps_active_index
            .unwrap_or(entries.len().saturating_sub(1));
        let mut action: Option<usize> = None;

        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                for (i, entry) in entries.iter().enumerate() {
                    let is_active = i == active_idx;
                    let is_past = i <= active_idx;

                    let label_text = format!("{}. {}", i + 1, entry.label);
                    let label = egui::RichText::new(&label_text).size(11.0).monospace();
                    let label = if is_active {
                        label.color(ACCENT).strong()
                    } else if is_past {
                        label.color(BLUE)
                    } else {
                        label.weak()
                    };

                    if ui
                        .add(egui::SelectableLabel::new(is_active, label))
                        .clicked()
                    {
                        action = Some(i);
                    }
                }
            });

        if let Some(idx) = action {
            state.select_step(idx);
        }
    });
}
