import type { CommandError } from "./commands";

export function isCommandError(value: unknown): value is CommandError {
  if (typeof value !== "object" || value === null) return false;
  const obj = value as Record<string, unknown>;
  return (
    typeof obj.code === "string" &&
    typeof obj.message === "string"
  );
}

export function normalizeInvokeError(value: unknown): CommandError {
  if (isCommandError(value)) return value;
  return {
    code: "Internal",
    message: value instanceof Error ? value.message : "未知错误",
    path: null,
    details: null,
  };
}
