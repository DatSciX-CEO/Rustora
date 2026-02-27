/**
 * GroupByDialog — modal for selecting group-by columns and aggregation expressions.
 *
 * Presents a checkbox grid of available columns for grouping, and a textarea
 * for entering SQL aggregate expressions (e.g. COUNT(*), AVG(salary)).
 */
import { useState } from "react";
import type { ColumnInfo } from "../hooks/useDataset";

interface GroupByDialogProps {
  /** Columns of the active dataset to display as grouping options. */
  columns: ColumnInfo[];
  /** Called with the selected group columns and parsed aggregation expressions. */
  onSubmit: (groupCols: string[], aggs: string[]) => void;
  /** Called when the dialog should be closed without submitting. */
  onClose: () => void;
}

export function GroupByDialog({ columns, onSubmit, onClose }: GroupByDialogProps) {
  const [selectedCols, setSelectedCols] = useState<Set<string>>(new Set());
  const [aggExpr, setAggExpr] = useState("COUNT(*)");

  const toggleCol = (name: string) => {
    setSelectedCols((prev) => {
      const next = new Set(prev);
      if (next.has(name)) next.delete(name);
      else next.add(name);
      return next;
    });
  };

  return (
    <div className="dialog-overlay" onClick={onClose}>
      <div className="dialog" onClick={(e) => e.stopPropagation()}>
        <h3>Group By</h3>
        <p className="dialog-help">Select columns to group by:</p>
        <div className="dialog-col-grid">
          {columns.map((c) => (
            <label key={c.name} className="dialog-col-check">
              <input
                type="checkbox"
                checked={selectedCols.has(c.name)}
                onChange={() => toggleCol(c.name)}
              />
              <span>{c.name}</span>
              <span className="dialog-col-dtype">{c.dtype}</span>
            </label>
          ))}
        </div>
        <p className="dialog-help" style={{ marginTop: 12 }}>
          Aggregation expressions (comma-separated):
        </p>
        <textarea
          className="dialog-input"
          placeholder="COUNT(*), AVG(salary), SUM(amount)"
          value={aggExpr}
          onChange={(e) => setAggExpr(e.target.value)}
        />
        <div className="dialog-actions">
          <button className="dialog-btn-secondary" onClick={onClose}>Cancel</button>
          <button
            className="dialog-btn-primary"
            disabled={selectedCols.size === 0 || !aggExpr.trim()}
            onClick={() =>
              onSubmit(
                Array.from(selectedCols),
                aggExpr.split(",").map((s) => s.trim()).filter(Boolean)
              )
            }
          >
            Apply Group By
          </button>
        </div>
      </div>
    </div>
  );
}
