use crate::state::AppState;
use crate::ui::theme::ACCENT;
use egui::Ui;

pub fn show_table_list(ui: &mut Ui, state: &mut AppState) {
    ui.vertical(|ui| {
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new("DATA")
                .size(10.0)
                .strong()
                .monospace()
                .weak(),
        );
        ui.add_space(4.0);
        ui.separator();

        let tables = state.tables.clone();
        let active = state.name.clone();
        let steps = &state.steps_entries;

        let (sources, queries): (Vec<&String>, Vec<&String>) =
            tables.iter().partition(|t| is_source_table(t, steps));

        let mut action: Option<TableAction> = None;

        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                if !sources.is_empty() {
                    ui.label(
                        egui::RichText::new("TABLES")
                            .size(9.0)
                            .strong()
                            .monospace()
                            .weak(),
                    );
                    for table in &sources {
                        show_table_row(ui, table, &active, &mut action);
                    }
                }

                if !queries.is_empty() {
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new("QUERIES")
                            .size(9.0)
                            .strong()
                            .monospace()
                            .weak(),
                    );
                    for table in &queries {
                        show_table_row(ui, table, &active, &mut action);
                    }
                }
            });

        match action {
            Some(TableAction::Select(name)) => state.select_table(&name),
            Some(TableAction::Remove(name)) => state.remove_dataset(&name),
            None => {}
        }
    });
}

fn show_table_row(
    ui: &mut egui::Ui,
    table: &str,
    active: &Option<String>,
    action: &mut Option<TableAction>,
) {
    let is_active = active.as_deref() == Some(table);

    ui.horizontal(|ui| {
        let label = egui::RichText::new(table).size(11.5).monospace();
        let label = if is_active {
            label.color(ACCENT).strong()
        } else {
            label
        };

        if ui
            .add(egui::SelectableLabel::new(is_active, label))
            .clicked()
        {
            *action = Some(TableAction::Select(table.to_string()));
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .small_button(egui::RichText::new("x").weak())
                .on_hover_text("Remove")
                .clicked()
            {
                *action = Some(TableAction::Remove(table.to_string()));
            }
        });
    });
}

fn is_source_table(
    name: &str,
    _steps: &[crate::facade::StepDisplayEntry],
) -> bool {
    !name.contains("_sorted")
        && !name.contains("_filtered_")
        && !name.contains("_grouped_")
        && !name.contains("_calc_")
        && !name.contains("_cols_")
        && !name.contains("_kept_")
        && !name.contains("_typed_")
        && !name.contains("_pivot_")
        && !name.contains("_unpivot_")
        && !name.starts_with("merged_")
        && !name.starts_with("appended_")
        && !name.starts_with("sql_result_")
}

enum TableAction {
    Select(String),
    Remove(String),
}
