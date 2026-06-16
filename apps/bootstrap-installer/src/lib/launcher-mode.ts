import { invoke } from "@tauri-apps/api/core";

export interface LaunchMode {
  kind: "first_install" | "settings" | "launch" | "update" | "silent";
  target_app_id: string | null;
}
export interface LauncherConfig {
  repo: { owner: string | null; name: string | null; ref_: string | null };
  update: { check_on_launch: boolean; auto_pre_download: boolean; check_interval_seconds: number };
  ui: { start_minimized: boolean; show_pending_update_banner: boolean };
}
export interface LaunchableApp {
  descriptor: { id: string; display_name: string; default: boolean; app_settings_url: string | null; min_launcher_version: string };
  installed: any | null;
  pending: { status: "avail"|"downloading"|"ready"|"failed" } | null;
  launcher_too_old: boolean;
}

export async function fetchLaunchMode(): Promise<LaunchMode> { return invoke<LaunchMode>("get_launch_mode"); }
export type Route = "welcome"|"home"|"app-detail"|"settings"|"progress"|"success"|"failure";

export function resolveInitialRoute(mode: LaunchMode): Route {
  switch (mode.kind) {
    case "first_install": return "welcome";
    case "settings":       return "home";
    case "launch":         return "home";
    case "update":         return "progress";
    case "silent":         return "home";
  }
}
