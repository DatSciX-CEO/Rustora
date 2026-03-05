use crate::state::AppState;

pub fn show_stats_panel(ctx: &egui::Context, state: &mut AppState) {
    if !state.stats_visible {
        return;
    }

    let (columns, rows) = match &state.stats_data {
        Some(s) => (s.columns.clone(), s.rows.clone()),
        None => {
            state.stats_visible = false;
            return;
        }
    };

    let mut close = false;

    egui::Window::new("Summary Statistics")
        .collapsible(false)
        .resizable(true)
        .default_size([700.0, 400.0])
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            egui::ScrollArea::both()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    egui::Grid::new("stats_grid")
                        .striped(true)
                        .min_col_width(80.0)
                        .show(ui, |ui| {
                            for col_name in &columns {
                                ui.label(
                                    egui::RichText::new(col_name.as_str())
                                        .strong()
                                        .monospace()
                                        .size(11.0),
                                );
                            }
                            ui.end_row();

                            for row in &rows {
                                for val in row {
                                    ui.label(
                                        egui::RichText::new(val.as_str())
                                            .monospace()
                                            .size(11.0),
                                    );
                                }
                                ui.end_row();
                            }
                        });
                });

            ui.add_space(8.0);
            if ui.button("Close").clicked() {
                close = true;
            }
        });

    if close {
        state.stats_visible = false;
        state.stats_data = None;
    }
}
