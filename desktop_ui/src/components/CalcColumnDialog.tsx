/**
 * CalcColumnDialog — modal for adding a new calculated (derived) column via a SQL expression.
 *
 * The user enters an alias (new column name) and a SQL expression (e.g. salary * 12).
 * The expression is evaluated by DuckDB, so any valid SQL scalar expression is supported.
 */
import { useState } from "react";

interface CalcColumnDialogProps {
  /** Called with the SQL expression and alias when the user confirms. */
  onSubmit: (expr: string, alias: string) => void;
  /** Called when the dialog should be closed without submitting. */
  onClose: () => void;
}

export function CalcColumnDialog({ onSubmit, onClose }: CalcColumnDialogProps) {
  const [expr, setExpr] = useState("");
  const [alias, setAlias] = useState("");

  return (
    <div className="dialog-overlay" onClick={onClose}>
      <div className="dialog" onClick={(e) => e.stopPropagation()}>
        <h3>Add Calculated Column</h3>
        <p className="dialog-help">Column Name:</p>
        <input
          className="dialog-input-single"
          placeholder="e.g. annual_salary"
          value={alias}
          onChange={(e) => setAlias(e.target.value)}
          autoFocus
        />
        <p className="dialog-help" style={{ marginTop: 8 }}>SQL Expression:</p>
        <textarea
          className="dialog-input"
          placeholder="e.g. salary * 12"
          value={expr}
          onChange={(e) => setExpr(e.target.value)}
        />
        <div className="dialog-actions">
          <button className="dialog-btn-secondary" onClick={onClose}>Cancel</button>
          <button
            className="dialog-btn-primary"
            disabled={!expr.trim() || !alias.trim()}
            onClick={() => onSubmit(expr.trim(), alias.trim())}
          >
            Add Column
          </button>
        </div>
      </div>
    </div>
  );
}
