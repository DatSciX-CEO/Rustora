/**
 * StatsPanel — full-screen overlay displaying DuckDB SUMMARIZE statistics.
 *
 * Shows a scrollable table of statistics (count, null_count, min, max, mean, std)
 * for all columns in the active dataset. Rendered from Arrow IPC data returned
 * by the `get_summary_stats` Tauri command.
 */
import type { ParsedTable } from "../lib/arrow";

interface StatsPanelProps {
  /** Parsed statistics table, or null while the data is loading. */
  data: ParsedTable | null;
  /** Called when the panel should be closed. */
  onClose: () => void;
}

export function StatsPanel({ data, onClose }: StatsPanelProps) {
  return (
    <div className="stats-panel-overlay" onClick={onClose}>
      <div className="stats-panel" onClick={(e) => e.stopPropagation()}>
        <div className="stats-panel-header">
          <h3>Summary Statistics</h3>
          <button className="stats-close-btn" onClick={onClose}>&times;</button>
        </div>
        {!data ? (
          <div className="stats-loading">
            <div className="grid-loading-spinner" />
            <span>Calculating…</span>
          </div>
        ) : (
          <div className="stats-table-wrap">
            <table className="stats-table">
              <thead>
                <tr>
                  {data.columns.map((c) => (
                    <th key={c}>{c}</th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {data.rows.map((row, i) => (
                  <tr key={i}>
                    {data.columns.map((c) => (
                      <td key={c}>{row[c] != null ? String(row[c]) : "—"}</td>
                    ))}
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </div>
  );
}
