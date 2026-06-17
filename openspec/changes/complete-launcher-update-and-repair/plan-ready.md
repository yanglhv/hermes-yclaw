# 实现计划：complete-launcher-update-and-repair

## 来源
- 提案：openspec/changes/complete-launcher-update-and-repair/proposal.md
- 设计：openspec/changes/complete-launcher-update-and-repair/design.md
- 规格：openspec/changes/complete-launcher-update-and-repair/specs/
- 任务：openspec/changes/complete-launcher-update-and-repair/tasks.md

## 工作目录约定
所有路径相对 `hermes-yclaw/`。`cargo` 命令在
`hermes-yclaw/apps/bootstrap-installer/src-tauri/` 下运行；`npm` 命令在
`hermes-yclaw/apps/bootstrap-installer/` 下运行。

## 实现步骤（按执行依赖排序）

### Task 1: semver_lt 版本比较助手 [M1 基础，无依赖]
- 目标：提供 launcher 与 app.min_launcher_version 的语义版本比较（数值，非字典序）。
- 改动文件：`apps/bootstrap-installer/src-tauri/src/app.rs`（或新建 `version.rs` 并在 `lib.rs` 声明）。
- 步骤：
  1. 写失败测试：`semver_lt("0.9.0","0.10.0")==true`、`("0.1.0","0.1.0")==false`、`("0.10.0","0.9.0")==false`、`("x","0.1.0")==false`。
  2. 实现 `pub fn semver_lt(a: &str, b: &str) -> bool`：按 `.` split，三段 parse::<u64> 数值比较；任一解析失败返回 `false`（安全默认）。
- 验证方式：`cargo test semver_lt` 通过。

### Task 2: list_available_apps 计算 launcher_too_old [M1]
- 目标：用真实比较替换 `launcher_too_old: false` 硬编码。
- 改动文件：`apps/bootstrap-installer/src-tauri/src/launcher/commands.rs:64`。
- 步骤：
  1. 把 `launcher_too_old: false` 改为
     `launcher_too_old: crate::app::semver_lt(env!("CARGO_PKG_VERSION"), descriptor.min_launcher_version.as_str())`。
- 验证方式：`cargo check`；`cargo test` 全绿。

### Task 3: 提升 launcher 版本到 0.1.0 [M1]
- 目标：让 Hermes（min 0.1.0）通过门控，避免开发期 tile 永远显示 ⚠。
- 改动文件：`apps/bootstrap-installer/src-tauri/Cargo.toml`（`version = "0.0.1"` → `"0.1.0"`）。
- 步骤：
  1. 改版本号。
- 验证方式：`cargo build` 通过；`env!("CARGO_PKG_VERSION")` 在测试中打印为 `0.1.0`。

### Task 4: 获取 repo HEAD SHA 助手 [M2]
- 目标：为 check_for_updates 提供“最新 commit”。
- 改动文件：`apps/bootstrap-installer/src-tauri/src/launcher/update.rs`。
- 步骤：
  1. 写失败测试（mockito）：`GET /repos/o/n/commits/{ref}` 返回 `{"sha":"def456",...}` → 返回 `Ok("def456")`；5xx/离线 → `Err`。
  2. 实现 `pub async fn fetch_head_sha(api_base, repo) -> Result<String, ...>`：reqwest GET，解析 JSON `sha` 字段，2s timeout。
- 验证方式：`cargo test fetch_head_sha` 通过。

### Task 5: 重写 check_for_updates [M2]
- 目标：真实比较 installed_commit vs HEAD SHA，填充 pending_updates。
- 改动文件：`apps/bootstrap-installer/src-tauri/src/launcher/commands.rs`（`check_for_updates`）。
- 步骤：
  1. 写失败测试（mockito + 临时 state）：installed_commit≠HEAD → pending[id]=Avail 且返回含 id；相等 → 不加入；离线 → 返回现有 pending 不变、不 panic。
  2. 实现：resolve RepoRef → probe_network → fetch_head_sha → 遍历 installed 比较提交；`Ready` 条目不降级为 `Avail`；保存 state。
- 验证方式：`cargo test check_for_updates` 通过。

### Task 6: pre_download_update 真实下载 [M3]
- 目标：把脚本写到 cached_path 并置 status=Ready。
- 改动文件：`apps/bootstrap-installer/src-tauri/src/launcher/commands.rs:120`（pre_download_update）。
- 步骤：
  1. 写失败测试（mockito）：resolve 成功 → cached_path 文件存在且 state.pending[id].status=Ready、downloaded_script 已设；resolve 失败 → status=Failed、last_error 已设、无 panic。
  2. 实现：在 `tokio::spawn` 内 resolve RepoRef + script_path，调 `install_script::resolve`，确保结果落到 `cached_path(kind, ref_name)`（不同则拷贝），写入 state 并保存；失败置 Failed。
- 验证方式：`cargo test pre_download_update` 通过。

### Task 7: 给 run_bootstrap 的 emit 站点传入 app_id [M4]
- 目标：让 apply/repair 发出的事件带 app_id，供前端按应用路由。
- 改动文件：`apps/bootstrap-installer/src-tauri/src/bootstrap.rs`（`run_bootstrap` 签名 + 所有 `emit_event` / `BootstrapEvent::*{app_id: None,...}` 站点）。
- 步骤：
  1. 给 `run_bootstrap` 增加 `app_id: Option<String>` 参数，逐个 emit 站点把 `app_id: None` 改为透传该参数。
  2. 更新 `start_bootstrap` 调用处传 `None`（保持现有 Hermes-only 行为不变）。
- 验证方式：`cargo test`（含既有 906+1073 bootstrap/update 测试）全绿。

### Task 8: 提取 run_app_install 共享驱动 [M4]
- 目标：apply 与 repair 共用的“跑 bootstrap + 完成后更新 installed[id]”。
- 改动文件：`apps/bootstrap-installer/src-tauri/src/launcher/commands.rs`（新增驱动；可能借 `bootstrap.rs` 的 worker 任务体）。
- 步骤：
  1. 实现 `async fn run_app_install(app: AppHandle, state: &LauncherStateHandle, descriptor: &AppDescriptor, id: &str, via: &str)`：spawn 与 `start_bootstrap` 相同的 worker（传 `Some(id.into())` 作为 app_id），在 `complete` 时锁 state 更新 `installed[id]`（commit/ref/installed_at=now/installed_via=via）并清 `pending_updates[id]`；`failed` 时不变。
  2. 单元测试用模拟 worker 结果：complete → installed 更新 + pending 清空；failed → 状态不变。
- 验证方式：`cargo test run_app_install` 通过。

### Task 9: apply_pending_update 真实实现 [M4, B1]
- 目标：用缓存脚本端到端跑安装（替换 "deferred to M6" stub）。
- 改动文件：`apps/bootstrap-installer/src-tauri/src/launcher/commands.rs:151`。
- 步骤：
  1. 写测试：无 ready pending（缺失/非 Ready/脚本文件不存在）→ 返回 `Err`、不跑 bootstrap。
  2. 实现：校验 `pending_updates[id].status==Ready` 且 `downloaded_script` 存在（否则 Err），调 `run_app_install(..., "update")`。
- 验证方式：`cargo test apply_pending_update` 通过；前端 "Install now" 能进入 progress。

### Task 10: repair_app 真实实现 [M4, B2]
- 目标：repair 跑安装（缓存优先，否则 fresh），替换 stub。
- 改动文件：`apps/bootstrap-installer/src-tauri/src/launcher/commands.rs:427`。
- 步骤：
  1. 实现：无前置条件，直接调 `run_app_install(..., "repair")`；worker 内 `install_script::resolve` 的 cached→network 优先级自动选脚本。
  2. 删除现有 "deferred to M6" 日志。
- 验证方式：`cargo test repair_app` 通过；Repair 按钮触发 progress。

### Task 11: 验收与校验 [M5]
- 目标：全量回归 + 验收走查 + openspec 校验。
- 改动文件：无（只跑命令）。
- 步骤：
  1. `cargo test`（crate）全绿；新增测试通过；既有 `lock_probe_paths`/`light_uninstall` 失败保持不变（T1 范围外）。
  2. `npm run typecheck` 通过（前端应无结构变更）。
  3. 验收走查 V5/V8/V16/V18/V19（见 tasks.md M5.3）。
  4. `openspec validate complete-launcher-update-and-repair --strict` 通过。
- 验证方式：上述命令全部 exit 0。

## 依赖顺序
Task 1→2→3（M1 门控）可独立先行；Task 4→5（M2 check）、Task 6（M3 predl）
彼此独立；Task 7→8→9/10（M4 apply/repair）依赖 bootstrap app_id 改造；
Task 11 最后总验。
