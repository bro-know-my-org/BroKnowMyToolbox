import { invoke } from "@tauri-apps/api/core";

export interface FileGenerationRequest {
  templateJson: string;
  destination: string;
  variables: Record<string, string>;
  overwrite: boolean;
}

export interface PlannedFile {
  path: string;
  action: "create" | "overwrite" | "conflict";
  targetRevision?: string | null;
}

export interface FileGenerationPlan {
  status: "planned";
  files: PlannedFile[];
}

export interface ExecutedFile {
  path: string;
  outcome: "created" | "overwritten" | "skipped_conflict" | "failed";
  error?: string;
}

export interface FileGenerationReport {
  status: "complete" | "conflict" | "partial_failure" | "failed";
  files: ExecutedFile[];
}

export interface CommandError {
  code: string;
  message: string;
}

export interface FileTemplateEntry {
  id: string;
  title: string;
  source: "built_in" | "user";
  templateJson: string;
}

export function planFileGeneration(
  request: FileGenerationRequest,
): Promise<FileGenerationPlan> {
  return invoke("plan_file_generation", { request });
}

export function executeFileGeneration(
  request: FileGenerationRequest,
  reviewedFiles: PlannedFile[],
): Promise<FileGenerationReport> {
  return invoke("execute_file_generation_command", { request, reviewedFiles });
}

export function setFileGenerationConsent(allowed: boolean): Promise<void> {
  return invoke("set_file_generation_consent_command", { allowed });
}

export function listFileTemplates(): Promise<FileTemplateEntry[]> {
  return invoke("list_file_templates_command");
}

export function saveUserTemplate(
  templateJson: string,
): Promise<FileTemplateEntry> {
  return invoke("save_user_template_command", { templateJson });
}
