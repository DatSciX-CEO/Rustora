/**
 * FilterPopover — inline column filter popover anchored below a column header button.
 *
 * Detects the column's data type (numeric vs. text) and shows the appropriate
 * set of operators. Applies the filter on "Apply" click or Enter key press.
 * Closes when clicking outside the popover or pressing Escape.
 */
import { useState, useRef, useEffect } from "react";
import { TEXT_OPERATORS, NUMERIC_OPERATORS } from "../constants/filterOperators";

export interface FilterCondition {
  column: string;
  operator: string;
  value: string;
}

interface FilterPopoverProps {
  /** Column name the filter applies to. */
  column: string;
  /** Data type string from Arrow/DuckDB (e.g. "Int64", "Utf8", "Float64"). */
  dtype: string;
  /** Bounding rect of the header button — used to position the popover. */
  anchorRect: DOMRect;
  /** Called with filter conditions when the user applies the filter. */
  onApply: (conditions: FilterCondition[], logic: string) => void;
  /** Called when the popover should be dismissed. */
  onClose: () => void;
}

function isNumericDtype(dtype: string): boolean {
  return (
    dtype.includes("Int") ||
    dtype.includes("Float") ||
    dtype.includes("UInt") ||
    dtype.includes("Decimal") ||
    dtype.includes("BIGINT") ||
    dtype.includes("INTEGER") ||
    dtype.includes("DOUBLE") ||
    dtype.includes("FLOAT")
  );
}

function isNullaryOp(op: string): boolean {
  return op === "is_null" || op === "is_not_null";
}

export function FilterPopover({
  column,
  dtype,
  anchorRect,
  onApply,
  onClose,
}: FilterPopoverProps) {
  const isNumeric = isNumericDtype(dtype);
  const operators = isNumeric ? NUMERIC_OPERATORS : TEXT_OPERATORS;

  const [operator, setOperator] = useState<string>(operators[0].value);
  const [value, setValue] = useState("");
  const popoverRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handler = (e: MouseEvent) => {
      if (
        popoverRef.current &&
        !popoverRef.current.contains(e.target as Node)
      ) {
        onClose();
      }
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, [onClose]);

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    document.addEventListener("keydown", handler);
    return () => document.removeEventListener("keydown", handler);
  }, [onClose]);

  const handleApply = () => {
    if (!isNullaryOp(operator) && !value.trim()) return;
    onApply(
      [{ column, operator, value: value.trim() }],
      "and"
    );
    onClose();
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") {
      e.preventDefault();
      handleApply();
    }
  };

  const left = Math.min(anchorRect.left, window.innerWidth - 280);
  const top = anchorRect.bottom + 4;

  return (
    <div
      ref={popoverRef}
      className="filter-popover"
      style={{ left, top }}
    >
      <div className="filter-popover-header">
        <span className="filter-popover-col">{column}</span>
        <span className="filter-popover-dtype">{dtype}</span>
      </div>

      <select
        className="filter-popover-select"
        value={operator}
        onChange={(e) => setOperator(e.target.value)}
      >
        {operators.map((op) => (
          <option key={op.value} value={op.value}>
            {op.label}
          </option>
        ))}
      </select>

      {!isNullaryOp(operator) && (
        <input
          className="filter-popover-input"
          type={isNumeric ? "number" : "text"}
          placeholder={isNumeric ? "Enter value..." : "Enter text..."}
          value={value}
          onChange={(e) => setValue(e.target.value)}
          onKeyDown={handleKeyDown}
          autoFocus
        />
      )}

      <div className="filter-popover-actions">
        <button className="filter-popover-btn-cancel" onClick={onClose}>
          Cancel
        </button>
        <button
          className="filter-popover-btn-apply"
          disabled={!isNullaryOp(operator) && !value.trim()}
          onClick={handleApply}
        >
          Apply
        </button>
      </div>
    </div>
  );
}
