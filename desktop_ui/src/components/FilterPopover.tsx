import { useState, useRef, useEffect } from "react";

export interface FilterCondition {
  column: string;
  operator: string;
  value: string;
}

interface FilterPopoverProps {
  column: string;
  dtype: string;
  anchorRect: DOMRect;
  onApply: (conditions: FilterCondition[], logic: string) => void;
  onClose: () => void;
}

const TEXT_OPERATORS = [
  { value: "equals", label: "Equals" },
  { value: "not_equals", label: "Does not equal" },
  { value: "contains", label: "Contains" },
  { value: "not_contains", label: "Does not contain" },
  { value: "starts_with", label: "Starts with" },
  { value: "ends_with", label: "Ends with" },
  { value: "is_null", label: "Is empty" },
  { value: "is_not_null", label: "Is not empty" },
];

const NUMERIC_OPERATORS = [
  { value: "equals", label: "=" },
  { value: "not_equals", label: "!=" },
  { value: "greater_than", label: ">" },
  { value: "greater_than_or_equal", label: ">=" },
  { value: "less_than", label: "<" },
  { value: "less_than_or_equal", label: "<=" },
  { value: "is_null", label: "Is empty" },
  { value: "is_not_null", label: "Is not empty" },
];

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

  const [operator, setOperator] = useState(operators[0].value);
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
