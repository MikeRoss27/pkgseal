/**
 * App-level error normalization. Keeps UI from branching on raw Tauri / fetch shapes.
 */

export type AppError = {
  code: string
  message: string
  recoverable: boolean
  cause?: unknown
}

const DEFAULT_CODE = "UNKNOWN_ERROR"

export function toAppError(e: unknown): AppError {
  if (isAppError(e)) return e

  if (e instanceof Error) {
    // Tauri invoke errors are often plain strings or Error with message
    return {
      code: inferCode(e.message),
      message: e.message || "An unexpected error occurred",
      recoverable: isRecoverableMessage(e.message),
      cause: e,
    }
  }

  if (typeof e === "string") {
    return {
      code: inferCode(e),
      message: e,
      recoverable: isRecoverableMessage(e),
      cause: e,
    }
  }

  if (e !== null && typeof e === "object" && "message" in e) {
    const msg = String((e as { message: unknown }).message)
    const code =
      "code" in e && typeof (e as { code: unknown }).code === "string"
        ? (e as { code: string }).code
        : inferCode(msg)
    return {
      code,
      message: msg,
      recoverable: isRecoverableMessage(msg),
      cause: e,
    }
  }

  return {
    code: DEFAULT_CODE,
    message: "An unexpected error occurred",
    recoverable: true,
    cause: e,
  }
}

export function isAppError(value: unknown): value is AppError {
  return (
    typeof value === "object" &&
    value !== null &&
    "code" in value &&
    "message" in value &&
    "recoverable" in value &&
    typeof (value as Record<string, unknown>).code === "string" &&
    typeof (value as Record<string, unknown>).message === "string" &&
    typeof (value as Record<string, unknown>).recoverable === "boolean"
  )
}

function inferCode(message: string): string {
  const m = message.toLowerCase()
  if (m.includes("not found") || m.includes("404")) return "NOT_FOUND"
  if (m.includes("network") || m.includes("failed to fetch") || m.includes("offline")) return "NETWORK_ERROR"
  if (m.includes("timeout")) return "TIMEOUT"
  if (m.includes("permission") || m.includes("forbidden") || m.includes("403")) return "FORBIDDEN"
  if (m.includes("unauthorized") || m.includes("401")) return "UNAUTHORIZED"
  if (m.includes("validation") || m.includes("invalid") || m.includes("parse")) return "VALIDATION_ERROR"
  return DEFAULT_CODE
}

function isRecoverableMessage(message: string): boolean {
  const m = message.toLowerCase()
  if (m.includes("not found")) return false
  if (m.includes("unauthorized") || m.includes("forbidden")) return false
  return true
}

export function getErrorMessage(e: unknown): string {
  return toAppError(e).message
}
