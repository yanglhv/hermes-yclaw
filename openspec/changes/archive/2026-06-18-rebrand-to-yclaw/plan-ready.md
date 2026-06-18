# 实现计划：rebrand-to-yclaw

## 来源
- 提案：`openspec/changes/rebrand-to-yclaw/proposal.md`
- 设计：`openspec/changes/rebrand-to-yclaw/design.md`
- 规格：`openspec/changes/rebrand-to-yclaw/specs/user-visible-branding/spec.md`
- 任务：`openspec/changes/rebrand-to-yclaw/tasks.md`

## 实现原则

1. **每条步骤标"目标 / 改动文件 / 验证方式"**——任何一步都能在 2-5 分钟内完成并自验。
2. **按执行依赖排序**——先浅后深：先改最小可见字符串(index.html title、About 头部),再改桌面端 i18n 4 语言(锁步),再改 intro JSONL、thinking/loading 通知,再改 web 端;**测试断言同步** 排在对应的源代码改动之后;**跨面 audit** 排在所有改动之后。
3. **后端标识全部保留**——以下内容**绝对不动**:`package.json` 的 `name`/`productName`/`appId`、协议 `hermes://`、IPC 频道 `hermes:*`、`HERMES_*` 环境变量、Python 包、CLI 子命令、`~/.hermes/` 数据目录、`localStorage` key、`X-Hermes-Session-Token`、`__HERMES_*` runtime globals、`HERMES_BASE_PATH`、`HERMES_DOCS_URL`、skill id `hermes-index`、`__hermes_memory_builtin__`、下载文件名 `hermes-config.json`、所有 `<code>hermes ...</code>` 形式的内嵌 CLI 命令。
4. **唯一例外(用户面 brand 字符串)**:Telegram `bot_name: "Hermes Agent"` → `"YClaw Agent"`、Matrix placeholder `@hermes:example.org` → `@yclaw:example.org`。
5. **storage key / preset id / IPC channel** 与 `label`/`description` 是不同概念——前者保持原字面,后者跟随主品牌重命名。

---

## 实现步骤

### Task 1: 桌面端窗口 `<title>` 改为 YClaw
- **目标**：桌面端 BrowserWindow 加载时,OS 级标题栏和 `<title>` 元素显示 "YClaw" 而不是 "Hermes"。
- **改动文件**：
  - `apps/desktop/index.html:11` — `<title>Hermes</title>` → `<title>YClaw</title>`
- **验证方式**：
  - `cd apps/desktop && npm run dev`
  - OS 标题栏和 DevTools `<title>` 元素都应显示 `YClaw`
  - `rg -n "Hermes" apps/desktop/index.html` 应只命中 `localStorage` key(`hermes-boot-background` / `hermes-boot-color-scheme`),不命中 `<title>`

### Task 2: 桌面端 About 面板改为 YClaw Desktop
- **目标**：Settings → About 面板的 heading / version 行显示 "YClaw Desktop"。
- **改动文件**：
  - `apps/desktop/src/i18n/en.ts:331` — `'About Hermes Desktop'` → `'About YClaw Desktop'`
  - `apps/desktop/src/i18n/en.ts:374` — `'Hermes Desktop'` → `'YClaw Desktop'`(settings heading)
  - `apps/desktop/src/i18n/en.ts:1573` — `desktopVersion: version => \`Hermes Desktop v${version}\`` → `desktopVersion: version => \`YClaw Desktop v${version}\``
  - `apps/desktop/src/i18n/ja.ts:245,496` — 同步
  - `apps/desktop/src/i18n/zh.ts:573` — 同步
  - `apps/desktop/src/i18n/zh-hant.ts:485` — 同步
- **验证方式**：
  - 桌面端 dev 启动 → Settings → About → heading 显示 "YClaw Desktop",version 行显示 "YClaw Desktop v<x.y.z>"
  - 切换 ja / zh / zh-hant 三个语种,About 面板 heading 都显示对应 "YClaw Desktop"

### Task 3: 桌面端 i18n 4 语言 brand 字符串统一替换（en 先行）
- **目标**：把 `en.ts` 里所有"用户可见"的 brand 字面量替换为 "YClaw" 系列(键名、类型、函数名、storage key 保持原名)。
- **改动文件**：
  - `apps/desktop/src/i18n/en.ts` — 替换以下键的 string value:
    - `boot.ready`, `boot.loadingSettings`, `boot.startingHermesDesktop`, `boot.backgroundExited`, `boot.backgroundExitedDuringStartup`, `boot.title`, `boot.backendOutdated`, `boot.updateHermes`, `boot.405`
    - `prompts.inputBody`, `prompts.turnDoneTitle`
    - `settings.resetConfirm`, `notifications.focusedHint`, `notifications.clarifyDesc`, `notifications.turnFinishedDesc`
    - `notifications.testTitle`(`'Hermes'`)
    - `settings.about`(`'About Hermes Desktop'`)
    - `appearance.colorModeDesc`, `updates.heading`, `updates.body`, `updates.loading`
    - `gateway.localDesc`, `gateway.localDesc2`, `gateway.remoteDesc`, `gateway.remoteUrlDesc`, `gateway.restartingMessage`, `gateway.connectedTo`
    - `providers.subscriptionPitch`
    - `menu.settings.title`, `menu.settings.detail`
    - `status.hermesActiveSessions`, `status.updateHermes`
    - `matrix.MATRIX_USER_ID.placeholder`(`'@hermes:example.org'` → `'@yclaw:example.org'`)
    - `profiles.createDesc`
    - `cron.createDesc`
    - `intro.placeholderStarting`, `intro.placeholderReconnecting`, `intro.placeholder`, `slashmenu.quit`(`'exit hermes'`)
    - `intro.attachUrlDesc`, `updateOverlay.restart`, `updateOverlay.unsupportedMessage`, `updateOverlay.availableBody`, `updateOverlay.availableBodyBackend`, `updateOverlay.manualBody`, `updateOverlay.manualPickedUp`, `updateOverlay.applyingBody`, `updateOverlay.applyingBodyRemote`, `updateOverlay.applyingClose`
    - `installOverlay.oneTimeTitle`, `installOverlay.settingUpTitle`, `installOverlay.errorBody`, `installOverlay.body`, `installOverlay.headerTitle`, `installOverlay.preparingInstall`, `installOverlay.starting`, `installOverlay.featuredPitch`, `installOverlay.openAIEndpointDesc`, `installOverlay.flow.device_code`, `installOverlay.flow.loopback`, `installOverlay.authorizeThere`, `installOverlay.openedBrowser`
    - `about.desktopVersion`, `about.gatewayTitle`
    - `preview.largeBody`, `preview.restarting`, `preview.askRestart`, `preview.lookingRestart`, `preview.restartingMessage`, `preview.finishedRestarting`, `preview.restartFailedMessage`, `preview.restartFailedNoResult`, `preview.loadingResponse`
    - `gatewayDisconnected`(在多处的根级 `gatewayDisconnected`)
    - `prompts.sudoDesc`, `prompts.secretDesc`
    - `imageTools.restartToUseSaveImage`, `imageTools.restartToSaveImages`
    - `gateway.timeout`
  - **不要改**:`status.startingHermesDesktop` 类型的**键名**(只改值),所有 `localStorage` key 常量,所有 `'hermesDesktop.*'` 桥接调用
- **验证方式**：
  - `rg -n "[\"'][^\"']*\\b[Hh]ermes[^\"']*[\"']" apps/desktop/src/i18n/en.ts` 应只剩键名命中(以 `:` 开头但不在字符串 value 里的不算)
  - 字符串 value 里的 brand 字面应清零

### Task 4: 桌面端 i18n 锁步到 ja / zh / zh-hant
- **目标**：让 ja / zh / zh-hant 三个桌面端语种的 brand-token 替换数与 `en.ts` 同步,避免漂移。
- **改动文件**：
  - `apps/desktop/src/i18n/ja.ts` — 同步 Task 3 列出的所有 key 的 string value(`Hermes Desktop` → `YClaw Desktop`、`Hermes のセッション履歴` → `YClaw のセッション履歴` 等)
  - `apps/desktop/src/i18n/zh.ts:573,640,684` — `Hermes Desktop` → `YClaw Desktop`、`Hermes 后端` → `YClaw 后端`
  - `apps/desktop/src/i18n/zh-hant.ts:46,485,552,596` — 同步
- **验证方式**：
  - `rg -c "YClaw" apps/desktop/src/i18n/{en,ja,zh,zh-hant}.ts` — 四个文件的 YClaw 计数应一致(±0)
  - `rg -c "Hermes" apps/desktop/src/i18n/{en,ja,zh,zh-hant}.ts` — 四个文件的残留 Hermes 计数应一致(都是 0 或只命中键名/类型)

### Task 5: 桌面端 intro JSONL 5 personality 重写
- **目标**：`apps/desktop/src/components/chat/intro-copy.jsonl` 里 5 条 personality 记录的 `headline` + `body` 全部去掉 "Hermes" 品牌 token,改为 "YClaw" 系列(保持各 personality 自身的文风:小写/大写、emoji 风格)。
- **改动文件**：
  - `apps/desktop/src/components/chat/intro-copy.jsonl`
    - 第 27 行 (`kawaii`): `headline: "hermes-chan is here! <3"` → `headline: "yclaw-chan is here! :3"`；body 提到 "hermes-chan" 处同步改 "yclaw-chan"
    - 第 37 行 (`pirate`): `headline: "Hermes at the helm, arrr"` → `headline: "YClaw at the helm, arrr"`；body 同步
    - 第 53 行 (`noir`): `headline: "Hermes. Code investigator."` → `headline: "YClaw. Code investigator."`；body 同步
    - 第 57 行 (`uwu`): `headline: "hermes-san is wistening"` → `headline: "yclaw-san is wistening"`；body 同步
    - 第 71 行 (`none`): `headline: "Hermes Agent is ready."` → `headline: "YClaw is ready."`；body 改为 "Ask a question, paste an error, or point me at a repo. I can read code, run tools, and help you ship."(去掉品牌 reference)
- **验证方式**：
  - 桌面端 dev 启动 → 新建会话 → 看 5 种 personality 切换(headline / body 都应没有 "Hermes" / "hermes" 字样)
  - `rg -n "[Hh]ermes" apps/desktop/src/components/chat/intro-copy.jsonl` 应为 0 命中

### Task 6: 桌面端 thinking / loading / notification / boot-error 文案
- **目标**：把 thinking label、loading response 状态、成功通知标题、boot 错误提示这 4 处用户面字符串从 "Hermes" 改为 "YClaw"。
- **改动文件**：
  - `apps/desktop/src/components/assistant-ui/thread.tsx:413` — `'Hermes is thinking'` → `'YClaw is thinking'`
  - `apps/desktop/src/components/assistant-ui/streaming.test.tsx:389,397` — `name: 'Hermes is loading a response'` → `name: 'YClaw is loading a response'`(两处断言)
  - `apps/desktop/src/store/onboarding.ts:192` — `title: 'Hermes is ready'` → `title: 'YClaw is ready'`
  - `apps/desktop/src/components/gateway-connecting-overlay.test.tsx:63` — `error: 'Hermes backend did not become ready'` → `error: 'YClaw backend did not become ready'`
- **验证方式**：
  - `cd apps/desktop && npm run test:ui` — 全部 vitest 通过
  - `scripts/run_tests.sh apps/desktop/ -q` — 全部 Node test 通过
  - 桌面端 dev 启动,触发一次模型回合 → thinking label 显示 "YClaw is thinking"

### Task 7: 桌面端 theme 展示名改为 YClaw
- **目标**：`apps/desktop/src/themes/presets.ts` 里所有用户面 `label` / `description` / `tooltip` 改为 "YClaw" 系列,所有 preset `id` 与 `localStorage` key 保持原名。
- **改动文件**：
  - `apps/desktop/src/themes/presets.ts:32` 注释 — `Hermes desktop identity` → `YClaw desktop identity`
  - `apps/desktop/src/themes/presets.ts` 内每个 preset 的 `label` 字段(如 `label: "Hermes Teal"` → `label: "YClaw Teal"`)和 `description` 字段
  - **不要改**:preset `id`(如 `default`、`nous`)、`hermes: 'nous'` 这种 use-skin-command 映射键、`SKIN_KEY` / `MODE_KEY` 等 localStorage key
- **验证方式**：
  - 桌面端 dev 启动 → 主题切换器 → 看到的 label 都是 "YClaw ..."
  - 切换主题后,重启 app,主题仍保持(说明内部 id 未被破坏)
  - `rg -n "[Hh]ermes" apps/desktop/src/themes/presets.ts` — 应只剩注释/类型/id 命中,无 string value 命中

### Task 8: 桌面端测试断言全量同步
- **目标**：所有桌面端 test 文件里 brand 字面量断言改为 "YClaw",所有 preserved identifier 断言(IPC channel、storage key、CLI 命令等)保持原字面。
- **改动文件**：
  - `apps/desktop/src/store/updates.test.ts` — 改 `name: 'update'` / `'hermes update'`(命令字面保留)以外的所有 brand 字符串
  - `apps/desktop/src/store/model-visibility.test.ts` — 保留 `hermes-3-llama-3.1-70b` 这种**模型 id**(不是品牌);如果有 brand 字面再改
  - `apps/desktop/src/store/subagents.test.ts` — `'pattern=hermes'` 这种 fixture 是测试输入,可能保留;若有用户可见 brand 字面,改
  - `apps/desktop/src/components/assistant-ui/directive-text.test.ts` — `describe('hermesDirectiveFormatter.parse', ...)` 改 `describe('yclawDirectiveFormatter.parse', ...)`(同步 `directive-text.tsx` 里的 `hermesDirectiveFormatter` 名称)
  - `apps/desktop/src/components/assistant-ui/message-render-boundary.test.tsx`、`tool-approval.test.tsx`、`tool-approval-group.test.tsx`、`user-message-edit.test.tsx` — 扫一遍 brand 字面
  - `apps/desktop/src/components/desktop-onboarding-overlay.test.tsx:92` — `'hermes-onboarding-skipped-v1'` 是 storage key,**保留**
  - `apps/desktop/src/components/language-switcher.test.tsx` — 看是否有用户可见 brand 字面
  - **不要改**:`'hermes-desktop-onboarded-v1'`、`'hermes-onboarding-skipped-v1'`、`'hermes-onboarding-show-all-v1'`、`'hermes-desktop-theme-v2'`、`'hermes-desktop-mode-v1'`、`'hermes-desktop-active-profile-v1'`、`'hermes.desktop.*'` 系列的 storage key；`hermes:connection` / `hermes:backend:touch` 系列的 IPC channel；`hermesDesktop.*` 桥接名
- **验证方式**：
  - `cd apps/desktop && npm run test:ui` — vitest 全部通过
  - `cd apps/desktop && npm run typecheck` — tsc 通过
  - `scripts/run_tests.sh apps/desktop/ -q` — Node test 全部通过

### Task 9: web index.html title + 顶栏 brand
- **目标**：浏览器 tab 标题显示 "YClaw - Dashboard",web 顶栏 brand 文字显示 "YClaw"。
- **改动文件**：
  - `web/index.html:10` — `<title>Hermes Agent - Dashboard</title>` → `<title>YClaw - Dashboard</title>`
  - `web/src/App.tsx:580` — `Hermes` → `YClaw`
- **验证方式**：
  - `cd web && npm run dev` → 浏览器 tab 标题显示 "YClaw - Dashboard"
  - 顶栏左侧 brand 文字显示 "YClaw"
  - `rg -n "[Hh]ermes" web/index.html web/src/App.tsx:580` 应为 0 命中

### Task 10: web 端 i18n 19 语言 brand 字符串统一替换（en 先行）
- **目标**：`web/src/i18n/en.ts` 里所有"用户可见"brand 字面量替换为 "YClaw"(键名、类型、函数名、storage key、env var、runtime global 保持原名)。
- **改动文件**：
  - `web/src/i18n/en.ts` — 替换以下键的 string value:
    - `nav.brand`(`'Hermes Agent'` → `'YClaw'`)
    - `status.updateHermes`(`'Update Hermes'` → `'Update YClaw'`)
    - `status.updatingHermes`(`'Updating Hermes...'` → `'Updating YClaw...'`)
    - `plugins.discoverDesc`(`'Discover, install, enable and update Hermes plugins...'` → 同步)
    - `plugins.removeConfirm`(`'Remove this plugin from ~/.hermes/plugins/?'` — 路径**保留**,字面保留)
    - `skills.noSkills`(路径**保留**)
    - `skills.configPath`(`'~/.hermes/config.yaml'` — 路径**保留**)
    - `achievements.title`(`'Hermes Achievements'` → `'YClaw Achievements'`)
    - `achievements.intro`、`achievements.scanning`、`achievements.latest_hint_empty`、`achievements.secretHint`、`achievements.localScan`
    - `achievements.tweet_text`(`'Just unlocked {tier_part}"{name}" in Hermes Agent ☤'` → 同步)
  - **不要改**:`'~/.hermes/plugins/'`、`'~/.hermes/skills/'`、`'~/.hermes/config.yaml'`、`'~/.hermes/dashboard-themes/'`、所有 `__HERMES_*` runtime global 名、`HERMES_*` env var、`hermes-config.json` 下载文件名、`hermes-index` skill id、`'__hermes_memory_builtin__'`、`<code>hermes ...</code>` 命令字面
- **验证方式**：
  - `rg -c "YClaw" web/src/i18n/en.ts` — 计数应与 Tasks 11 之后其它语种的计数对齐
  - 浏览器切换语言到 en,所有用户可见 brand 字符串应为 "YClaw"

### Task 11: web 端 i18n 锁步到其余 18 个语种
- **目标**：让 web 端 18 个其他语种(`af / de / es / fr / ga / hu / it / ja / ko / pt / ru / tr / uk / zh / zh-hant`)的 brand-token 替换数与 `en.ts` 同步。
- **改动文件**：
  - `web/src/i18n/{af,de,es,fr,ga,hu,it,ja,ko,pt,ru,tr,uk,zh,zh-hant}.ts` — 同步 Task 10 列出的所有 key 的 string value,语种相关 token 保留(如日文 "を" / 中文 "的" / 西文 "Hermes" 复数化等)
  - 关键键(每个语种都有):
    - `nav.brand` → `'YClaw'` / `'YClaw Agent'`(看各语种原文)
    - `status.updateHermes` → `'Update YClaw'` 系列
    - `status.updatingHermes` → `'Updating YClaw...'` 系列
    - `plugins.discoverDesc` → 改 brand reference
    - `achievements.title` → `'YClaw Achievements'`
    - `achievements.tweet_text` → `'Just unlocked {tier_part}"{name}" in YClaw ☤'`
  - **不要改**:所有 `~/.hermes/...` 路径、所有 `__HERMES_*` runtime global 名、所有 `HERMES_*` env var、`hermes-index` skill id、`hermes-config.json`、所有 `<code>hermes ...</code>` 命令字面
- **验证方式**：
  - `for f in web/src/i18n/{af,de,en,es,fr,ga,hu,it,ja,ko,pt,ru,tr,uk,zh,zh-hant}.ts; do echo "$f: $(rg -c 'YClaw' $f)"; done` — 19 个文件的 YClaw 计数应在同一量级(允许 ±2 因为各语种 brand reference 数不同)
  - 浏览器切换到 19 种语言逐一检查 nav 区域、status 区、achievements 区,brand 字样都应为 "YClaw"

### Task 12: web 端 theme 展示名改为 YClaw
- **目标**：`web/src/themes/presets.ts` 里用户面 `label` / `description` 改为 "YClaw",`localStorage` key 与 preset `id` 保持原名。
- **改动文件**：
  - `web/src/themes/presets.ts:43,44` — `label: "Hermes Teal"` → `label: "YClaw Teal"`、`description: "Classic dark teal — the canonical Hermes look"` → `description: "Classic dark teal — the canonical YClaw look"`
  - `web/src/themes/presets.ts:288,289` — 同步 `(Large)` 变体
  - `web/src/themes/presets.ts:11,188,201` — 注释里的 "Hermes" / "Hermes teal" → "YClaw" / "YClaw teal"
  - **不要改**:preset `id`、`hermes-dashboard-theme` / `hermes-dashboard-font` / `hermes-theme-custom-css` 类的 storage key、`data-hermes-theme-css` / `data-hermes-theme-font` 类的 `data-*` 属性
- **验证方式**：
  - 浏览器打开 theme picker → label 显示 "YClaw Teal" / "YClaw Teal (Large)"
  - 切换主题后,刷新页面,主题保持(说明 storage key 未被破坏)
  - `rg -n "[Hh]ermes" web/src/themes/presets.ts` — 应只剩 id / storage key / `data-*` 命中

### Task 13: web 端错误 / 系统 / 渠道页文案
- **目标**：把 `gatewayClient.ts`、`ChatPage.tsx`、`SystemPage.tsx`、`ChannelsPage.tsx` 里用户面 brand 字面量改为 "YClaw",**所有 `<code>hermes ...</code>` 形式的内嵌 CLI 命令字面保留**。
- **改动文件**：
  - `web/src/lib/gatewayClient.ts:130` — `"Session token not available — page must be served by the Hermes dashboard"` → `"... by the YClaw dashboard"`
  - `web/src/pages/ChatPage.tsx:137` — `"Session token unavailable. Open this page through \`hermes dashboard\`, not directly."` — 改 prose 部分,`<code>hermes dashboard</code>` **保留**
  - `web/src/pages/SystemPage.tsx:423` — `"Hermes updates are managed outside this dashboard."` → `"YClaw updates are managed outside this dashboard."`
  - `web/src/pages/SystemPage.tsx:527` — `title="Update Hermes?"` → `title="Update YClaw?"`
  - `web/src/pages/SystemPage.tsx:530,531` — prose 提到 "Hermes" 处改 "YClaw",`<code>hermes update</code>` **保留**
  - `web/src/pages/SystemPage.tsx:700` — `<div ... >Hermes</div>` 标签 → `<div ... >YClaw</div>`
  - `web/src/pages/SystemPage.tsx:702` — `v{stats?.hermes_version}` 上下文里如果有 brand 标签改 "YClaw"
  - `web/src/pages/ChannelsPage.tsx:565` — `bot_name: "Hermes Agent"` → `bot_name: "YClaw Agent"`(**例外,允许改**)
  - **不要改**:`<code>hermes gateway start</code>`、`<code>hermes update</code>`、`<code>hermes portal</code>`、`<code>hermes memory setup</code>`、`<code>hermes skills search</code>`、`<code>hermes tools</code>`、`<code>hermes-config.json</code>`、`<code>~/.hermes/.env</code>`、`<code>hermes-index</code>`、`<code>__hermes_memory_builtin__</code>`、`<code>HERMES_*</code>`、`<code>X-Hermes-Session-Token</code>`、`__HERMES_*` runtime globals
- **验证方式**：
  - 浏览器打开 Channels → Telegram onboarding → 默认 bot_name 字段是 "YClaw Agent"
  - 触发 system 页 update 对话框 → 标题 "Update YClaw?"
  - 打开 system 页 Hermes version 卡片(标签)→ 显示 "YClaw"
  - 浏览器 devtools → 搜索所有 `<code>` 内容,确认每个 `hermes ...` 命令字面未被改
  - `rg -n "[Hh]ermes" web/src/pages/{ChatPage,SystemPage,ChannelsPage}.tsx web/src/lib/gatewayClient.ts` — 命中应只在 `<code>...</code>` 子串、命令字面、storage key、env var、runtime global 处

### Task 14: web 端 aria / id / className / dev-log 前缀
- **目标**：`web/src/App.tsx` 的 aria 关联字符串、`web/src/pages/ChatPage.tsx` 的 dev-log 前缀和 xterm className 全部从 "hermes" 改为 "yclaw"。
- **改动文件**：
  - `web/src/App.tsx:634` — `aria-labelledby="hermes-sidebar-plugin-nav-heading"` → `aria-labelledby="yclaw-sidebar-plugin-nav-heading"`
  - `web/src/App.tsx:644` — `id="hermes-sidebar-plugin-nav-heading"` → `id="yclaw-sidebar-plugin-nav-heading"`(与 634 同步)
  - `web/src/pages/ChatPage.tsx:473` — `"[hermes-chat] WebGL renderer unavailable; falling back to default"` → `"[yclaw-chat] WebGL renderer unavailable; falling back to default"`
  - `web/src/pages/ChatPage.tsx:895` — `className="hermes-chat-xterm-host min-h-0 min-w-0 flex-1"` → `className="yclaw-chat-xterm-host min-h-0 min-w-0 flex-1"`
- **验证方式**：
  - 浏览器 devtools → Elements 面板,sidebar plugin nav 区域,`aria-labelledby` 指向一个真实存在且 `id` 匹配的 heading
  - 浏览器 devtools → Console 触发 WebGL fallback 路径(可通过降级 GPU 模拟)→ 看到 `[yclaw-chat] ...` 日志
  - 浏览器 devtools → Elements 面板搜 "yclaw-chat-xterm-host" → 命中 xterm host 元素
  - `rg -n "hermes-sidebar-plugin-nav-heading" web/src/App.tsx` 应为 0 命中

### Task 15: web 端测试断言全量同步
- **目标**：所有 web 端 test 文件里 brand 字面量断言改为 "YClaw",所有 preserved identifier 断言(storage key、env var、runtime global、下载文件名、skill id、CLI 命令、HTTP 头、docs URL、数据目录)保持原字面。
- **改动文件**：
  - 扫 `web/src/**/*.test.{ts,tsx}` 找 brand 字面
  - 已识别的可能点(继续 grep 兜底):
    - `web/src/i18n/languages.test.ts` / `web/src/i18n/context.test.tsx` / `web/src/i18n/runtime.test.ts` — 改 brand 断言(若有)
    - `web/src/lib/api.test.ts`(若有) — `X-Hermes-Session-Token` 是 HTTP 头,**保留**
    - `web/src/themes/*.test.ts` — 改 theme label 断言(若有 brand 字面)
  - **不要改**:`hermes-config.json`、`HERMES_*`、`__HERMES_*`、`hermes-index`、`__hermes_memory_builtin__`、`hermes.sidebar-collapsed` / `hermes-dashboard-theme` 类 storage key、`<code>hermes ...</code>` 命令字面、`hermes-agent.nousresearch.com`、`X-Hermes-Session-Token`、`hermes_session_at`
- **验证方式**：
  - `cd web && npm run test:ui` — vitest 全部通过
  - `cd web && npm run typecheck` — tsc 通过
  - `cd web && npm run lint` — eslint 通过

### Task 16: 跨面 audit + 总验证
- **目标**：在整个 `apps/desktop/` 和 `web/` 范围内做一次 grep 兜底,确保 (1) 没有 brand 字面遗漏在用户面字符串里,(2) 没有 preserved identifier 被误改。
- **改动文件**：
  - 无新增改动；这是一次 audit + 修复性改动
  - 如果 grep 发现遗漏,回头补到对应 Task(根据命中位置:en.ts → Task 3,ja.ts → Task 4,web i18n → Task 10/11,etc.)
- **验证方式**：
  - `rg -n "[\"'\`][^\"'\`]*\\b[Hh]ermes[^\"'\`]*[\"'\`]" apps/desktop/src web/src` — 在 desktop src + web src 下,所有 `Hermes` / `hermes` 字面量都应只在以下位置:
    - `localStorage` key 字面(以 `'hermes` 或 `"hermes` 开头)
    - `__HERMES_*` runtime global 名
    - `HERMES_*` env var 名
    - `hermes:connection` 等 IPC channel
    - `hermesDesktop.*` 桥接调用
    - `X-Hermes-Session-Token` HTTP 头
    - `hermes-config.json` 下载文件名
    - `hermes-index` skill id / `__hermes_memory_builtin__` 常量
    - `<code>hermes ...</code>` / `<code>~/.hermes/...</code>` / `<code>HERMES_*</code>` 命令和路径
    - 文件名/路径引用(`HERMES_DOCS_URL`、`hermes:dev-session-token` 之类 vite plugin 内部名)
    - 注释里"preserved identifier" 的说明文字
  - `rg -c "YClaw" apps/desktop/src web/src` — 两个 src 树都应有 YClaw 命中,且与 Tasks 1-15 的预期匹配
  - 桌面端全验证:
    - `cd apps/desktop && npm run typecheck`
    - `cd apps/desktop && npm run lint`
    - `cd apps/desktop && npm run test:ui`
    - `scripts/run_tests.sh apps/desktop/ -q`
  - web 端全验证:
    - `cd web && npm run typecheck`
    - `cd web && npm run lint`
    - `cd web && npm run test:ui`
  - 肉眼 end-to-end:
    - 启动 desktop dev(`cd apps/desktop && npm run dev`)
    - 启动 web dev(`cd web && npm run dev`)
    - 桌面端:窗口 title 显示 "YClaw"、About heading 显示 "YClaw Desktop"、新建会话看 intro personality 5 个 headline/body 全部没有 "Hermes"、思考中显示 "YClaw is thinking"、主题切换器 label 是 "YClaw Teal"
    - web 端:浏览器 tab 标题 "YClaw - Dashboard"、顶栏 brand "YClaw"、19 种语言切换 nav 区域都是 "YClaw"、theme picker label 是 "YClaw Teal"、Channels → Telegram onboarding 默认 bot_name 是 "YClaw Agent"、System → Update 对话框标题 "Update YClaw?"、Matrix provider config placeholder 是 `@yclaw:example.org`、devtools console 看到 `[yclaw-chat]` 日志、Elements 面板 aria-labelledby 链断检查通过
    - **绝对不变**:OS 协议 `hermes://`、CLI 命令 `hermes`、数据目录 `~/.hermes/`、storage key、IPC channel、HTTP 头、runtime global、env var、内嵌 `<code>hermes ...</code>` 命令字面、`hermes-config.json`、`hermes-index`、`hermes-agent.nousresearch.com`

---

## 风险回退预案

- **如果某个 i18n 语种 grep 漏改**:回到 Task 4 / Task 11,把 `en.ts` 替换的清单用同一份 grep 结果遍历其它语种。
- **如果某个测试断言改错(改了 preserved identifier)**:从 git diff 反查,对照 Task 8 / Task 15 的"不要改"清单,撤回该断言。
- **如果 dev 启动后还有肉眼可见 "Hermes" 字符串**:用浏览器 devtools / VSCode search 定位文件,按文件路径回溯到对应 Task。
- **如果 `apps/desktop` 的 Vite dev 报 stale cache**:杀掉所有 `node` 进程,删 `apps/desktop/.vite`(如有)与 `node_modules/.vite` 缓存,重新 `npm run dev`。

## 下一步

调用 `/openflow build` 进入实现阶段。build 阶段会:
1. 用 Superpowers 流程按本 plan-ready.md 的 16 个 Task 顺序执行
2. 每完成一个 Task 在 `tasks.md` 里勾选对应 checkbox(用 `openspec` 跟踪)
3. 完成后用 `/openflow close` 验证一致性 + 归档
