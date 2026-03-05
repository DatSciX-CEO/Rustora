use crate::state::AppState;
use crate::ui::theme::ACCENT;

const DELIMITER_OPTIONS: &[(&str, u8)] = &[
    ("Comma (,)", b','),
    ("Tab", b'\t'),
    ("Semicolon (;)", b';'),
    ("Pipe (|)", b'|'),
];

pub fn show_ingest_dialog(ctx: &egui::Context, state: &mut AppState) {
    if !state.ingest_dialog {
        return;
    }

    egui::Window::new("Import Data")
        .collapsible(false)
        .resizable(true)
        .default_size([700.0, 500.0])
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label(
                egui::RichText::new(&state.ingest_file_path)
                    .monospace()
                    .size(11.0)
                    .weak(),
            );
            ui.add_space(8.0);

            let is_csv = {
                let ext = std::path::Path::new(&state.ingest_file_path)
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                ext == "csv" || ext == "tsv"
            };

            if is_csv {
                let mut options_changed = false;

                ui.horizontal(|ui| {
                    ui.label("Delimiter:");
                    let current_label = DELIMITER_OPTIONS
                        .iter()
                        .find(|(_, b)| *b == state.ingest_delimiter)
                        .map(|(l, _)| *l)
                        .unwrap_or("Comma (,)");
                    egui::ComboBox::from_id_salt("ingest_delim")
                        .selected_text(current_label)
                        .show_ui(ui, |ui| {
                            for (label, byte_val) in DELIMITER_OPTIONS {
                                if ui
                                    .selectable_value(
                                        &mut state.ingest_delimiter,
                                        *byte_val,
                                        *label,
                                    )
                                    .changed()
                                {
                                    options_changed = true;
                                }
                            }
                        });
                });

                ui.horizontal(|ui| {
                    if ui
                        .checkbox(&mut state.ingest_has_header, "First row is header")
                        .changed()
                    {
                        options_changed = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Skip rows:");
                    if ui
                        .add(
                            egui::DragValue::new(&mut state.ingest_skip_rows).range(0..=1000),
                        )
                        .changed()
                    {
                        options_changed = true;
                    }
                });

                if options_changed {
                    state.preview_ingest();
                }
            }

            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label("Table name:");
                ui.add(
                    egui::TextEdit::singleline(&mut state.ingest_table_name)
                        .desired_width(200.0)
                        .hint_text("auto-generated if empty"),
                );
            });

            ui.separator();

            if let Some(ref preview) = state.ingest_preview {
                ui.label(
                    egui::RichText::new(format!(
                        "Preview ({} rows, {} columns)",
                        preview.row_count,
                        preview.columns.len()
                    ))
                    .size(11.0)
                    .weak(),
                );
                egui::ScrollArea::both()
                    .max_height(280.0)
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        egui::Grid::new("ingest_preview_grid")
                            .striped(true)
                            .min_col_width(60.0)
                            .show(ui, |ui| {
                                for col in &preview.columns {
                                    ui.label(
                                        egui::RichText::new(col.as_str())
                                            .strong()
                                            .monospace()
                                            .size(11.0),
                                    );
                                }
                                ui.end_row();

                                for row in &preview.rows {
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
            } else {
                ui.label(egui::RichText::new("Loading preview...").weak());
            }

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    state.ingest_dialog = false;
                    state.ingest_preview = None;
                }
                if ui
                    .button(egui::RichText::new("Import").color(ACCENT).strong())
                    .clicked()
                {
                    state.commit_ingest();
                }
            });
        });
}
