/**
 * FilterDialog — modal for entering a SQL WHERE clause to filter the active dataset.
 *
 * Displays a textarea pre-populated with column names for reference. Submits
 * on "Apply Filter" click or Ctrl+Enter keyboard shortcut.
 */
import { useState } from "react";
import type { ColumnInfo } from "../hooks/useDataset";

interface FilterDialogProps {
  /** Columns of the active dataset, used to display available column names. */
  columns: ColumnInfo[];
  /** Called with the WHERE clause string when the user applies the filter. */
  onSubmit: (clause: string) => void;
  /** Called when the dialog should be closed without submitting. */
  onClose: () => void;
}

export function FilterDialog({ columns, onSubmit, onClose }: FilterDialogProps) {
  const [clause, setClause] = useState("");
  return (
    <div className="dialog-overlay" onClick={onClose}>
      <div className="dialog" onClick={(e) => e.stopPropagation()}>
        <h3>Filter Rows</h3>
        <p className="dialog-help">
          Enter a SQL WHERE clause. Available columns:{" "}
          <span className="dialog-cols">{columns.map((c) => c.name).join(", ")}</span>
        </p>
        <textarea
          className="dialog-input"
          placeholder='e.g. age > 30 AND city = &#39;Boston&#39;'
          value={clause}
          onChange={(e) => setClause(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter" && (e.ctrlKey || e.metaKey) && clause.trim()) {
              onSubmit(clause.trim());
            }
          }}
          autoFocus
        />
        <div className="dialog-actions">
          <button className="dialog-btn-secondary" onClick={onClose}>Cancel</button>
          <button
            className="dialog-btn-primary"
            disabled={!clause.trim()}
            onClick={() => onSubmit(clause.trim())}
          >
            Apply Filter
          </button>
        </div>
      </div>
    </div>
  );
}
