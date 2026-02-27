import { tableFromIPC } from "apache-arrow";

/**
 * Parsed representation of an Arrow IPC stream after deserialization.
 * All tabular data from Tauri commands arrives in this form.
 */
export interface ParsedTable {
  /** Column names in schema order. */
  columns: string[];
  /** Arrow type strings for each column (e.g. "Int64", "Utf8", "Float32"). */
  dtypes: string[];
  /** Row data as plain objects keyed by column name. */
  rows: Record<string, unknown>[];
  /** Total number of rows in this payload (may be a page, not the full dataset). */
  rowCount: number;
}

/**
 * Parse Arrow IPC bytes (received as number[] from Tauri) into rows and columns.
 * This is the ONLY deserialization point -- data arrives as raw binary, not JSON.
 *
 * Uses column-wise iteration to avoid O(n*m) getChild lookups per cell.
 * Each column vector is extracted once, then all rows for that column are populated
 * in a single pass — resulting in O(n + m) instead of O(n*m) access patterns.
 */
export function parseIpcBytes(bytes: number[]): ParsedTable {
  const buffer = new Uint8Array(bytes);
  const table = tableFromIPC(buffer);

  const columns = table.schema.fields.map((f) => f.name);
  const dtypes = table.schema.fields.map((f) => f.type.toString());
  const numRows = table.numRows;

  const rows: Record<string, unknown>[] = new Array(numRows);
  for (let i = 0; i < numRows; i++) {
    rows[i] = {};
  }

  for (const col of columns) {
    const vector = table.getChild(col);
    if (!vector) continue;
    for (let i = 0; i < numRows; i++) {
      rows[i][col] = vector.get(i);
    }
  }

  return { columns, dtypes, rows, rowCount: numRows };
}
