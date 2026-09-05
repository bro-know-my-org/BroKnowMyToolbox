import { invoke } from "@tauri-apps/api/core";

export type ThemeMode = "system" | "light" | "dark";
export type DensityMode = "comfortable" | "compact";

export interface AppConfig {
  schemaVersion: number;
  locale: string;
  theme: ThemeMode;
  accentColor: string;
  fontScalePercent: number;
  density: DensityMode;
  reducedMotion: boolean;
  favoriteToolIds: string[];
  recentToolIds: string[];
}

export function loadAppConfig(): Promise<AppConfig> {
  return invoke("load_app_config_command");
}

export function saveAppConfig(config: AppConfig): Promise<void> {
  return invoke("save_app_config_command", { config });
}
