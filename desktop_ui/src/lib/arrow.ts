import { tableFromIPC } from "apache-arrow";

export interface ParsedTable {
  columns: string[];
  dtypes: string[];
  rows: Record<string, unknown>[];
  rowCount: number;
}

/**
 * Parse Arrow IPC bytes (received as number[] from Tauri) into rows and columns.
 * This is the ONLY deserialization point -- data arrives as raw binary, not JSON.
 *
 * Uses column-wise iteration to avoid O(n*m) getChild lookups per cell.
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
