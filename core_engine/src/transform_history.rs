use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransformStep {
    Source { file_path: String },
    Sort { columns: Vec<String>, descending: Vec<bool> },
    Filter { where_clause: String },
    GroupBy { group_columns: Vec<String>, agg_exprs: Vec<String> },
    AddColumn { expression: String, alias: String },
    RemoveColumns { columns: Vec<String> },
    KeepColumns { columns: Vec<String> },
    ChangeType { column: String, new_type: String },
    RenameColumn { old_name: String, new_name: String },
    Pivot { index_cols: Vec<String>, pivot_col: String, value_col: String, agg: String },
    Unpivot { id_cols: Vec<String>, value_cols: Vec<String>, var_name: String, value_name: String },
    Merge { right_table: String, left_col: String, right_col: String, join_type: String },
    Append { tables: Vec<String> },
    Sql { query: String },
}

impl TransformStep {
    pub fn label(&self) -> String {
        match self {
            Self::Source { file_path } => {
                let stem = std::path::Path::new(file_path)
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or(file_path);
                format!("Source: {}", stem)
            }
            Self::Sort { columns, descending } => {
                let dirs: Vec<String> = columns.iter().zip(descending.iter())
                    .map(|(c, d)| format!("{} {}", c, if *d { "DESC" } else { "ASC" }))
                    .collect();
                format!("Sorted: {}", dirs.join(", "))
            }
            Self::Filter { where_clause } => {
                let s = if where_clause.len() > 50 { &where_clause[..50] } else { where_clause };
                format!("Filtered: {}", s)
            }
            Self::GroupBy { group_columns, agg_exprs } => {
                format!("Grouped: {} | {}", group_columns.join(", "), agg_exprs.join(", "))
            }
            Self::AddColumn { alias, .. } => format!("Added column: {}", alias),
            Self::RemoveColumns { columns } => format!("Removed: {}", columns.join(", ")),
            Self::KeepColumns { columns } => format!("Kept: {}", columns.join(", ")),
            Self::ChangeType { column, new_type } => format!("Type: {} -> {}", column, new_type),
            Self::RenameColumn { old_name, new_name } => format!("Renamed: {} -> {}", old_name, new_name),
            Self::Pivot { pivot_col, .. } => format!("Pivot on: {}", pivot_col),
            Self::Unpivot { value_cols, .. } => format!("Unpivot: {}", value_cols.join(", ")),
            Self::Merge { right_table, join_type, .. } => format!("Merge: {} ({})", right_table, join_type),
            Self::Append { tables } => format!("Append: {}", tables.join(", ")),
            Self::Sql { query } => {
                let s = if query.len() > 40 { &query[..40] } else { query };
                format!("SQL: {}", s)
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepEntry {
    pub step: TransformStep,
    pub result_table: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TransformHistory {
    entries: Vec<StepEntry>,
}

impl TransformHistory {
    pub fn new() -> Self { Self { entries: Vec::new() } }

    pub fn push(&mut self, step: TransformStep, result_table: String) {
        self.entries.push(StepEntry { step, result_table });
    }

    pub fn entries(&self) -> &[StepEntry] { &self.entries }
    pub fn len(&self) -> usize { self.entries.len() }
    pub fn is_empty(&self) -> bool { self.entries.is_empty() }

    pub fn table_at(&self, index: usize) -> Option<&str> {
        self.entries.get(index).map(|e| e.result_table.as_str())
    }
}
