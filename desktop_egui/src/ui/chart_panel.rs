use crate::state::{AppState, ChartType};
use crate::ui::theme::{ACCENT, CHART_COLORS};
use egui_plot::{Bar, BarChart, Line, Plot, PlotPoints};

const AGG_TYPES: &[(&str, &str)] = &[
    ("count", "Count"),
    ("sum", "Sum"),
    ("avg", "Average"),
    ("min", "Min"),
    ("max", "Max"),
];

pub fn show_chart_panel(ctx: &egui::Context, state: &mut AppState) {
    if !state.chart_visible {
        return;
    }

    egui::Window::new("Chart Builder")
        .collapsible(false)
        .resizable(true)
        .default_size([800.0, 500.0])
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Config sidebar
                ui.vertical(|ui| {
                    ui.set_width(200.0);

                    ui.label(egui::RichText::new("CHART TYPE").strong().size(10.0).weak());
                    ui.horizontal(|ui| {
                        if ui
                            .selectable_label(state.chart_type == ChartType::Bar, "Bar")
                            .clicked()
                        {
                            state.chart_type = ChartType::Bar;
                        }
                        if ui
                            .selectable_label(state.chart_type == ChartType::Line, "Line")
                            .clicked()
                        {
                            state.chart_type = ChartType::Line;
                        }
                    });

                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("GROUP BY").strong().size(10.0).weak());
                    egui::ComboBox::from_id_salt("chart_group")
                        .selected_text(&state.chart_group_col)
                        .show_ui(ui, |ui| {
                            for col in &state.columns {
                                ui.selectable_value(
                                    &mut state.chart_group_col,
                                    col.name.clone(),
                                    format!("{} ({})", col.name, col.dtype),
                                );
                            }
                        });

                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("AGGREGATION").strong().size(10.0).weak());
                    egui::ComboBox::from_id_salt("chart_agg")
                        .selected_text(
                            AGG_TYPES
                                .iter()
                                .find(|(v, _)| *v == state.chart_agg_type)
                                .map(|(_, l)| *l)
                                .unwrap_or("Count"),
                        )
                        .show_ui(ui, |ui| {
                            for (val, label) in AGG_TYPES {
                                ui.selectable_value(
                                    &mut state.chart_agg_type,
                                    val.to_string(),
                                    *label,
                                );
                            }
                        });

                    if state.chart_agg_type != "count" {
                        ui.add_space(8.0);
                        ui.label(egui::RichText::new("VALUE COLUMN").strong().size(10.0).weak());
                        egui::ComboBox::from_id_salt("chart_val")
                            .selected_text(&state.chart_value_col)
                            .show_ui(ui, |ui| {
                                for col in &state.columns {
                                    ui.selectable_value(
                                        &mut state.chart_value_col,
                                        col.name.clone(),
                                        format!("{} ({})", col.name, col.dtype),
                                    );
                                }
                            });
                    }

                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("MAX GROUPS").strong().size(10.0).weak());
                    ui.add(egui::DragValue::new(&mut state.chart_limit).range(5..=100));

                    ui.add_space(12.0);
                    if ui
                        .add_enabled(
                            !state.chart_group_col.is_empty(),
                            egui::Button::new(
                                egui::RichText::new("Generate Chart").color(ACCENT).strong(),
                            ),
                        )
                        .clicked()
                    {
                        state.generate_chart();
                    }

                    if let Some(ref err) = state.chart_error {
                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new(err.as_str())
                                .color(crate::ui::theme::ERROR)
                                .size(11.0),
                        );
                    }
                });

                ui.separator();

                // Chart display
                ui.vertical(|ui| {
                    if let Some(ref data) = state.chart_data {
                        if data.is_empty() {
                            ui.centered_and_justified(|ui| {
                                ui.label("No data returned");
                            });
                        } else {
                            match state.chart_type {
                                ChartType::Bar => {
                                    let bars: Vec<Bar> = data
                                        .iter()
                                        .enumerate()
                                        .map(|(i, r)| {
                                            Bar::new(i as f64, r.value)
                                                .name(&r.label)
                                                .fill(CHART_COLORS[i % CHART_COLORS.len()])
                                        })
                                        .collect();

                                    Plot::new("bar_chart")
                                        .allow_drag(false)
                                        .allow_zoom(false)
                                        .show(ui, |plot_ui| {
                                            plot_ui.bar_chart(BarChart::new(bars));
                                        });
                                }
                                ChartType::Line => {
                                    let points: PlotPoints = data
                                        .iter()
                                        .enumerate()
                                        .map(|(i, r)| [i as f64, r.value])
                                        .collect();

                                    Plot::new("line_chart")
                                        .allow_drag(false)
                                        .allow_zoom(false)
                                        .show(ui, |plot_ui| {
                                            plot_ui.line(
                                                Line::new(points)
                                                    .color(ACCENT)
                                                    .width(2.0),
                                            );
                                        });
                                }
                            }
                        }
                    } else {
                        ui.centered_and_justified(|ui| {
                            ui.label(
                                egui::RichText::new("Configure options and click Generate")
                                    .weak(),
                            );
                        });
                    }
                });
            });

            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if ui.button("Close").clicked() {
                    state.chart_visible = false;
                }
            });
        });
}
