import { useState, useEffect } from "react";

interface StatusBarProps {
  totalRows: number;
  totalColumns: number;
  offset: number;
  pageSize: number;
  datasetName: string | null;
  sizeBytes: number | null;
  persistent: boolean;
  projectPath: string | null;
  error: string | null;
  loading: boolean;
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
}

export function StatusBar({
  totalRows,
  totalColumns,
  offset,
  pageSize,
  datasetName,
  sizeBytes,
  persistent,
  projectPath,
  error,
  loading,
}: StatusBarProps) {
  const visibleEnd = Math.min(offset + pageSize, totalRows);
  const [dismissedError, setDismissedError] = useState<string | null>(null);

  useEffect(() => {
    if (error && error !== dismissedError) {
      setDismissedError(null);
    }
  }, [error, dismissedError]);

  const showError = error && error !== dismissedError;

  return (
    <div className={`status-bar ${showError ? "status-bar-error" : ""}`}>
      <div className="status-left">
        {showError ? (
          <span className="status-error" title={error}>
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <circle cx="12" cy="12" r="10" />
              <line x1="12" y1="8" x2="12" y2="12" />
              <line x1="12" y1="16" x2="12.01" y2="16" />
            </svg>
            {error}
            <button
              className="status-error-dismiss"
              onClick={() => setDismissedError(error)}
              title="Dismiss"
            >
              &times;
            </button>
          </span>
        ) : loading ? (
          <span className="status-item status-loading">
            <span className="status-spinner" />
            Processing...
          </span>
        ) : datasetName ? (
          <>
            <span className="status-item">
              Rows: <strong>{totalRows.toLocaleString()}</strong>
            </span>
            <span className="status-sep">|</span>
            <span className="status-item">
              Columns: <strong>{totalColumns}</strong>
            </span>
            {sizeBytes != null && sizeBytes > 0 && (
              <>
                <span className="status-sep">|</span>
                <span className="status-item">
                  Size: <strong>{formatBytes(sizeBytes)}</strong>
                </span>
              </>
            )}
            {totalRows > 0 && (
              <>
                <span className="status-sep">|</span>
                <span className="status-item">
                  Showing {(offset + 1).toLocaleString()}&ndash;
                  {visibleEnd.toLocaleString()}
                </span>
              </>
            )}
          </>
        ) : (
          <span className="status-item status-muted">No data loaded</span>
        )}
      </div>
      <div className="status-right">
        {persistent && (
          <>
            <span className="status-item status-persistent">DuckDB</span>
            <span className="status-sep">|</span>
          </>
        )}
        <span className="status-item status-muted">100% Local</span>
        <span className="status-sep">|</span>
        <span className="status-item status-muted">Arrow IPC</span>
        {projectPath && (
          <>
            <span className="status-sep">|</span>
            <span className="status-item status-muted" title={projectPath}>
              {projectPath.split(/[\\/]/).pop()}
            </span>
          </>
        )}
      </div>
    </div>
  );
}
