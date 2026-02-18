import { useRef, useCallback, useEffect } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import type { ParsedTable } from "../lib/arrow";
import type { ColumnInfo } from "../hooks/useDataset";

interface DataGridProps {
  page: ParsedTable | null;
  columns: ColumnInfo[];
  totalRows: number;
  offset: number;
  pageSize: number;
  sortColumn: string | null;
  sortDesc: boolean;
  onSort: (column: string) => void;
  onPageChange: (offset: number) => void;
  loading: boolean;
}

const ROW_HEIGHT = 28;
const MIN_COL_WIDTH = 100;
const MAX_COL_WIDTH = 400;

function estimateColumnWidth(
  col: string,
  rows: Record<string, unknown>[],
  dtype: string
): number {
  const headerLen = col.length;
  let maxLen = headerLen;

  const sampleSize = Math.min(rows.length, 50);
  for (let i = 0; i < sampleSize; i++) {
    const val = rows[i][col];
    const len = val != null ? String(val).length : 4;
    if (len > maxLen) maxLen = len;
  }

  const isNumeric =
    dtype.includes("Int") ||
    dtype.includes("Float") ||
    dtype.includes("UInt");
  const charWidth = isNumeric ? 8.4 : 7.8;
  const width = Math.max(MIN_COL_WIDTH, Math.min(maxLen * charWidth + 24, MAX_COL_WIDTH));
  return Math.ceil(width);
}

function formatCellValue(val: unknown): string {
  if (val === null || val === undefined) return "";
  if (typeof val === "number") {
    if (Number.isInteger(val)) return val.toLocaleString();
    return val.toLocaleString(undefined, {
      minimumFractionDigits: 1,
      maximumFractionDigits: 6,
    });
  }
  if (typeof val === "bigint") return val.toLocaleString();
  return String(val);
}

export function DataGrid({
  page,
  columns,
  totalRows,
  offset,
  pageSize,
  sortColumn,
  sortDesc,
  onSort,
  onPageChange,
  loading,
}: DataGridProps) {
  const parentRef = useRef<HTMLDivElement>(null);
  const rows = page?.rows ?? [];

  const rowVirtualizer = useVirtualizer({
    count: rows.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => ROW_HEIGHT,
    overscan: 20,
  });

  const colWidths = columns.map((c) =>
    rows.length > 0
      ? estimateColumnWidth(c.name, rows, c.dtype)
      : MIN_COL_WIDTH
  );
  const totalWidth = colWidths.reduce((a, b) => a + b, 0) + 52;

  const handleScroll = useCallback(() => {
    const el = parentRef.current;
    if (!el || loading) return;

    const { scrollTop, scrollHeight, clientHeight } = el;
    const atBottom = scrollHeight - scrollTop - clientHeight < ROW_HEIGHT * 5;
    const atTop = scrollTop < ROW_HEIGHT * 2;

    if (atBottom && offset + pageSize < totalRows) {
      onPageChange(Math.min(offset + pageSize, totalRows - 1));
    } else if (atTop && offset > 0) {
      onPageChange(Math.max(0, offset - pageSize));
    }
  }, [offset, pageSize, totalRows, loading, onPageChange]);

  useEffect(() => {
    const el = parentRef.current;
    if (!el) return;
    el.addEventListener("scroll", handleScroll, { passive: true });
    return () => el.removeEventListener("scroll", handleScroll);
  }, [handleScroll]);

  if (columns.length === 0) {
    return (
      <div className="grid-empty">
        <div className="grid-empty-content">
          <div className="welcome-icon">🦀</div>
          <h2 className="welcome-title">Welcome to Rustora</h2>
          <p className="welcome-subtitle">
            High-performance data analysis, 100% on your machine
          </p>
          <div className="welcome-actions">
            <div className="welcome-card">
              <div className="welcome-card-icon">
                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <path d="M12 5v14M5 12h14" />
                </svg>
              </div>
              <div className="welcome-card-text">
                <strong>New Project</strong>
                <span>Create a .duckdb project for persistent storage</span>
              </div>
            </div>
            <div className="welcome-card">
              <div className="welcome-card-icon">
                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <ellipse cx="12" cy="5" rx="9" ry="3" />
                  <path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3" />
                  <path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5" />
                </svg>
              </div>
              <div className="welcome-card-text">
                <strong>Open Project</strong>
                <span>Resume work on an existing .duckdb project</span>
              </div>
            </div>
            <div className="welcome-card">
              <div className="welcome-card-icon">
                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4" />
                  <polyline points="7 10 12 15 17 10" />
                  <line x1="12" y1="15" x2="12" y2="3" />
                </svg>
              </div>
              <div className="welcome-card-text">
                <strong>Open File</strong>
                <span>Quick-analyze a CSV, Parquet, or Arrow file</span>
              </div>
            </div>
          </div>
          <span className="grid-empty-hint">
            Use the toolbar above to get started
          </span>
        </div>
      </div>
    );
  }

  return (
    <div className="grid-container">
      {/* Column Headers */}
      <div className="grid-header" style={{ width: totalWidth }}>
        <div className="grid-row-num-header">#</div>
        {columns.map((col, i) => (
          <div
            key={col.name}
            className={`grid-header-cell ${sortColumn === col.name ? "sorted" : ""}`}
            style={{ width: colWidths[i] }}
            onClick={() => onSort(col.name)}
            title={`${col.name} (${col.dtype})`}
          >
            <span className="header-name">{col.name}</span>
            <span className="header-dtype">{col.dtype}</span>
            {sortColumn === col.name && (
              <span className="sort-indicator">
                {sortDesc ? "\u25BC" : "\u25B2"}
              </span>
            )}
          </div>
        ))}
      </div>

      {/* Virtualized Rows */}
      <div ref={parentRef} className="grid-body">
        <div
          style={{
            height: rowVirtualizer.getTotalSize(),
            width: totalWidth,
            position: "relative",
          }}
        >
          {rowVirtualizer.getVirtualItems().map((virtualRow) => {
            const row = rows[virtualRow.index];
            const rowIdx = offset + virtualRow.index;
            return (
              <div
                key={virtualRow.index}
                className={`grid-row ${virtualRow.index % 2 === 0 ? "even" : "odd"}`}
                style={{
                  position: "absolute",
                  top: virtualRow.start,
                  height: ROW_HEIGHT,
                  width: totalWidth,
                }}
              >
                <div className="grid-row-num">{rowIdx + 1}</div>
                {columns.map((col, i) => {
                  const val = row?.[col.name];
                  const isNull = val === null || val === undefined;
                  const isNumeric =
                    col.dtype.includes("Int") ||
                    col.dtype.includes("Float") ||
                    col.dtype.includes("UInt");
                  return (
                    <div
                      key={col.name}
                      className={`grid-cell ${isNumeric ? "numeric" : ""} ${isNull ? "null-val" : ""}`}
                      style={{ width: colWidths[i] }}
                      title={isNull ? "NULL" : String(val)}
                    >
                      {isNull ? "NULL" : formatCellValue(val)}
                    </div>
                  );
                })}
              </div>
            );
          })}
        </div>
        {loading && (
          <div className="grid-loading-overlay">
            <div className="grid-loading-spinner" />
          </div>
        )}
      </div>
    </div>
  );
}
