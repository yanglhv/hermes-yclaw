# Implementation Plan: add-launcher-features-to-bootstrap-installer

## 来源

- 提案：`openspec/changes/add-launcher-features-to-bootstrap-installer/proposal.md`
- 设计：`openspec/changes/add-launcher-features-to-bootstrap-installer/design.md`
- 规格：`openspec/changes/add-launcher-features-to-bootstrap-installer/specs/`
- 任务：`openspec/changes/add-launcher-features-to-bootstrap-installer/tasks.md`

## 执行依赖图

```
M1 (parameterization) ──┬──> M2 (backend skeleton) ──┬──> M3 (CLI modes + silent)
                        │                              │
                        └──> M4 (frontend) ←───────────┘
                                       │
                                       └──> M5 (pending updates)
                                       └──> M6 (uninstall / repair)
                                                          │
                                                          v
                                                       M7 (verification)
```

M1 是地基，所有 M 都依赖它；M2 后端独立；M3 依赖 M2；M4 依赖 M2；
M5 依赖 M3 + M4；M6 依赖 M4；M7 依赖所有。

## 实现步骤

每步 2-5 分钟工作量；按依赖排序；总步数 ~80。

---

### M1 — Parameterization (foundation; no user-visible change)

**Goal**: Make `bootstrap.rs`, `update.rs`, `install_script.rs` accept
parameterized inputs without changing observable behavior. After M1, all
existing tests must still pass.

#### Task 1.1: Define `RepoRef` struct in `install_script.rs`

- 目标：Add a `RepoRef { owner, name, ref_name }` struct and a
  `RepoRef::hardcoded_default()` that returns the current
  `NousResearch/hermes-agent/main` values.
- 改动文件：
  - `apps/bootstrap-installer/src-tauri/src/install_script.rs`
- 验证方式：
  - `cd apps/bootstrap-installer/src-tauri && cargo build` 编译通过
  - `cargo test install_script` 全部通过

#### Task 1.2: Refactor `install_script::resolve()` to accept `RepoRef`

- 目标：`resolve()` 内部 URL 构造改为
  `https://raw.githubusercontent.com/{owner}/{name}/{ref_name}/{path}`。
  接收 `RepoRef` 而非硬编码 owner/name。
- 改动文件：
  - `apps/bootstrap-installer/src-tauri/src/install_script.rs`
- 验证方式：
  - `cargo test install_script::tests::is_valid_commit_accepts_short_and_full_shas` 通过
  - 新增 unit test 验证给定
    `RepoRef { owner: "alice", name: "x", ref_name: "main" }` 时 URL 是
    `https://raw.githubusercontent.com/alice/x/main/...`

#### Task 1.3: Add `AppDescriptor` skeleton in new `app.rs`

- 目标：新建 `app.rs`；定义 `AppDescriptor` 与 `AppJson`；
  `AppDescriptor::literal_hermes()` 工厂返回硬编码 Hermes 值。
- 改动文件：
  - `apps/bootstrap-installer/src-tauri/src/app.rs` (new)
  - `apps/bootstrap-installer/src-tauri/src/lib.rs` (add `mod app;`)
- 验证方式：
  - `cargo build` 通过
  - `cargo test app::tests::literal_hermes_has_expected_fields` 通过

#### Task 1.4: Modify `bootstrap::run_bootstrap` signature

- 目标：`run_bootstrap` 接受 `app: &AppDescriptor`；URL 拼接从
  `AppDescriptor.repo_*` + `script_path` 来；install_root 从
  `HERMES_HOME/<install_root_template>` 来。
- 改动文件：
  - `apps/bootstrap-installer/src-tauri/src/bootstrap.rs`
- 验证方式：
  - `cargo build` 通过
  - `cargo test bootstrap` 全部通过（用 `literal_hermes()` 默认值）

#### Task 1.5: Modify `update::run_update` signature

- 目标：同 1.4，但作用于 `update.rs` 的 `run_update`；保留 Windows
  lock probe、retry-once、`/usr/bin/ditto` 等所有现有逻辑。
- 改动文件：
  - `apps/bootstrap-installer/src-tauri/src/update.rs`
- 验证方式：
  - `cargo test update` 全部通过

#### Task 1.6: Add `appId: Option<String>` to every `BootstrapEvent`

- 目标：`events.rs` 中每个 `BootstrapEvent` variant 增加
  `appId: Option<String>` 字段，`#[serde(skip_serializing_if = "Option::is_none")]`。
  emit sites 全部增加 `app_id: None` 参数。
- 改动文件：
  - `apps/bootstrap-installer/src-tauri/src/events.rs`
  - `apps/bootstrap-installer/src-tauri/src/bootstrap.rs`
  - `apps/bootstrap-installer/src-tauri/src/update.rs`
- 验证方式：
  - `cargo build` 通过
  - `cargo test` 全套通过（V20 起步验证）

#### Task 1.7: Confirm V20 — all existing tests pass

- 目标：跑全套 `cargo test`，906+1073+357+273 LOC 测试无回归。
- 改动文件：无
- 验证方式：
  - `cargo test` 全绿
  - **里程碑**：M1 完成

---

### M2 — Multi-app backend skeleton

**Goal**: 实现 `app.rs` 全量、`launcher.rs` catalog/state 模块、paths
扩展、Tauri 命令；无 frontend 改动。

#### Task 2.1: Complete `app.rs` data model

- 目标：实现 `AppDescriptor`, `AppJson`, `LaunchableApp`, `LauncherState`,
  `InstalledApp`, `PendingUpdate`, `LauncherConfig`, `RepoRef`；
  `parse_app_json` 接受合法 schema，拒绝非法。
- 改动文件：
  - `apps/bootstrap-installer/src-tauri/src/app.rs`
- 验证方式：
  - `cargo test app::tests::parse_app_json_*` 通过（接受/拒绝矩阵）
  - `cargo test app::tests::launcher_state_round_trip` 通过

#### Task 2.2: Add `launcher_state_path` / `launcher_config_path`

- 目标：`paths.rs` 新增两个 helper；Win/macOS/Linux 各自解析到
  `$HERMES_HOME/launcher-state.json` / `launcher-config.yaml`。
- 改动文件：
  - `apps/bootstrap-installer/src-tauri/src/paths.rs`
- 验证方式：
  - `cargo test paths::tests::launcher_state_path_*` 各 OS 通过

#### Task 2.3: Implement `launcher::state` module

- 目标：`load_or_default()`、`save()`（tmp+rename atomic）、
  `validate_pending_cache()`（检查 downloaded_script 文件存在性）、
  损坏恢复（备份 `.bak.<ts>` + 重建）。
- 改动文件：
  - `apps/bootstrap-installer/src-tauri/src/launcher.rs` (new)
  - `apps/bootstrap-installer/src-tauri/src/lib.rs` (add `mod launcher;`)
- 验证方式：
  - `cargo test launcher::state::tests::*` 通过：
    - atomic write 不破坏现有文件
    - 损坏 JSON → 备份 + 重建
    - pending_updates[].status="ready" 但文件丢失 → 降级到 "failed"

#### Task 2.4: Implement `launcher::catalog::list_available_apps`

- 目标：`list_available_apps(repo)` 调 GitHub Contents API + per-app
  app.json 抓取 + 与 state 合并。
- 改动文件：
  - `apps/bootstrap-installer/src-tauri/src/launcher.rs`
- 验证方式：
  - `cargo test launcher::catalog::tests::*` 通过：
    - 网络失败 → `Ok(empty)`
    - 一个 app.json 损坏 → 跳过该项不影响其他

#### Task 2.5: Implement `launcher::config::RepoRef::resolve()`

- 目标：三层优先级（build < env < yaml）；返回最终 `RepoRef`。
- 改动文件：
  - `apps/bootstrap-installer/src-tauri/src/launcher.rs`
- 验证方式：
  - `cargo test launcher::config::tests::*` 通过：
    - 全部默认 → build-time
    - env 覆盖 build-time
    - yaml 覆盖 env（覆盖 V12/V13/V14）

#### Task 2.6: Implement `launcher::network::probe_network`

- 目标：单 GET `api.github.com/repos/{owner}/{name}/contents/apps?ref={ref}`，
  timeout 2s，返回 bool。
- 改动文件：
  - `apps/bootstrap-installer/src-tauri/src/launcher.rs`
- 验证方式：
  - `cargo test launcher::network::tests::*` 通过：
    - mock 200 → true
    - mock timeout → false
    - mock 5xx → false

#### Task 2.7: Wire Tauri commands for catalog + state + config

- 目标：注册 `list_available_apps`, `get_app`, `get_launcher_state`,
  `get_launcher_config`, `set_launcher_config`, `set_default_app`,
  `check_for_updates` 命令；写入 `lib.rs::invoke_handler!`。
- 改动文件：
  - `apps/bootstrap-installer/src-tauri/src/launcher.rs`
  - `apps/bootstrap-installer/src-tauri/src/lib.rs`
- 验证方式：
  - `cargo build` 通过
  - 手工 `cargo tauri dev` + DevTools console 调
    `invoke('list_available_apps')` 返回正确 shape

#### Task 2.8: Backend unit tests for all modules (covers V12/V13/V14)

- 目标：M2 全部 unit tests；尤其验证 V12（build-time 常量）、
  V13（env 覆盖）、V14（yaml 覆盖）。
- 改动文件：
  - `apps/bootstrap-installer/src-tauri/src/launcher.rs` (tests 模块)
  - `apps/bootstrap-installer/src-tauri/src/app.rs`
- 验证方式：
  - `cargo test launcher::` 全部通过
  - **里程碑**：M2 完成（后端可独立手工验证）

---

### M3 — Silent default launch + CLI flags

**Goal**: 实现静默默认流；lib.rs setup 钩子按 CLI flag 分发；替换现有
macOS-only 快路径为跨平台通用；新增 `launch_app` 与 `get_launch_mode` 命令。

#### Task 3.1: Implement `launcher::run_silent_default`

- 目标：probe_network → catalog → pre_download_update →
  launch_app_silent → exit。包含重入保护 `AtomicBool`。
- 改动文件：
  - `apps/bootstrap-installer/src-tauri/src/launcher.rs`
- 验证方式：
  - `cargo test launcher::silent::tests::*` 通过：
    - 重入调用 no-op
    - 无 network → 跳过 update 检查
    - 无 installed apps → 触发 first_install 标记

#### Task 3.2: Implement `launcher::launch_app_silent` + failure recording

- 目标：spawn binary → sleep 150ms → `app.exit(0)`；失败时
  记录 `state.last_launch_error` 但仍 exit。
- 改动文件：
  - `apps/bootstrap-installer/src-tauri/src/launcher.rs`
- 验证方式：
  - `cargo test launcher::silent::tests::launch_failure_records_state` 通过

#### Task 3.3: Replace `launch_hermes_desktop` with generic `launch_app`

- 目标：`launcher::launch_app(app, app_id)` 查 descriptor → 解析
  `binary.{windows|macos|linux}` → macOS 走 `/usr/bin/open .app`、
  Win/Linux 直 spawn；sleep 150ms 后 exit。binary 找不到返回 Err。
- 改动文件：
  - `apps/bootstrap-installer/src-tauri/src/launcher.rs` (new function)
  - `apps/bootstrap-installer/src-tauri/src/bootstrap.rs` (remove old fn)
  - `apps/bootstrap-installer/src-tauri/src/lib.rs` (update invoke_handler)
- 验证方式：
  - `cargo test launcher::launch::tests::*` 通过：
    - macOS .app → `/usr/bin/open` 路径正确
    - Win .exe → `DETACHED_PROCESS` flag 设置正确
    - binary 缺失 → 返回带 actionable message 的 Err

#### Task 3.4: Add `get_launch_mode` Tauri command

- 目标：实现命令，按 spec `launcher-cli-modes` 中的优先级返回
  `LaunchMode { kind, target_app_id }`。
- 改动文件：
  - `apps/bootstrap-installer/src-tauri/src/launcher.rs`
  - `apps/bootstrap-installer/src-tauri/src/lib.rs`
- 验证方式：
  - `cargo build` 通过
  - 手工 DevTools 调 `invoke('get_launch_mode')` 返回正确 shape

#### Task 3.5: Extend `lib.rs` setup hook dispatch

- 目标：按 `launcher-cli-modes` 中的 dispatch table 改造 setup 钩子；
  折叠现有 macOS-only 快路径到 portable silent flow。
- 改动文件：
  - `apps/bootstrap-installer/src-tauri/src/lib.rs`
- 验证方式：
  - `cargo build` 通过
  - **V1 (first_install shows welcome)** 通过 E2E
  - **V3 (silent no-op when no update)** 通过 E2E
  - **V4 (silent skip on no network)** 通过 E2E
  - **V7 (`--launch hermes` no UI)** 通过 E2E

#### Task 3.6: AtomicBool concurrency guard for `run_silent_default`

- 目标：`static SILENT_DEFAULT_LOCK: AtomicBool`，`compare_exchange`
  防止重入；保留 `UPDATE_RUNNING`（V20 不变）。
- 改动文件：
  - `apps/bootstrap-installer/src-tauri/src/launcher.rs`
- 验证方式：
  - `cargo test launcher::silent::tests::double_invocation_noop` 通过

#### Task 3.7: E2E V1/V2/V3/V4/V7

- 目标：跑 spec 中 V1/V2/V3/V4/V7 全部场景。
- 改动文件：无
- 验证方式：
  - V1：删 state.json + 启动 → welcome 屏渲染
  - V2：完成 install + 点 Launch Hermes → desktop 启动 + installer exit
  - V3：online + no update → 无主窗口 + 1s 内 Hermes Desktop 出现
  - V4：断网 → 无主窗口 + Hermes Desktop 启动 + 无可见错误
  - V7：`--launch hermes` → Hermes Desktop 启动 + 不显示窗口
  - **里程碑**：M3 完成（静默模式全链路）

---

### M4 — Frontend: store + new routes + components

**Goal**: 实现 Home / AppDetail / Settings 三个新路由 + AppTile /
PendingUpdateBanner / MiniProgress 三个新组件；改造 store.ts 与 4 现有路由。

#### Task 4.1: Update `store.ts` per-app atoms

- 目标：增加 `$bootstrapByApp`, `$launchMode`, `$currentAppId`, `$apps`,
  `$appsList`, `$launcherState`, `$launcherConfig`, `$pendingUpdates`,
  `$updateCheckStatus`；事件 listener 按 `payload.appId` 路由；legacy
  `$bootstrap` 改为 computed view。
- 改动文件：
  - `apps/bootstrap-installer/src/store.ts`
- 验证方式：
  - `npm run typecheck` 通过
  - vitest `store.test.ts`：每个 event 都路由到正确的 `$bootstrapByApp`

#### Task 4.2: Parameterize existing 4 routes

- 目标：`progress.tsx` / `success.tsx` / `failure.tsx` 接受 `appId`；
  `progress` 从 `bootstrap.stages[currentStage].info.title` 读标题；
  `success` 调 `launchApp(appId)`；`failure` 调
  `applyPendingUpdate(appId)` 或 `startInstall(appId)`。
- 改动文件：
  - `apps/bootstrap-installer/src/routes/welcome.tsx`
  - `apps/bootstrap-installer/src/routes/progress.tsx`
  - `apps/bootstrap-installer/src/routes/success.tsx`
  - `apps/bootstrap-installer/src/routes/failure.tsx`
- 验证方式：
  - `npm run typecheck` 通过
  - vitest 各 route 渲染 smoke test

#### Task 4.3: Implement `lib/launcher-mode.ts`

- 目标：`resolveInitialRoute()` 调 `get_launch_mode`，按 spec 表格
  映射到 `Route`。
- 改动文件：
  - `apps/bootstrap-installer/src/lib/launcher-mode.ts` (new)
- 验证方式：
  - vitest `launcher-mode.test.ts`：5 种 `LaunchMode.kind` 全部覆盖

#### Task 4.4: Implement `components/app-tile.tsx`

- 目标：icon / name / status badge / 主按钮 / `⋯` 菜单；状态徽章逻辑
  按 spec 实现（installed / update available / downloading / not installed /
  launcher too old）。
- 改动文件：
  - `apps/bootstrap-installer/src/components/app-tile.tsx` (new)
- 验证方式：
  - vitest snapshot 5 种状态

#### Task 4.5: Implement `components/pending-update-banner.tsx`

- 目标：当 `Object.keys(pendingUpdates).length > 0` 时显示；
  Install now / Later 两个按钮；session-only dismiss。
- 改动文件：
  - `apps/bootstrap-installer/src/components/pending-update-banner.tsx` (new)
- 验证方式：
  - vitest 渲染 smoke test

#### Task 4.6: Implement `components/mini-progress.tsx`

- 目标：320x80 浮窗，无标题栏，置顶；用于 `--launch <id>` 模式下
  显示 install 进度。
- 改动文件：
  - `apps/bootstrap-installer/src/components/mini-progress.tsx` (new)
- 验证方式：
  - vitest 渲染 smoke test

#### Task 4.7: Implement `routes/home.tsx`

- 目标：header + 条件 banner + tile 网格 + footer（HERMES_HOME / log path）。
  Default app tile 排第一位。
- 改动文件：
  - `apps/bootstrap-installer/src/routes/home.tsx` (new)
- 验证方式：
  - vitest：空 / 1 app / 2 apps 三种渲染

#### Task 4.8: Implement `routes/app-detail.tsx`

- 目标：图标 + name + status badge + 版本信息 + install root 路径 +
  时间戳 + 6 个 action 按钮 + 可折叠 log 预览；Repair 和
  Uninstall (full) 二次确认。
- 改动文件：
  - `apps/bootstrap-installer/src/routes/app-detail.tsx` (new)
- 验证方式：
  - vitest：按钮 enable/disable 按状态切换

#### Task 4.9: Implement `routes/settings.tsx`

- 目标：Repo 编辑表单 + Update 偏好 + Diagnostics 区。
  Save 写回 yaml；Reset 删除 yaml。
- 改动文件：
  - `apps/bootstrap-installer/src/routes/settings.tsx` (new)
- 验证方式：
  - vitest：Save / Reset 按钮触发正确 invoke

#### Task 4.10: Update `app.tsx` to render 7 routes

- 目标：根据 `$route` 切 welcome / home / app-detail / settings /
  progress / success / failure。
- 改动文件：
  - `apps/bootstrap-installer/src/app.tsx`
- 验证方式：
  - `npm run typecheck` 通过
  - vitest 各 route 渲染

#### Task 4.11: TS unit tests for new modules

- 目标：launcher-mode 路由解析 + store per-app 事件路由 + tile 状态
  派生。
- 改动文件：
  - 各新增文件旁 `.test.tsx`
- 验证方式：
  - `npm run typecheck` 通过
  - vitest 全套通过

#### Task 4.12: E2E V6 + V8

- 目标：
  - V6：`--settings` → Home 屏显示 catalog apps
  - V8：Home 屏点 Install update → progress 屏 → success 屏
- 改动文件：无
- 验证方式：
  - E2E 走通 V6 + V8 完整路径
  - **里程碑**：M4 完成（前端可独立演示）

---

### M5 — Pending update flow

**Goal**: 完整实现后台预下载 + 应用 pending update + 缓存校验。

#### Task 5.1: Implement `launcher::pre_download_update`

- 目标：调 `install_script::resolve()`；下载到 bootstrap-cache；
  更新 state 到 `status="ready"`；emit progress events on bootstrap channel。
- 改动文件：
  - `apps/bootstrap-installer/src-tauri/src/launcher.rs`
- 验证方式：
  - `cargo test launcher::update::tests::pre_download_writes_ready_state` 通过

#### Task 5.2: Implement `launcher::apply_pending_update`

- 目标：复制 cached script + 跑参数化 bootstrap；成功后更新
  `state.installed[id]`；清理 `pending_updates[id]`。
- 改动文件：
  - `apps/bootstrap-installer/src-tauri/src/launcher.rs`
- 验证方式：
  - `cargo test launcher::update::tests::*` 通过：
    - cached script 使用
    - 应用失败 → pending entry 保留

#### Task 5.3: Implement `launcher::validate_pending_cache`

- 目标：每次启动检查 `pending_updates[id].downloaded_script` 是否存在；
  丢失则降级到 `status="failed"`。
- 改动文件：
  - `apps/bootstrap-installer/src-tauri/src/launcher.rs`
- 验证方式：
  - `cargo test launcher::state::tests::pending_cache_miss_degrades_to_failed` 通过

#### Task 5.4: Add Tauri commands for update flow

- 目标：`pre_download_update(id)`, `apply_pending_update(id)`；
  写入 `lib.rs::invoke_handler!`。
- 改动文件：
  - `apps/bootstrap-installer/src-tauri/src/launcher.rs`
  - `apps/bootstrap-installer/src-tauri/src/lib.rs`
- 验证方式：
  - `cargo build` 通过
  - 手工 DevTools 验证 invoke

#### Task 5.5: Wire banner "Install now" to `apply_pending_update`

- 目标：Home 屏 banner 点 Install now → 串行对每个 ready 的 id 调
  `apply_pending_update`。
- 改动文件：
  - `apps/bootstrap-installer/src/components/pending-update-banner.tsx`
- 验证方式：
  - vitest：点击触发串行 apply

#### Task 5.6: E2E V5 + V19

- 目标：
  - V5：silent 模式预下载写入 `state.pending_updates[id].status="ready"`
  - V19：cache 文件被删 → state 降级到 failed → 下次启动重新下载
- 改动文件：无
- 验证方式：
  - E2E 走通
  - **里程碑**：M5 完成

---

### M6 — Uninstall + repair + settings editor

**Goal**: 完整实现 light / full uninstall、repair、open_app_settings、
settings 编辑器。

#### Task 6.1: Implement `launcher::uninstall_app` (light + full)

- 目标：
  - light：调 `-Uninstall`（如支持）+ 删 install_root + 保留 HERMES_HOME
  - full：备份 state.json + 删整个 HERMES_HOME + 重建空目录
- 改动文件：
  - `apps/bootstrap-installer/src-tauri/src/launcher.rs`
- 验证方式：
  - `cargo test launcher::uninstall::tests::*` 通过：
    - light 保留 logs / config / 其他 app state
    - full 备份 .bak.<ts> + 重建空目录

#### Task 6.2: Implement `launcher::repair_app`

- 目标：等价 `apply_pending_update`，但若 cached script 缺失则 fetch 新。
- 改动文件：
  - `apps/bootstrap-installer/src-tauri/src/launcher.rs`
- 验证方式：
  - `cargo test launcher::repair::tests::*` 通过：
    - 无 pending update → fetch + run
    - 有 pending update → 用 cached

#### Task 6.3: Implement `launcher::open_app_settings`

- 目标：用 `opener` 插件打开 `app_settings_url`；null 时返回 Err。
- 改动文件：
  - `apps/bootstrap-installer/src-tauri/src/launcher.rs`
- 验证方式：
  - `cargo test launcher::settings::tests::*` 通过

#### Task 6.4: Wire Tauri commands for uninstall/repair/settings

- 目标：`uninstall_app(id, scope)`, `repair_app(id)`,
  `open_app_settings(id)`；写入 `lib.rs::invoke_handler!`。
- 改动文件：
  - `apps/bootstrap-installer/src-tauri/src/launcher.rs`
  - `apps/bootstrap-installer/src-tauri/src/lib.rs`
- 验证方式：
  - `cargo build` 通过

#### Task 6.5: Wire settings screen Save / Reset to Tauri commands

- 目标：Settings 屏 Save 调 `saveLauncherConfig`；Reset 调
  `set_launcher_config` 传空（删除文件）。
- 改动文件：
  - `apps/bootstrap-installer/src/routes/settings.tsx`
- 验证方式：
  - vitest：按钮触发正确 invoke

#### Task 6.6: E2E V9 / V10 / V11 / V14 / V16 / V17 / V18

- 目标：
  - V9：light uninstall 保留 HERMES_HOME 其余
  - V10：full uninstall 二次确认 + 备份
  - V11：损坏 state.json 恢复
  - V14：yaml 覆盖 build-time
  - V16：`--repair` 走 welcome
  - V17：app.json 缺 binary.macos 显示 tile 提示
  - V18：min_launcher_version 高于启动器显示徽章
- 改动文件：无
- 验证方式：
  - E2E 全部走通
  - **里程碑**：M6 完成

---

### M7 — Verification

**Goal**: 跑完整 openspec validate + cargo test + vitest + E2E 全场景。

#### Task 7.1: Final `openspec validate`

- 目标：`openspec validate add-launcher-features-to-bootstrap-installer --strict` 通过。
- 改动文件：无
- 验证方式：
  - 命令退出码 0

#### Task 7.2: Full Rust test suite (V20 final confirmation)

- 目标：`cargo test -p hermes-bootstrap` 全绿。
- 改动文件：无
- 验证方式：
  - 命令退出码 0
  - 输出无 FAIL

#### Task 7.3: Full TypeScript test suite

- 目标：`npm run typecheck` + vitest 全绿。
- 改动文件：无
- 验证方式：
  - 两命令退出码 0

#### Task 7.4: E2E V1-V20

- 目标：spec proposal.md 中全部 V1-V20 走一遍。
- 改动文件：无
- 验证方式：
  - 用 `docs/superpowers/plans/2026-MM-DD-add-launcher-features-to-bootstrap-installer.md`
    中的 E2E checklist 全部勾选

#### Task 7.5: Update `AGENTS.md`

- 目标：若 change 引入新约定（如多 app catalog、三层 repo 配置），
  补充到 `hermes-yclaw/AGENTS.md`。
- 改动文件：
  - `hermes-yclaw/AGENTS.md` (if needed)
- 验证方式：
  - 文档结构合理 + 一致性

#### Task 7.6: Commit with conventional commit messages

- 目标：M1-M7 每个 milestone 一个 commit（或合并 M2+M3），
  全部 commit 跑过本地 `cargo test` + `npm run typecheck`。
- 改动文件：无（git 操作）
- 验证方式：
  - `git log --oneline` 显示清晰的提交链

---

## Summary

| Milestone | Steps   | Person-days | Critical E2E |
|-----------|---------|-------------|--------------|
| M1        | 1.1–1.7 (7)  | ~2          | V20 unit     |
| M2        | 2.1–2.8 (8)  | ~3          | V12/V13/V14  |
| M3        | 3.1–3.7 (7)  | ~2          | V1/V2/V3/V4/V7 |
| M4        | 4.1–4.12 (12) | ~4         | V6/V8        |
| M5        | 5.1–5.6 (6)  | ~2          | V5/V19       |
| M6        | 6.1–6.6 (6)  | ~2          | V9/V10/V11/V14/V16/V17/V18 |
| M7        | 7.1–7.6 (6)  | ~0.5        | all          |
| **Total** | **52 steps** | **~15.5** | **V1–V20** |

Minimum demonstrable (M1+M2+M3+M4) = 34 steps, ~11 person-days.

## 下一步

按 `openflow/build.md` 流程，本 plan-ready.md 已被锁定。下一步用
`/openflow build` 调用 Superpowers `writing-plans` 生成
`docs/superpowers/plans/YYYY-MM-DD-add-launcher-features-to-bootstrap-installer.md`
作为可执行的细粒度计划（含 TDD 铁律与 checkbox 跟踪）。