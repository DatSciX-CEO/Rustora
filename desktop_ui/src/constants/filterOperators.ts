/**
 * Filter operator definitions for text (string) columns.
 * Each entry maps a value (sent to Rust) to a display label.
 */
export const TEXT_OPERATORS = [
  { value: "equals", label: "Equals" },
  { value: "not_equals", label: "Does not equal" },
  { value: "contains", label: "Contains" },
  { value: "not_contains", label: "Does not contain" },
  { value: "starts_with", label: "Starts with" },
  { value: "ends_with", label: "Ends with" },
  { value: "is_null", label: "Is empty" },
  { value: "is_not_null", label: "Is not empty" },
] as const;

/**
 * Filter operator definitions for numeric columns.
 * Each entry maps a value (sent to Rust) to a display label.
 */
export const NUMERIC_OPERATORS = [
  { value: "equals", label: "=" },
  { value: "not_equals", label: "!=" },
  { value: "greater_than", label: ">" },
  { value: "greater_than_or_equal", label: ">=" },
  { value: "less_than", label: "<" },
  { value: "less_than_or_equal", label: "<=" },
  { value: "is_null", label: "Is empty" },
  { value: "is_not_null", label: "Is not empty" },
] as const;

export type FilterOperatorValue =
  | (typeof TEXT_OPERATORS)[number]["value"]
  | (typeof NUMERIC_OPERATORS)[number]["value"];
