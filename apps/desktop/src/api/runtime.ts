import { invoke } from "@tauri-apps/api/core";

export interface RuntimeDiagnostics {
  dataRoot: string;
  dataRootSource: "explicit" | "environment" | "portable" | "system";
}

export function loadRuntimeDiagnostics(): Promise<RuntimeDiagnostics> {
  return invoke("runtime_diagnostics_command");
}
