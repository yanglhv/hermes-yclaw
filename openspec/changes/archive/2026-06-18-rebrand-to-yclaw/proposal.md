# Rebrand to YClaw — Proposal

## 1. 目标

把所有用户可见的 "Hermes" 品牌文本与品牌图形改为 "YClaw",先在 `apps/desktop/`(Electron 桌面端)与 `web/`(Dashboard,实际位于仓库根 `web/` 而非 `apps/web/`)两个用户面上落地;**保持后端标识、数据、协议、IPC 频道、环境变量、CLI 子命令、Python 模块名、electron-builder `appId`、安装包元数据、磁盘数据目录全部不变**,以避免跨版本兼容断裂。

## 2. 范围

### 2.1 包含(用户可见的显示层)

- 应用窗口标题、`<title>` 标签、About 对话框
- 启动闪屏 / 启动横幅 / 启动占位文案
- 4 个(桌面)/ 19 个(网页)语言文件里所有用户面 "Hermes" 字样
- 主题/皮肤系统里的品牌字段("Hermes Teal" 主题名/描述)
- BrandMark 组件渲染的 `nous-girl` 字面与 alt 文本
- 文档站点的内嵌标题(`<title>`、`<h1>`)、`DocsPage` 里的 logo 块
- 引导(Onboarding)、首启、About 页面文本
- 介绍语 JSONL(`intro-copy.jsonl`)中各 personality 的 headline/body
- 通知 toast 标题/正文
- App 标题栏的 "Hermes" 字样(若 BrandMark 之外另有文字)
- 主题市场(VSCode Marketplace)展示出来的描述里的 "Hermes" 字样
- 站点名 `hermes-agent.nousresearch.com` —— **保留**(这是真实产品域名,不在本次改动范围)

### 2.2 不包含(后端标识,全部保持原名)

- `apps/desktop/package.json` 的 `name: "hermes"`、`productName: "Hermes"`
- `apps/desktop/package.json` 的 `appId: "com.nousresearch.hermes"`
- `electron-builder` 全部配置(`executableName`、`artifactName`、`extraResources` 路径、协议名 `hermes://`、`appId`)
- 协议处理 `hermes://` 协议名
- macOS plist 里的 `CFBundleDisplayName` / `CFBundleExecutable` / `CFBundleName`
- IPC 频道前缀 `hermes:*`(preload.cjs 暴露的 `window.hermesDesktop.*`)
- 环境变量 `HERMES_*`
- Python 包名 / 模块名(CLI 内部使用的 `hermes_cli.*`、Python `import hermes_*`)
- CLI 入口命令 `hermes`(以及子命令 `hermes dashboard`、`hermes tools`、`hermes update`、`hermes auth`、`hermes gateway` 等)
- 磁盘数据目录 `~/.hermes/` 与 `~/.hermes/profiles/*`
- `web/src/lib/api.ts` 的 `HERMES_BASE_PATH`、`X-Hermes-Session-Token`、内部事件名 `__HERMES_*`
- 主题存储 key / localStorage key:`hermes-desktop-theme-v2`、`hermes-sidebar-collapsed`、`hermes-locale`、`hermes-dashboard-theme` 等
- 数据库/会话存储里的 session profile 名
- 内部消息协议、JSON-RPC 方法名

## 3. 用户可见 "Hermes" 出现位置清单(仅显示层,所有项目)

### 3.1 `apps/desktop/`(Electron 桌面端)

| 类型 | 位置 | 当前内容 | 建议替换 |
|---|---|---|---|
| HTML `<title>` | `apps/desktop/index.html:11` | `<title>Hermes</title>` | `<title>YClaw</title>` |
| i18n 字符串(英) | `apps/desktop/src/i18n/en.ts:46,50,53,56,57,64,101,102,110,140,141,266,288,296,300,312,331,348,374,392,405,442,452,455,457,489,491,567,741,763,765,901,971,1058,1204,1205,1209,1257,1270,1348,1356,1362,1364,1370,1371,1375,1377,1378,1400,1409,1412,1414,1432,1434,1435,1443,1455,1468,1469,1480,1485,1573,1589,1661,1691,1692,1693,1695,1708,1710,1728,1755,1766,1771,1808,1812,1815,1863,1864,1883` | "Hermes Desktop is ready"、"Update Hermes"、"About Hermes Desktop" 等 | "YClaw Desktop is ready"、"Update YClaw"、"About YClaw Desktop" |
| i18n 字符串(日) | `apps/desktop/src/i18n/ja.ts:46,50,53,245,496,563,616,1539,1562` | "Hermes Desktop の準備ができました" 等 | "YClaw Desktop の準備ができました" 等 |
| i18n 字符串(简中) | `apps/desktop/src/i18n/zh.ts:573,640,684` | "Hermes Desktop"、"Hermes 后端" | "YClaw Desktop"、"YClaw 后端" |
| i18n 字符串(繁中) | `apps/desktop/src/i18n/zh-hant.ts:46,485,552,596` | "Hermes Desktop 已就緒" 等 | "YClaw Desktop 已就緒" 等 |
| BrandMark 品牌 | `apps/desktop/src/components/brand-mark.tsx` | 显示 "nous-girl" 标(实际是 Hermes 标志)| 暂时改 alt/aria-label 文案;logo 图本身**不动** |
| 标题栏 | `apps/desktop/src/app/shell/titlebar.ts` 及关联 `titlebar-controls.tsx` | 若显示 "Hermes" 字样 | 改为 "YClaw" |
| 启动横幅 | `apps/desktop/src/components/chat/intro.tsx` + `intro-copy.jsonl:27,37,53,57,71` | "hermes-chan is here! <3"、"Hermes at the helm, arrr"、"Hermes. Code investigator."、"hermes-san is wistening"、"Hermes Agent is ready." | 全部改为 "YClaw" 系列 |
| Thinking 占位 | `apps/desktop/src/components/assistant-ui/thread.tsx:413` | `'Hermes is thinking'` | `'YClaw is thinking'` |
| Loading response | `apps/desktop/src/components/assistant-ui/streaming.test.tsx:389,397` | `'Hermes is loading a response'` | `'YClaw is loading a response'`(测试断言) |
| 通知标题 | `apps/desktop/src/store/onboarding.ts:192` | `title: 'Hermes is ready'` | `title: 'YClaw is ready'` |
| Boot 错误文案 | `apps/desktop/src/components/gateway-connecting-overlay.test.tsx:63` | `'Hermes backend did not become ready'` | `'YClaw backend did not become ready'` |
| About 面板 | `apps/desktop/src/app/settings/about-settings.tsx` | "Hermes Desktop" | "YClaw Desktop" |
| Theme 名称 | `apps/desktop/src/themes/presets.ts:32` 注释;`apps/desktop/src/themes/use-skin-command.ts:10`(`hermes: 'nous'`) | "Hermes desktop identity" 注释、内部 skin 键 | 内部 skin 键(`hermes`)保持不动;注释文案改 |
| Theme 标签 | `apps/desktop/src/themes/presets.ts` 中如 `label: "Hermes Teal"` | 主题展示名 | "YClaw Teal"(仅展示名,内部 ID 不动) |

### 3.2 `web/`(Dashboard,仓库根)

| 类型 | 位置 | 当前内容 | 建议替换 |
|---|---|---|---|
| HTML `<title>` | `web/index.html:10` | `<title>Hermes Agent - Dashboard</title>` | `<title>YClaw - Dashboard</title>` |
| 顶栏 brand | `web/src/App.tsx:580` | `Hermes` | `YClaw` |
| i18n 字符串(英) | `web/src/i18n/en.ts:53,123,124,321,338,339,361,375,487,489,491,506,537,540,587` 及 i18n 全文 | "Hermes Agent"、"Update Hermes" 等 | "YClaw"、"Update YClaw" 等 |
| i18n 字符串(其余 18 种) | `web/src/i18n/{af,de,es,fr,ga,hu,it,ja,ko,pt,ru,tr,uk,zh,zh-hant}.ts:53,123,124,322~341,488~541,587` | 各语种 "Hermes Agent"、"Update Hermes" 等 | 全部对应替换 |
| Theme 名 | `web/src/themes/presets.ts:43,44,288,289` | `label: "Hermes Teal"`、`description: "Classic dark teal — the canonical Hermes look"` | `label: "YClaw Teal"`、`description: "Classic dark teal — the canonical YClaw look"` |
| Theme 注释 | `web/src/themes/presets.ts:11,188,201` | "Hermes teal" 注释 | 注释文案改 |
| Docs 站嵌入 | `web/src/pages/DocsPage.tsx`(嵌入的 docs iframe 来自 `https://hermes-agent.nousresearch.com/docs/`,**不在本次改动范围**) | — | — |
| 错误信息 | `web/src/lib/gatewayClient.ts:130` | "page must be served by the Hermes dashboard" | "page must be served by the YClaw dashboard" |
| 错误信息 | `web/src/pages/ChatPage.tsx:137` | "Open this page through `hermes dashboard`, not directly." | 文案 "Hermes dashboard" 部分改 "YClaw dashboard";`hermes dashboard` 命令字面**保留**(是 CLI 子命令) |
| 设置页面 | `web/src/pages/SystemPage.tsx:423,527,530,531,700,702,849,957,1193` | "Hermes updates are managed outside..."、"Update Hermes?"、"Hermes" 标签、`hermes portal` | "YClaw updates..."、"Update YClaw?"、标签改 "YClaw"、命令字面保留 |
| 配置/环境页 | `web/src/pages/ConfigPage.tsx:339`、`web/src/pages/EnvPage.tsx:52,758` | "hermes-config.json" 下载文件名、`HERMES_QWEN_` 前缀、`~/.hermes/.env` | 下载文件名**保留**(`hermes-config.json` 是配置文件名,改了会找不到);`HERMES_QWEN_` 是环境变量前缀**保留**;`~/.hermes/.env` 路径**保留** |
| 渠道页 | `web/src/pages/ChannelsPage.tsx:256,265,565` | `<code>hermes gateway start</code>`、`~/.hermes/.env`、`bot_name: "Hermes Agent"` | 命令字面**保留**;`bot_name` 是用户配置的 bot 显示名——这是产品显示字符串,改 "YClaw Agent" |
| 技能页 | `web/src/pages/SkillsPage.tsx:1083,1095,1105` | `hermes skills search` 命令、skill id `hermes-index` | 命令字面**保留**;`hermes-index` 是内置 skill id,**保留** |
| 侧栏插件 | `web/src/App.tsx:634,644` | `aria-labelledby="hermes-sidebar-plugin-nav-heading"`、`id="hermes-sidebar-plugin-nav-heading"` | 这些是 `id`/`aria-labelledby` 字符串,与样式不挂钩但出现在 DOM 上,改 "yclaw-sidebar-plugin-nav-heading" 同步两处 |
| 聊天页 CSS 类 | `web/src/pages/ChatPage.tsx:473,895` | `[hermes-chat] ...`、`className="hermes-chat-xterm-host ..."` | 这俩是 dev 日志和 xterm 容器 className;改 "yclaw-chat-xterm-host" 等;`data-` 属性改 "yclaw-chat" |

### 3.3 图片资源(logo 图形)

- `apps/desktop/assets/` — `icon.icns` / `icon.ico` / `icon.png`(**全部保留**)
- `apps/desktop/public/` — `hermes-frames/`、`hermes-sprite.png`、`hermes.png`、`nous-girl.jpg`(**全部保留**;本次不改图形资源)
- `web/public/favicon.ico`(**保留**)

**理由**:本次选择"先只做字"——logo 图形(`BrandMark` 内部用的图)统一保留原图,只把 alt/aria 文本和文字标签换掉。

## 4. 验收条件

1. `apps/desktop` 和 `web` 编译/类型检查通过(无 `tsc` 报错)
2. 桌面端启动后,窗口标题、About、启动横幅、首启引导、设置面板中所有面向用户的 "Hermes" 文本均变为 "YClaw"
3. Dashboard 启动后,顶栏 brand、About、所有 i18n 字符串、所有 Theme 展示名均显示 "YClaw"
4. 后端侧全部保持原名:CLI 仍是 `hermes`、数据目录仍是 `~/.hermes/`、IPC 频道仍是 `hermes:*`、Python 包名不变、磁盘数据不迁移
5. 现有桌面端的所有 `*.test.ts` / `*.test.tsx` 中关于 "Hermes" 的字符串断言同步更新,且测试通过
6. 现有 web 端的所有 `*.test.ts` / `*.test.tsx` 中关于 "Hermes" 的字符串断言同步更新,且测试通过
7. 修改后 4 个桌面 i18n 全部字符串 key/语言文件同步改动(en/ja/zh/zh-hant)
8. 修改后 19 个 web i18n 全部字符串 key/语言文件同步改动

## 5. 风险

| 风险 | 缓解 |
|---|---|
| 测试断言里的 "Hermes" 字面被遗漏,导致 `npm run test` 失败 | 改完字符串后,跑 `scripts/run_tests.sh apps/desktop/` 和 `web` 端测试,补齐所有断言 |
| Theme 展示名改了但用户已有的 `localStorage` 缓存键未刷新 | Theme 内部 `id` 不动,仅改 `label`/`description`;缓存键沿用 `id` 不受影响 |
| `aria-labelledby` / `id` 改字符串后无障碍文本链断 | 在 `web/src/App.tsx:634,644` 同步两处,不改语义 |
| 桌面端 `aria-label` 里写死 "Hermes" | 在 `desktop-controller.tsx` 与 shell 组件里 grep 兜底 |
| i18n 部分语种漏改 | 用 `rg "Hermes" web/src/i18n` 兜底,逐个文件改完后再 grep 一次 |
| `BrandMark` 改了 alt 但图本身没换,视觉上仍是 Hermes 标志 | 用户已选"先只做字",接受此状态;后续单独立项换图 |
| 桌面端 `intro-copy.jsonl` 改动后 personality 顺序/字段未变 | 仅改 headline/body 文案,不改 JSONL schema |

## 6. 不在范围 / 后续

- 替换 `BrandMark` 实际 logo 图(需用户提供新图)
- 替换 `assets/icon.*` 应用图标(影响 electron-builder 打包)
- 替换 `web/public/favicon.ico`
- 改 `package.json` 的 `name`/`productName`/`appId`、协议名 `hermes://`、IPC 频道前缀
- 改 CLI 子命令、数据目录、环境变量
- 改后端 Python 模块名

## 7. 下一步

调用 `/openflow spec` 进入规格生成阶段,把上述"清单"翻译成可执行的需求规格 + 实现计划。
