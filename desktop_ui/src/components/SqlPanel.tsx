import { useState, useRef, useCallback } from "react";

interface SqlPanelProps {
  onExecute: (sql: string) => void;
  loading: boolean;
  visible: boolean;
}

export function SqlPanel({ onExecute, loading, visible }: SqlPanelProps) {
  const [sql, setSql] = useState("");
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === "Enter") {
        e.preventDefault();
        if (sql.trim() && !loading) {
          onExecute(sql.trim());
        }
      }
    },
    [sql, loading, onExecute]
  );

  if (!visible) return null;

  return (
    <div className="sql-panel">
      <div className="sql-header">
        <span className="sql-label">SQL QUERY</span>
        <span className="sql-hint">Ctrl+Enter to execute</span>
      </div>
      <div className="sql-editor-wrap">
        <textarea
          ref={textareaRef}
          className="sql-editor"
          value={sql}
          onChange={(e) => setSql(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder='SELECT * FROM "dataset_name" WHERE column > value'
          spellCheck={false}
        />
        <button
          className="sql-run-btn"
          onClick={() => sql.trim() && onExecute(sql.trim())}
          disabled={loading || !sql.trim()}
        >
          Run
        </button>
      </div>
    </div>
  );
}
