/**
 * Structured error returned by Tauri commands when a `RustoraError` occurs.
 * Mirrors the `CommandError` struct in `desktop_ui/src-tauri/src/lib.rs`.
 */
export interface CommandError {
  /** Machine-readable error code (e.g. "file_not_found", "unsupported_format"). */
  code: string;
  /** Error category grouping related codes (e.g. "file", "sql", "data"). */
  category: string;
  /** Human-readable error message suitable for display in the UI. */
  message: string;
}

/**
 * Extract a human-readable message from a Tauri command error.
 * Handles both legacy string errors and the new structured {@link CommandError} objects.
 */
export function errorMessage(e: unknown): string {
  if (typeof e === "string") return e;
  if (e && typeof e === "object" && "message" in e) {
    return String((e as CommandError).message);
  }
  return String(e);
}
