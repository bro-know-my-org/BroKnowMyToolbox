import { invoke } from "@tauri-apps/api/core";

export interface RuntimeDiagnostics {
  dataRoot: string;
  dataRootSource: string;
}

export function loadRuntimeDiagnostics(): Promise<RuntimeDiagnostics> {
  return invoke("runtime_diagnostics_command");
}
