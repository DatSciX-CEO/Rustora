interface TableListProps {
  tables: string[];
  activeTable: string | null;
  onSelectTable: (name: string) => void;
  onRemoveTable: (name: string) => void;
  visible: boolean;
}

export function TableList({
  tables,
  activeTable,
  onSelectTable,
  onRemoveTable,
  visible,
}: TableListProps) {
  if (!visible || tables.length === 0) return null;

  return (
    <div className="table-list">
      <div className="table-list-header">
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
          <ellipse cx="12" cy="5" rx="9" ry="3" />
          <path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3" />
          <path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5" />
        </svg>
        TABLES
      </div>
      <div className="table-list-items">
        {tables.map((name) => (
          <div
            key={name}
            className={`table-list-item ${activeTable === name ? "active" : ""}`}
            onClick={() => onSelectTable(name)}
          >
            <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <rect x="3" y="3" width="18" height="18" rx="2" />
              <line x1="3" y1="9" x2="21" y2="9" />
              <line x1="9" y1="21" x2="9" y2="9" />
            </svg>
            <span className="table-name">{name}</span>
            <button
              className="table-remove-btn"
              onClick={(e) => {
                e.stopPropagation();
                onRemoveTable(name);
              }}
              title="Drop table"
            >
              &times;
            </button>
          </div>
        ))}
      </div>
    </div>
  );
}
