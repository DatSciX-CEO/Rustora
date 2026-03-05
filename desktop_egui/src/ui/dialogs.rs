use crate::state::AppState;

const TEXT_OPERATORS: &[(&str, &str)] = &[
    ("equals", "Equals"),
    ("not_equals", "Not Equals"),
    ("contains", "Contains"),
    ("not_contains", "Not Contains"),
    ("starts_with", "Starts With"),
    ("ends_with", "Ends With"),
    ("is_null", "Is Null"),
    ("is_not_null", "Is Not Null"),
];

const NUMERIC_OPERATORS: &[(&str, &str)] = &[
    ("equals", "Equals"),
    ("not_equals", "Not Equals"),
    ("greater_than", "Greater Than"),
    ("greater_than_or_equal", ">="),
    ("less_than", "Less Than"),
    ("less_than_or_equal", "<="),
    ("is_null", "Is Null"),
    ("is_not_null", "Is Not Null"),
];

fn is_numeric_dtype(dtype: &str) -> bool {
    dtype.contains("Int")
        || dtype.contains("Float")
        || dtype.contains("UInt")
        || dtype.contains("Decimal")
        || dtype.contains("BIGINT")
        || dtype.contains("INTEGER")
        || dtype.contains("DOUBLE")
}

fn is_nullary_op(op: &str) -> bool {
    op == "is_null" || op == "is_not_null"
}

pub fn show_dialogs(ctx: &egui::Context, state: &mut AppState) {
    show_filter_dialog(ctx, state);
    show_group_by_dialog(ctx, state);
    show_calc_column_dialog(ctx, state);
    if state.features.pivot_unpivot {
        show_pivot_dialog(ctx, state);
        show_unpivot_dialog(ctx, state);
    }
    if state.features.merge_append {
        show_merge_dialog(ctx, state);
        show_append_dialog(ctx, state);
    }
    if state.features.column_ops {
        show_rename_column_dialog(ctx, state);
    }
}

fn show_filter_dialog(ctx: &egui::Context, state: &mut AppState) {
    if !state.filter_dialog {
        return;
    }

    egui::Window::new("Filter Dataset")
        .collapsible(false)
        .resizable(true)
        .default_width(500.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Mode:").strong());
                ui.selectable_value(&mut state.filter_mode_structured, false, "SQL WHERE");
                ui.selectable_value(&mut state.filter_mode_structured, true, "Column Filter");
            });
            ui.separator();

            if state.filter_mode_structured {
                show_structured_filter(ui, state);
            } else {
                show_sql_filter(ui, state);
            }
        });
}

fn show_sql_filter(ui: &mut egui::Ui, state: &mut AppState) {
    ui.label("Enter a SQL WHERE clause:");
    if !state.columns.is_empty() {
        ui.label(
            egui::RichText::new(format!(
                "Columns: {}",
                state
                    .columns
                    .iter()
                    .map(|c| c.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
            .weak()
            .size(11.0)
            .monospace(),
        );
    }
    ui.add_space(4.0);

    ui.add(
        egui::TextEdit::multiline(&mut state.filter_where)
            .desired_rows(2)
            .desired_width(400.0)
            .font(egui::TextStyle::Monospace)
            .hint_text("age > 30 AND city = 'Boston'"),
    );

    ui.add_space(8.0);
    ui.horizontal(|ui| {
        if ui.button("Cancel").clicked() {
            state.filter_dialog = false;
            state.filter_where.clear();
        }
        if ui
            .add_enabled(
                !state.filter_where.trim().is_empty(),
                egui::Button::new("Apply Filter"),
            )
            .clicked()
        {
            state.filter_dataset_sql();
        }
    });
}

fn show_structured_filter(ui: &mut egui::Ui, state: &mut AppState) {
    let columns = state.columns.clone();
    if columns.is_empty() {
        ui.label("No columns available");
        return;
    }

    // Ensure defaults are set
    if state.struct_filter_column.is_empty() {
        state.struct_filter_column = columns[0].name.clone();
    }

    ui.horizontal(|ui| {
        ui.label("Column:");
        egui::ComboBox::from_id_salt("filter_col")
            .selected_text(&state.struct_filter_column)
            .show_ui(ui, |ui| {
                for col in &columns {
                    ui.selectable_value(
                        &mut state.struct_filter_column,
                        col.name.clone(),
                        format!("{} ({})", col.name, col.dtype),
                    );
                }
            });
    });

    let selected_dtype = columns
        .iter()
        .find(|c| c.name == state.struct_filter_column)
        .map(|c| c.dtype.as_str())
        .unwrap_or("");
    let is_numeric = is_numeric_dtype(selected_dtype);
    let operators = if is_numeric {
        NUMERIC_OPERATORS
    } else {
        TEXT_OPERATORS
    };

    ui.horizontal(|ui| {
        ui.label("Operator:");
        let current_label = operators
            .iter()
            .find(|(v, _)| *v == state.struct_filter_operator)
            .map(|(_, l)| *l)
            .unwrap_or(operators[0].1);

        egui::ComboBox::from_id_salt("filter_op")
            .selected_text(current_label)
            .show_ui(ui, |ui| {
                for (val, label) in operators {
                    ui.selectable_value(
                        &mut state.struct_filter_operator,
                        val.to_string(),
                        *label,
                    );
                }
            });
    });

    if !is_nullary_op(&state.struct_filter_operator) {
        ui.horizontal(|ui| {
            ui.label("Value:");
            ui.add(
                egui::TextEdit::singleline(&mut state.struct_filter_value)
                    .desired_width(200.0)
                    .hint_text(if is_numeric {
                        "Enter number..."
                    } else {
                        "Enter text..."
                    }),
            );
        });
    }

    ui.add_space(8.0);
    ui.horizontal(|ui| {
        if ui.button("Cancel").clicked() {
            state.filter_dialog = false;
            state.struct_filter_value.clear();
        }
        let can_apply = is_nullary_op(&state.struct_filter_operator)
            || !state.struct_filter_value.trim().is_empty();
        if ui
            .add_enabled(can_apply, egui::Button::new("Apply Filter"))
            .clicked()
        {
            state.apply_structured_filter();
        }
    });
}

fn show_group_by_dialog(ctx: &egui::Context, state: &mut AppState) {
    if !state.group_dialog {
        return;
    }

    egui::Window::new("Group By")
        .collapsible(false)
        .resizable(true)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label("Group columns (comma-separated):");
            ui.add(
                egui::TextEdit::singleline(&mut state.group_cols_input)
                    .desired_width(400.0)
                    .font(egui::TextStyle::Monospace)
                    .hint_text("city, department"),
            );

            ui.add_space(4.0);
            ui.label("Aggregate expressions (comma-separated):");
            ui.add(
                egui::TextEdit::singleline(&mut state.group_aggs_input)
                    .desired_width(400.0)
                    .font(egui::TextStyle::Monospace)
                    .hint_text("COUNT(*), AVG(salary)"),
            );

            if !state.columns.is_empty() {
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(format!(
                        "Available: {}",
                        state
                            .columns
                            .iter()
                            .map(|c| c.name.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ))
                    .weak()
                    .size(10.0)
                    .monospace(),
                );
            }

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    state.group_dialog = false;
                    state.group_cols_input.clear();
                    state.group_aggs_input.clear();
                }
                let can_apply = !state.group_cols_input.trim().is_empty()
                    && !state.group_aggs_input.trim().is_empty();
                if ui
                    .add_enabled(can_apply, egui::Button::new("Apply Group By"))
                    .clicked()
                {
                    state.group_by_dataset();
                }
            });
        });
}

fn show_calc_column_dialog(ctx: &egui::Context, state: &mut AppState) {
    if !state.calc_dialog {
        return;
    }

    egui::Window::new("Add Calculated Column")
        .collapsible(false)
        .resizable(true)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label("SQL expression:");
            ui.add(
                egui::TextEdit::singleline(&mut state.calc_expr)
                    .desired_width(400.0)
                    .font(egui::TextStyle::Monospace)
                    .hint_text("salary * 12"),
            );

            ui.add_space(4.0);
            ui.label("Column name (alias):");
            ui.add(
                egui::TextEdit::singleline(&mut state.calc_alias)
                    .desired_width(400.0)
                    .font(egui::TextStyle::Monospace)
                    .hint_text("annual_salary"),
            );

            if !state.columns.is_empty() {
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(format!(
                        "Columns: {}",
                        state
                            .columns
                            .iter()
                            .map(|c| c.name.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ))
                    .weak()
                    .size(10.0)
                    .monospace(),
                );
            }

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    state.calc_dialog = false;
                    state.calc_expr.clear();
                    state.calc_alias.clear();
                }
                let can_apply =
                    !state.calc_expr.trim().is_empty() && !state.calc_alias.trim().is_empty();
                if ui
                    .add_enabled(can_apply, egui::Button::new("Add Column"))
                    .clicked()
                {
                    state.add_calculated_column();
                }
            });
        });
}

fn show_pivot_dialog(ctx: &egui::Context, state: &mut AppState) {
    if !state.pivot_dialog {
        return;
    }

    egui::Window::new("Pivot Table")
        .collapsible(false)
        .resizable(true)
        .default_width(500.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label("Index columns (comma-separated, kept as rows):");
            ui.add(
                egui::TextEdit::singleline(&mut state.pivot_index_cols)
                    .desired_width(400.0)
                    .font(egui::TextStyle::Monospace)
                    .hint_text("department"),
            );

            ui.add_space(4.0);
            ui.label("Pivot column (values become new columns):");
            egui::ComboBox::from_id_salt("pivot_col")
                .selected_text(if state.pivot_col.is_empty() {
                    "Select..."
                } else {
                    &state.pivot_col
                })
                .show_ui(ui, |ui| {
                    for col in &state.columns.clone() {
                        ui.selectable_value(
                            &mut state.pivot_col,
                            col.name.clone(),
                            &col.name,
                        );
                    }
                });

            ui.add_space(4.0);
            ui.label("Value column (aggregated values):");
            egui::ComboBox::from_id_salt("pivot_val")
                .selected_text(if state.pivot_value_col.is_empty() {
                    "Select..."
                } else {
                    &state.pivot_value_col
                })
                .show_ui(ui, |ui| {
                    for col in &state.columns.clone() {
                        ui.selectable_value(
                            &mut state.pivot_value_col,
                            col.name.clone(),
                            &col.name,
                        );
                    }
                });

            ui.add_space(4.0);
            ui.label("Aggregation:");
            egui::ComboBox::from_id_salt("pivot_agg")
                .selected_text(&state.pivot_agg)
                .show_ui(ui, |ui| {
                    for agg in &["sum", "avg", "count", "min", "max"] {
                        ui.selectable_value(&mut state.pivot_agg, agg.to_string(), *agg);
                    }
                });

            show_columns_hint(ui, state);

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    state.pivot_dialog = false;
                }
                let can_apply = !state.pivot_index_cols.trim().is_empty()
                    && !state.pivot_col.is_empty()
                    && !state.pivot_value_col.is_empty();
                if ui
                    .add_enabled(can_apply, egui::Button::new("Apply Pivot"))
                    .clicked()
                {
                    state.pivot_dataset();
                }
            });
        });
}

fn show_unpivot_dialog(ctx: &egui::Context, state: &mut AppState) {
    if !state.unpivot_dialog {
        return;
    }

    egui::Window::new("Unpivot (Columns to Rows)")
        .collapsible(false)
        .resizable(true)
        .default_width(500.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label("Value columns to unpivot (comma-separated):");
            ui.add(
                egui::TextEdit::singleline(&mut state.unpivot_value_cols)
                    .desired_width(400.0)
                    .font(egui::TextStyle::Monospace)
                    .hint_text("jan, feb, mar"),
            );

            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label("Variable name:");
                ui.add(
                    egui::TextEdit::singleline(&mut state.unpivot_var_name)
                        .desired_width(150.0)
                        .hint_text("variable"),
                );
            });

            ui.horizontal(|ui| {
                ui.label("Value name:");
                ui.add(
                    egui::TextEdit::singleline(&mut state.unpivot_value_name)
                        .desired_width(150.0)
                        .hint_text("value"),
                );
            });

            show_columns_hint(ui, state);

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    state.unpivot_dialog = false;
                }
                if ui
                    .add_enabled(
                        !state.unpivot_value_cols.trim().is_empty(),
                        egui::Button::new("Apply Unpivot"),
                    )
                    .clicked()
                {
                    state.unpivot_dataset();
                }
            });
        });
}

fn show_merge_dialog(ctx: &egui::Context, state: &mut AppState) {
    if !state.merge_dialog {
        return;
    }

    egui::Window::new("Merge (Join) Tables")
        .collapsible(false)
        .resizable(true)
        .default_width(500.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label("Right table:");
            let tables = state.tables.clone();
            let mut table_changed = false;
            egui::ComboBox::from_id_salt("merge_right")
                .selected_text(if state.merge_right_table.is_empty() {
                    "Select..."
                } else {
                    &state.merge_right_table
                })
                .show_ui(ui, |ui| {
                    for t in &tables {
                        if ui
                            .selectable_value(
                                &mut state.merge_right_table,
                                t.clone(),
                                t.as_str(),
                            )
                            .changed()
                        {
                            table_changed = true;
                        }
                    }
                });
            if table_changed {
                state.load_merge_right_columns();
            }

            ui.add_space(4.0);
            ui.label("Left join column:");
            egui::ComboBox::from_id_salt("merge_lcol")
                .selected_text(if state.merge_left_col.is_empty() {
                    "Select..."
                } else {
                    &state.merge_left_col
                })
                .show_ui(ui, |ui| {
                    for col in &state.columns.clone() {
                        ui.selectable_value(
                            &mut state.merge_left_col,
                            col.name.clone(),
                            &col.name,
                        );
                    }
                });

            ui.label("Right join column:");
            let right_cols = state.merge_right_columns.clone();
            egui::ComboBox::from_id_salt("merge_rcol")
                .selected_text(if state.merge_right_col.is_empty() {
                    "Select..."
                } else {
                    &state.merge_right_col
                })
                .show_ui(ui, |ui| {
                    for col in &right_cols {
                        ui.selectable_value(
                            &mut state.merge_right_col,
                            col.name.clone(),
                            &col.name,
                        );
                    }
                });

            ui.add_space(4.0);
            ui.label("Join type:");
            egui::ComboBox::from_id_salt("merge_type")
                .selected_text(&state.merge_join_type)
                .show_ui(ui, |ui| {
                    for jt in &["inner", "left", "right", "full"] {
                        ui.selectable_value(
                            &mut state.merge_join_type,
                            jt.to_string(),
                            *jt,
                        );
                    }
                });

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    state.merge_dialog = false;
                }
                let can_apply = !state.merge_right_table.is_empty()
                    && !state.merge_left_col.is_empty()
                    && !state.merge_right_col.is_empty();
                if ui
                    .add_enabled(can_apply, egui::Button::new("Merge"))
                    .clicked()
                {
                    state.merge_datasets();
                }
            });
        });
}

fn show_append_dialog(ctx: &egui::Context, state: &mut AppState) {
    if !state.append_dialog {
        return;
    }

    egui::Window::new("Append (Union) Tables")
        .collapsible(false)
        .resizable(true)
        .default_width(500.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label("Tables to append (comma-separated):");
            ui.add(
                egui::TextEdit::singleline(&mut state.append_tables_input)
                    .desired_width(400.0)
                    .font(egui::TextStyle::Monospace)
                    .hint_text("table_a, table_b"),
            );

            if !state.tables.is_empty() {
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(format!(
                        "Available: {}",
                        state.tables.join(", ")
                    ))
                    .weak()
                    .size(10.0)
                    .monospace(),
                );
            }

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    state.append_dialog = false;
                }
                if ui
                    .add_enabled(
                        !state.append_tables_input.trim().is_empty(),
                        egui::Button::new("Append"),
                    )
                    .clicked()
                {
                    state.append_datasets();
                }
            });
        });
}

fn show_rename_column_dialog(ctx: &egui::Context, state: &mut AppState) {
    if !state.rename_dialog {
        return;
    }
    egui::Window::new("Rename Column")
        .collapsible(false)
        .resizable(false)
        .default_width(350.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            show_columns_hint(ui, state);
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label("Current name:");
                egui::ComboBox::from_id_salt("rename_old")
                    .selected_text(&state.rename_old_name)
                    .width(180.0)
                    .show_ui(ui, |ui| {
                        let cols: Vec<String> = state.columns.iter().map(|c| c.name.clone()).collect();
                        for c in cols {
                            ui.selectable_value(&mut state.rename_old_name, c.clone(), &c);
                        }
                    });
            });
            ui.horizontal(|ui| {
                ui.label("New name:");
                ui.text_edit_singleline(&mut state.rename_new_name);
            });
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    state.rename_dialog = false;
                }
                if ui.button("Rename").clicked() {
                    state.rename_column_action();
                }
            });
        });
}

fn show_columns_hint(ui: &mut egui::Ui, state: &AppState) {
    if !state.columns.is_empty() {
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new(format!(
                "Columns: {}",
                state
                    .columns
                    .iter()
                    .map(|c| c.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
            .weak()
            .size(10.0)
            .monospace(),
        );
    }
}
