use crate::facade::ColumnInfo;
use crate::state::AppState;
use crate::ui::theme::{ACCENT, BLUE, NULL_COLOR};
use egui::Ui;
use egui_extras::{Column, TableBuilder};

const ROW_HEIGHT: f32 = 24.0;
const ROW_NUM_WIDTH: f32 = 52.0;
const MIN_COL_WIDTH: f32 = 80.0;
const DEFAULT_COL_WIDTH: f32 = 120.0;

pub fn show_data_grid(ui: &mut Ui, state: &mut AppState) {
    let page_rows: Vec<Vec<String>> = match &state.current_page {
        Some(p) => p.rows.clone(),
        None => return,
    };

    let columns = state.columns.clone();
    if columns.is_empty() {
        return;
    }

    if state.features.formula_bar {
        show_formula_bar(ui, state, &page_rows, &columns);
        ui.separator();
    }

    let num_rows = page_rows.len();
    let current_offset = state.offset;
    let sort_column = state.sort_column.clone();
    let sort_desc = state.sort_desc;
    let total_rows = state.total_rows;
    let page_size = state.page_size;
    let has_column_ops = state.features.column_ops;

    let mut sort_action: Option<String> = None;
    let mut cell_click: Option<(usize, usize)> = None;
    let mut col_remove: Option<String> = None;
    let mut col_keep: Option<String> = None;
    let mut col_change_type: Option<(String, String)> = None;
    let mut col_rename: Option<String> = None;

    let available = ui.available_size();

    let table = TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .min_scrolled_height(0.0)
        .max_scroll_height(available.y - 30.0)
        .column(Column::initial(ROW_NUM_WIDTH).resizable(false))
        .columns(
            Column::initial(DEFAULT_COL_WIDTH)
                .at_least(MIN_COL_WIDTH)
                .resizable(true)
                .clip(true),
            columns.len(),
        );

    table
        .header(ROW_HEIGHT, |mut header| {
            header.col(|ui| {
                ui.label(egui::RichText::new("#").weak().monospace().size(10.0));
            });
            for col in &columns {
                header.col(|ui| {
                    let is_sorted = sort_column.as_deref() == Some(&col.name);
                    let arrow = if is_sorted {
                        if sort_desc {
                            " \u{25BC}"
                        } else {
                            " \u{25B2}"
                        }
                    } else {
                        ""
                    };
                    let text = format!("{}{}", col.name, arrow);
                    let label = egui::RichText::new(&text).strong().size(11.5);
                    let label = if is_sorted { label.color(BLUE) } else { label };

                    let response =
                        ui.add(egui::Label::new(label).sense(egui::Sense::click()));
                    if response.clicked() {
                        sort_action = Some(col.name.clone());
                    }

                    if has_column_ops {
                        let col_name = col.name.clone();
                        response.context_menu(|ui| {
                            ui.label(
                                egui::RichText::new(format!(
                                    "{} ({})",
                                    col.name, col.dtype
                                ))
                                .weak()
                                .size(10.0),
                            );
                            ui.separator();
                            if ui.button("Remove column").clicked() {
                                col_remove = Some(col_name.clone());
                                ui.close_menu();
                            }
                            if ui.button("Keep only this column").clicked() {
                                col_keep = Some(col_name.clone());
                                ui.close_menu();
                            }
                            ui.menu_button("Change type", |ui| {
                                for type_name in &[
                                    "VARCHAR", "INTEGER", "BIGINT", "DOUBLE", "BOOLEAN",
                                    "DATE", "TIMESTAMP",
                                ] {
                                    if ui.button(*type_name).clicked() {
                                        col_change_type = Some((
                                            col_name.clone(),
                                            type_name.to_string(),
                                        ));
                                        ui.close_menu();
                                    }
                                }
                            });
                            if ui.button("Rename column").clicked() {
                                col_rename = Some(col_name.clone());
                                ui.close_menu();
                            }
                        });
                    }
                });
            }
        })
        .body(|body| {
            body.rows(ROW_HEIGHT, num_rows, |mut row| {
                let row_idx = row.index();
                let global_row = current_offset + row_idx;

                row.col(|ui| {
                    ui.label(
                        egui::RichText::new(format!("{}", global_row + 1))
                            .weak()
                            .monospace()
                            .size(10.0),
                    );
                });

                for (col_idx, col) in columns.iter().enumerate() {
                    row.col(|ui| {
                        let val = page_rows[row_idx]
                            .get(col_idx)
                            .map(|s| s.as_str())
                            .unwrap_or("");
                        let is_null = val.is_empty();
                        let is_numeric = col.dtype.contains("Int")
                            || col.dtype.contains("Float")
                            || col.dtype.contains("UInt")
                            || col.dtype.contains("BIGINT")
                            || col.dtype.contains("INTEGER")
                            || col.dtype.contains("DOUBLE");

                        let text = if is_null { "NULL" } else { val };
                        let rich = egui::RichText::new(text).monospace().size(11.5);
                        let rich = if is_null {
                            rich.color(NULL_COLOR).italics()
                        } else if is_numeric {
                            rich.color(BLUE)
                        } else {
                            rich
                        };

                        let is_selected =
                            state.selected_cell == Some((row_idx, col_idx));
                        let response =
                            ui.add(egui::SelectableLabel::new(is_selected, rich));
                        if response.clicked() {
                            cell_click = Some((row_idx, col_idx));
                        }
                    });
                }
            });
        });

    if let Some(col_name) = sort_action {
        state.sort_by(&col_name);
        return;
    }
    if let Some((row, col)) = cell_click {
        state.selected_cell = Some((row, col));
    }
    if let Some(col_name) = col_remove {
        state.remove_column(&col_name);
        return;
    }
    if let Some(col_name) = col_keep {
        state.keep_columns_action(vec![col_name]);
        return;
    }
    if let Some((col_name, new_type)) = col_change_type {
        state.change_column_type_action(&col_name, &new_type);
        return;
    }
    if let Some(col_name) = col_rename {
        state.rename_old_name = col_name;
        state.rename_new_name.clear();
        state.rename_dialog = true;
        return;
    }

    // Pagination + Go to row
    ui.horizontal(|ui| {
        if !state.loading && current_offset > 0 {
            if ui.button("<< Previous").clicked() {
                state.load_page(current_offset.saturating_sub(page_size));
            }
        }
        if !state.loading && num_rows > 0 && current_offset + page_size < total_rows {
            if ui.button("Next >>").clicked() {
                state
                    .load_page((current_offset + page_size).min(total_rows.saturating_sub(1)));
            }
        }

        ui.separator();
        ui.label(egui::RichText::new("Go to row:").size(10.5));
        let response = ui.add(
            egui::TextEdit::singleline(&mut state.goto_row_input)
                .desired_width(60.0)
                .hint_text("row #"),
        );
        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            if let Ok(row_num) = state.goto_row_input.trim().parse::<usize>() {
                let target = row_num.saturating_sub(1).min(total_rows.saturating_sub(1));
                let page_start = (target / page_size) * page_size;
                state.load_page(page_start);
            }
        }
    });
}

fn show_formula_bar(
    ui: &mut Ui,
    state: &AppState,
    page_rows: &[Vec<String>],
    columns: &[ColumnInfo],
) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("fx")
                .monospace()
                .strong()
                .color(ACCENT)
                .size(12.0),
        );
        ui.separator();
        if let Some((row, col)) = state.selected_cell {
            if let Some(col_info) = columns.get(col) {
                ui.label(
                    egui::RichText::new(format!("{} [{}]:", col_info.name, col_info.dtype))
                        .monospace()
                        .size(11.0)
                        .weak(),
                );
            }
            let value = page_rows
                .get(row)
                .and_then(|r| r.get(col))
                .map(|s| s.as_str())
                .unwrap_or("");
            ui.label(
                egui::RichText::new(if value.is_empty() { "NULL" } else { value })
                    .monospace()
                    .size(11.5),
            );
        } else {
            ui.label(egui::RichText::new("Select a cell").weak().size(11.0));
        }
    });
}
