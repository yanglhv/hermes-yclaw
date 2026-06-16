import { atom, computed } from 'nanostores'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'

/*
 * Bootstrap state store — single source of truth for installer screens.
 *
 * Lives in nanostores per the project's TypeScript guidelines (apps/desktop
 * AGENTS.md): "Prefer small nanostores over component state when state is
 * shared, reused, or read by distant UI."
 *
 * One channel from Rust ('bootstrap' event), discriminated by payload.type.
 * We translate those events into typed atom updates here so the rest of
 * the app only deals with React-friendly state.
 */

// ---------------------------------------------------------------------------
// Types — mirror src-tauri/src/events.rs
// ---------------------------------------------------------------------------

export interface StageInfo {
  name: string
  title: string
  category: string
  needs_user_input: boolean
}

export type StageState = 'running' | 'succeeded' | 'skipped' | 'failed'

export interface StageRecord {
  info: StageInfo
  state: StageState | null
  durationMs?: number
  error?: string
}

export interface BootstrapStateModel {
  status: 'idle' | 'running' | 'completed' | 'failed'
  protocolVersion: number | null
  stages: Record<string, StageRecord>
  stageOrder: string[]
  currentStage: string | null
  installRoot: string | null
  error: string | null
  logs: Array<{ stage?: string; line: string; stream?: 'stdout' | 'stderr' }>
}

const INITIAL: BootstrapStateModel = {
  status: 'idle',
  protocolVersion: null,
  stages: {},
  stageOrder: [],
  currentStage: null,
  installRoot: null,
  error: null,
  logs: []
}

// ---------------------------------------------------------------------------
// Per-app launcher types — mirror src-tauri/src/app.rs
// ---------------------------------------------------------------------------

export interface PendingUpdate {
  latest_commit: string
  latest_ref_name: string
  status: 'avail' | 'downloading' | 'ready' | 'failed'
  downloaded_script?: string
  downloaded_at?: string
  last_error?: string
  last_error_at?: string
}

export interface InstalledApp {
  app_id: string
  version: string
  install_root: string
  last_updated: string
  pending_update?: PendingUpdate
}

export interface LauncherConfig {
  repo: { owner: string | null; name: string | null; ref_: string | null }
  update: { check_on_launch: boolean; auto_pre_download: boolean; check_interval_seconds: number }
  ui: { start_minimized: boolean; show_pending_update_banner: boolean }
}

export interface AppDescriptor {
  schema_version: number
  id: string
  display_name: string
  category: string
  default: boolean
  script_path: string
  install_root: string
  binary: { windows: string; macos: string; linux: string }
  uninstall_supported: boolean
  app_settings_url: string | null
  min_launcher_version: string
  icon?: string
}

export interface LaunchableApp {
  descriptor: AppDescriptor
  installed: InstalledApp | null
  pending: PendingUpdate | null
  launcher_too_old: boolean
}

// ---------------------------------------------------------------------------
// Atoms
// ---------------------------------------------------------------------------

export type Route = 'welcome' | 'progress' | 'success' | 'failure' | 'home' | 'app-detail' | 'settings'

/// How the installer was launched, mirrored from src-tauri AppMode.
/// 'install' = first-run onboarding (bare launch). 'update' = driven by the
/// desktop app handing off via `Hermes-Setup.exe --update`.
export type AppMode = 'install' | 'update'

export const $route = atom<Route>('welcome')
export const $mode = atom<AppMode>('install')
export const $bootstrap = atom<BootstrapStateModel>(INITIAL)
export const $logPath = atom<string | null>(null)
export const $hermesHome = atom<string | null>(null)

// ---------------------------------------------------------------------------
// Per-app atoms — launcher state per app_id
// ---------------------------------------------------------------------------

export const $bootstrapByApp = atom<Record<string, BootstrapStateModel>>({})
export const $currentAppId = atom<string | null>(null)
export const $launchMode = atom<'first_install' | 'settings' | 'launch' | 'update' | 'silent' | null>(null)
export const $launcherState = atom<{ schema_version: number; default_app_id: string | null; installed: Record<string, InstalledApp>; pending_updates: Record<string, PendingUpdate>; last_update_check_at: string | null } | null>(null)
export const $launcherConfig = atom<LauncherConfig | null>(null)
export const $networkStatus = atom<{ hasInternet: boolean; latencyMs: number | null }>({ hasInternet: true, latencyMs: null })
export const $catalogApps = atom<AppDescriptor[]>([])
export const $updateCheckStatus = atom<'idle' | 'checking' | 'done' | 'error'>('idle')

export const $apps = atom<Record<string, LaunchableApp>>({})

export const $appsList = computed($apps, (a) => Object.values(a))

export const $bootstrap_computed = computed(
  [$bootstrapByApp, $currentAppId],
  (byApp, currentId) => {
    if (!currentId) return INITIAL
    return byApp[currentId] ?? INITIAL
  }
)

export const $progress = computed($bootstrap, (b) => {
  const total = b.stageOrder.length
  if (total === 0) return { done: 0, total: 0, fraction: 0 }
  let done = 0
  for (const name of b.stageOrder) {
    const s = b.stages[name]?.state
    if (s === 'succeeded' || s === 'skipped' || s === 'failed') done += 1
  }
  return { done, total, fraction: done / total }
})

// ---------------------------------------------------------------------------
// Tauri event subscription
// ---------------------------------------------------------------------------

interface BootstrapManifestEvent {
  type: 'manifest'
  app_id?: string | null
  stages: StageInfo[]
  protocolVersion: number | null
}

interface BootstrapStageEvent {
  type: 'stage'
  app_id?: string | null
  name: string
  state: StageState
  durationMs?: number
  error?: string
}

interface BootstrapLogEvent {
  type: 'log'
  app_id?: string | null
  stage?: string
  line: string
  stream?: 'stdout' | 'stderr'
}

interface BootstrapCompleteEvent {
  type: 'complete'
  app_id?: string | null
  installRoot: string
  marker: unknown
}

interface BootstrapFailedEvent {
  type: 'failed'
  app_id?: string | null
  stage?: string
  error: string
}

type BootstrapEvent =
  | BootstrapManifestEvent
  | BootstrapStageEvent
  | BootstrapLogEvent
  | BootstrapCompleteEvent
  | BootstrapFailedEvent

function getTargetAppId(evt: BootstrapEvent): string | null {
  return (evt as any).app_id ?? $currentAppId.get()
}

function applyEventToBootstrap(target: BootstrapStateModel, evt: BootstrapEvent): BootstrapStateModel {
  switch (evt.type) {
    case 'manifest': {
      const stages: Record<string, StageRecord> = {}
      const order: string[] = []
      for (const s of evt.stages) {
        stages[s.name] = { info: s, state: null }
        order.push(s.name)
      }
      return {
        ...target,
        status: 'running',
        protocolVersion: evt.protocolVersion,
        stages,
        stageOrder: order,
        currentStage: null,
        installRoot: null,
        error: null,
        logs: []
      }
    }
    case 'stage': {
      const existing = target.stages[evt.name]
      if (!existing) return target
      const next: StageRecord = {
        ...existing,
        state: evt.state,
        durationMs: evt.durationMs,
        error: evt.error
      }
      return {
        ...target,
        stages: { ...target.stages, [evt.name]: next },
        currentStage: evt.state === 'running' ? evt.name : target.currentStage
      }
    }
    case 'log': {
      const logs = [...target.logs, { stage: evt.stage, line: evt.line, stream: evt.stream }]
      const trimmed = logs.length > 2000 ? logs.slice(-2000) : logs
      return { ...target, logs: trimmed }
    }
    case 'complete':
      return {
        ...target,
        status: 'completed',
        installRoot: evt.installRoot,
        currentStage: null
      }
    case 'failed':
      return {
        ...target,
        status: 'failed',
        error: evt.error,
        currentStage: null
      }
  }
}

export function routeEvent(evt: BootstrapEvent): void {
  const appId = getTargetAppId(evt)
  if (!appId) return
  $currentAppId.set(appId)
  const byApp = $bootstrapByApp.get()
  const current = byApp[appId] ?? { ...INITIAL }
  const next = applyEventToBootstrap(current, evt)
  $bootstrapByApp.set({ ...byApp, [appId]: next })
  $bootstrap.set(next)
  if (evt.type === 'complete') {
    if ($mode.get() !== 'update') {
      $route.set('success')
    }
  } else if (evt.type === 'failed') {
    $route.set('failure')
  } else if (evt.type === 'manifest') {
    $route.set('progress')
  }
}

let unlisten: UnlistenFn | null = null

export async function initialize(): Promise<void> {
  if (unlisten) return

  // Pull static info on mount for the diagnostics footer.
  try {
    const [logPath, hermesHome, mode] = await Promise.all([
      invoke<string>('get_log_path'),
      invoke<string>('get_hermes_home'),
      invoke<AppMode>('get_mode')
    ])
    $logPath.set(logPath)
    $hermesHome.set(hermesHome)
    $mode.set(mode)
  } catch (err) {
    console.warn('failed to fetch installer paths', err)
  }

  unlisten = await listen<BootstrapEvent>('bootstrap', (event) => {
    routeEvent(event.payload)
  })

  // Update mode is a hand-off, not a user-initiated flow: the desktop already
  // exited and re-launched us as `--update`. Kick the update immediately so
  // the user lands on progress, not a redundant "click to update" screen.
  if ($mode.get() === 'update') {
    void startUpdate()
  }
}

// ---------------------------------------------------------------------------
// Actions
// ---------------------------------------------------------------------------

export async function startInstall(opts?: { branch?: string }): Promise<void> {
  // Reset before kicking off so a retry from the failure screen clears
  // the previous run's state.
  $bootstrap.set(INITIAL)
  $route.set('progress')
  await invoke('start_bootstrap', {
    args: {
      commit: null,
      branch: opts?.branch ?? null,
      include_desktop: true,
      hermes_home: null
    }
  })
}

export async function startUpdate(): Promise<void> {
  // Update is driven by the desktop handing off (Hermes-Setup.exe --update);
  // there's no welcome click. Reset + jump straight to progress, then let the
  // Rust side stream the synthetic update manifest.
  $bootstrap.set(INITIAL)
  $route.set('progress')
  await invoke('start_update')
}

export async function cancelInstall(): Promise<void> {
  await invoke('cancel_bootstrap')
}

export async function launchHermesDesktop(): Promise<void> {
  const installRoot = $bootstrap.get().installRoot
  if (!installRoot) throw new Error('no install root')
  await invoke('launch_hermes_desktop', { installRoot })
}

export async function openLogDir(): Promise<void> {
  await invoke('open_log_dir')
}

// ---------------------------------------------------------------------------
// Launcher actions — per-app state management
// ---------------------------------------------------------------------------

export async function launchApp(id: string): Promise<void> {
  await invoke('launch_app', { id })
}

export async function uninstallApp(id: string, scope: 'user' | 'machine'): Promise<void> {
  await invoke('uninstall_app', { id, scope })
}

export async function repairApp(id: string): Promise<void> {
  $currentAppId.set(id)
  $bootstrap.set(INITIAL)
  $route.set('progress')
  await invoke('repair_app', { id })
}

export async function checkForUpdates(): Promise<void> {
  $updateCheckStatus.set('checking')
  try {
    const updated = await invoke<Record<string, PendingUpdate>>('check_for_updates')
    const state = $launcherState.get()
    if (state) {
      $launcherState.set({
        ...state,
        pending_updates: { ...state.pending_updates, ...updated }
      })
    }
    $updateCheckStatus.set('done')
  } catch {
    $updateCheckStatus.set('error')
  }
}

export async function preDownloadUpdate(id: string): Promise<void> {
  await invoke('pre_download_update', { id })
}

export async function applyPendingUpdate(id: string): Promise<void> {
  $currentAppId.set(id)
  $route.set('progress')
  await invoke('apply_pending_update', { appId: id })
  await loadAppsList()
}

export async function openAppSettings(id: string): Promise<void> {
  await invoke('open_app_settings', { id })
}

export async function setDefaultApp(id: string): Promise<void> {
  await invoke('set_default_app', { id })
  const state = $launcherState.get()
  if (state) {
    $launcherState.set({ ...state, default_app_id: id })
  }
}

export async function loadLauncherConfig(): Promise<void> {
  const config = await invoke<LauncherConfig>('get_launcher_config')
  $launcherConfig.set(config)
}

export async function saveLauncherConfig(yaml: string): Promise<void> {
  await invoke('set_launcher_config', { yaml })
  await loadLauncherConfig()
}

export async function loadLauncherState(): Promise<void> {
  const state = await invoke<{ schema_version: number; default_app_id: string | null; installed: Record<string, InstalledApp>; pending_updates: Record<string, PendingUpdate>; last_update_check_at: string | null }>('get_launcher_state')
  $launcherState.set(state)
}

export async function loadAppsList(): Promise<void> {
  const apps = await invoke<LaunchableApp[]>('list_available_apps')
  const appsMap: Record<string, LaunchableApp> = {}
  for (const app of apps) {
    appsMap[app.descriptor.id] = app
  }
  $apps.set(appsMap)
}
