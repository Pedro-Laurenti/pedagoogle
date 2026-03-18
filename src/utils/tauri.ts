import { invoke } from "@tauri-apps/api/core";

export function invokeCmd<T = unknown>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  return invoke<T>(cmd, args);
}
