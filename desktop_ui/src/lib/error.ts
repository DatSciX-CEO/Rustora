export interface CommandError {
  code: string;
  category: string;
  message: string;
}

/**
 * Extract a human-readable message from a Tauri command error.
 * Handles both legacy string errors and the new structured CommandError objects.
 */
export function errorMessage(e: unknown): string {
  if (typeof e === "string") return e;
  if (e && typeof e === "object" && "message" in e) {
    return String((e as CommandError).message);
  }
  return String(e);
}
