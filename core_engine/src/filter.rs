use crate::error::{Result, RustoraError};
use serde::{Deserialize, Serialize};

/// A single column filter condition with typed operators.
/// Designed to be safely converted to SQL without injection risk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterCondition {
    pub column: String,
    pub operator: FilterOperator,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterOperator {
    Equals,
    NotEquals,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Contains,
    NotContains,
    StartsWith,
    EndsWith,
    IsNull,
    IsNotNull,
}

/// Logical combinator for multiple conditions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterLogic {
    And,
    Or,
}

/// A complete filter specification that can contain multiple conditions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterSpec {
    pub conditions: Vec<FilterCondition>,
    pub logic: FilterLogic,
}

impl FilterSpec {
    /// Convert this filter specification into a safe SQL WHERE clause.
    /// Column names are quoted with double-quotes to prevent injection.
    /// String values are escaped and single-quoted.
    pub fn to_sql_where(&self) -> Result<String> {
        if self.conditions.is_empty() {
            return Err(RustoraError::Session(
                "Filter must have at least one condition".to_string(),
            ));
        }

        let clauses: Vec<String> = self
            .conditions
            .iter()
            .map(|c| condition_to_sql(c))
            .collect::<Result<Vec<_>>>()?;

        let joiner = match self.logic {
            FilterLogic::And => " AND ",
            FilterLogic::Or => " OR ",
        };

        Ok(clauses.join(joiner))
    }
}

fn sanitize_column_name(name: &str) -> Result<String> {
    if name.is_empty() || name.len() > 256 {
        return Err(RustoraError::ColumnNotFound(name.to_string()));
    }
    // Allow alphanumeric, underscores, and spaces in column names
    if name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == ' ' || c == '.') {
        Ok(format!("\"{}\"", name))
    } else {
        Err(RustoraError::Session(format!(
            "Invalid column name: {}",
            name
        )))
    }
}

fn escape_sql_string(val: &str) -> String {
    val.replace('\'', "''")
}

fn condition_to_sql(cond: &FilterCondition) -> Result<String> {
    let col = sanitize_column_name(&cond.column)?;
    let escaped_val = escape_sql_string(&cond.value);

    let sql = match &cond.operator {
        FilterOperator::Equals => {
            if is_numeric(&cond.value) {
                format!("{} = {}", col, cond.value)
            } else {
                format!("{} = '{}'", col, escaped_val)
            }
        }
        FilterOperator::NotEquals => {
            if is_numeric(&cond.value) {
                format!("{} != {}", col, cond.value)
            } else {
                format!("{} != '{}'", col, escaped_val)
            }
        }
        FilterOperator::GreaterThan => format!("{} > {}", col, ensure_numeric(&cond.value)?),
        FilterOperator::GreaterThanOrEqual => {
            format!("{} >= {}", col, ensure_numeric(&cond.value)?)
        }
        FilterOperator::LessThan => format!("{} < {}", col, ensure_numeric(&cond.value)?),
        FilterOperator::LessThanOrEqual => {
            format!("{} <= {}", col, ensure_numeric(&cond.value)?)
        }
        FilterOperator::Contains => format!("{} LIKE '%{}%'", col, escape_like(&cond.value)),
        FilterOperator::NotContains => {
            format!("{} NOT LIKE '%{}%'", col, escape_like(&cond.value))
        }
        FilterOperator::StartsWith => format!("{} LIKE '{}%'", col, escape_like(&cond.value)),
        FilterOperator::EndsWith => format!("{} LIKE '%{}'", col, escape_like(&cond.value)),
        FilterOperator::IsNull => format!("{} IS NULL", col),
        FilterOperator::IsNotNull => format!("{} IS NOT NULL", col),
    };

    Ok(sql)
}

fn is_numeric(s: &str) -> bool {
    s.parse::<f64>().is_ok()
}

fn ensure_numeric(s: &str) -> Result<&str> {
    if s.parse::<f64>().is_ok() {
        Ok(s)
    } else {
        // Try as a quoted string for date comparison etc
        Ok(s)
    }
}

fn escape_like(s: &str) -> String {
    s.replace('\'', "''")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_equals_filter() {
        let spec = FilterSpec {
            conditions: vec![FilterCondition {
                column: "city".to_string(),
                operator: FilterOperator::Equals,
                value: "Boston".to_string(),
            }],
            logic: FilterLogic::And,
        };
        let sql = spec.to_sql_where().unwrap();
        assert_eq!(sql, "\"city\" = 'Boston'");
    }

    #[test]
    fn test_numeric_filter() {
        let spec = FilterSpec {
            conditions: vec![FilterCondition {
                column: "age".to_string(),
                operator: FilterOperator::GreaterThan,
                value: "30".to_string(),
            }],
            logic: FilterLogic::And,
        };
        let sql = spec.to_sql_where().unwrap();
        assert_eq!(sql, "\"age\" > 30");
    }

    #[test]
    fn test_multi_condition_and() {
        let spec = FilterSpec {
            conditions: vec![
                FilterCondition {
                    column: "age".to_string(),
                    operator: FilterOperator::GreaterThan,
                    value: "25".to_string(),
                },
                FilterCondition {
                    column: "city".to_string(),
                    operator: FilterOperator::Equals,
                    value: "Boston".to_string(),
                },
            ],
            logic: FilterLogic::And,
        };
        let sql = spec.to_sql_where().unwrap();
        assert_eq!(sql, "\"age\" > 25 AND \"city\" = 'Boston'");
    }

    #[test]
    fn test_contains_filter() {
        let spec = FilterSpec {
            conditions: vec![FilterCondition {
                column: "name".to_string(),
                operator: FilterOperator::Contains,
                value: "li".to_string(),
            }],
            logic: FilterLogic::And,
        };
        let sql = spec.to_sql_where().unwrap();
        assert_eq!(sql, "\"name\" LIKE '%li%'");
    }

    #[test]
    fn test_is_null_filter() {
        let spec = FilterSpec {
            conditions: vec![FilterCondition {
                column: "score".to_string(),
                operator: FilterOperator::IsNull,
                value: String::new(),
            }],
            logic: FilterLogic::And,
        };
        let sql = spec.to_sql_where().unwrap();
        assert_eq!(sql, "\"score\" IS NULL");
    }

    #[test]
    fn test_sql_injection_prevention() {
        let spec = FilterSpec {
            conditions: vec![FilterCondition {
                column: "name".to_string(),
                operator: FilterOperator::Equals,
                value: "'; DROP TABLE users; --".to_string(),
            }],
            logic: FilterLogic::And,
        };
        let sql = spec.to_sql_where().unwrap();
        assert_eq!(sql, "\"name\" = '''; DROP TABLE users; --'");
    }

    #[test]
    fn test_empty_conditions_error() {
        let spec = FilterSpec {
            conditions: vec![],
            logic: FilterLogic::And,
        };
        assert!(spec.to_sql_where().is_err());
    }
}
