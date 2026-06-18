# Rebrand to YClaw — Design

## Context

The desktop (`apps/desktop/`) and dashboard (`web/`) ship visible
"Hermes" branding across window titles, About panels, i18n strings,
intro personalities, theme preset labels, and a handful of error
messages. The rebrand is a display-layer-only change: the goal is to
make "YClaw" the visible brand without touching any backend
identifier, IPC channel, environment variable, data directory, or CLI
command — so that the existing `~/.hermes/` install base, OAuth
sessions, dashboard cookies, and the in-tree JSON-RPC protocol all
keep working unchanged.

Two pieces of the visible surface have non-obvious carry-over
behavior and need explicit decisions:

- **Storage keys** like `hermes-desktop-theme-v2` and
  `hermes-dashboard-theme` are localStorage keys, not user-facing
  brand strings. Renaming them would invalidate existing user
  state without giving any visible win.
- **CLI commands** like `hermes update`, `hermes gateway start`,
  `hermes skills search` appear inside `<code>`-formatted
  substrings throughout the web UI. Those substrings are real
  commands the user can copy-paste; replacing the brand inside
  them would point users at a command that does not exist.

## Goals / Non-Goals

**Goals:**

- Replace every user-visible "Hermes" string with "YClaw" in the
  two surfaces the user is asking about (`apps/desktop/` and
  `web/`).
- Keep the four desktop locales (`en` / `ja` / `zh` / `zh-hant`) and
  the 19 web locales in lock-step — no language drifts out of sync.
- Update every test assertion that names the brand so the desktop
  and web test suites still pass.
- Make no data migration: the user keeps their `~/.hermes/`
  directory, OAuth sessions, themes, and pinned sidebar items.
- Make no protocol or IPC change: desktop ↔ backend and dashboard ↔
  backend continue to speak the same JSON-RPC and WebSocket.

**Non-Goals:**

- Replacing any of the `assets/icon.*` files or the `BrandMark`
  logo image. The user explicitly chose "先只做字" (text-only for
  this PR); a logo swap is a separate change.
- Renaming `package.json` `name` / `productName` / `appId`, the
  `hermes://` URL protocol, the IPC `hermes:*` channel prefix, the
  `HERMES_*` environment variables, the Python module names, the
  CLI command names (`hermes`, `hermes desktop`, etc.), the
  `~/.hermes/` data directory, the `X-Hermes-Session-Token` HTTP
  header, the `__HERMES_*` runtime globals, or the
  `hermes-agent.nousresearch.com` docs URL.
- Migrating the `hermes-config.json` download filename, the
  `hermes-index` skill ID, the `MEMORY_PROVIDER_BUILTIN` constant,
  or any `localStorage` key.
- Touching the `ui-tui/` TUI, the Python CLI banner, the gateway
  messaging platform names, or the website (`website/`) — all are
  out of scope per the user's selection of "桌面端 + web/Dashboard".

## Decisions

### Decision 1: Two capabilities, one spec file

The change set lives under a single OpenSpec capability
`user-visible-branding` with all 12 ADDED Requirements bundled into
`specs/user-visible-branding/spec.md`. Splitting the spec into
"desktop" / "web" sub-capabilities would force cross-file
synchronization rules that OpenSpec cannot express; bundling the
brand surfaces that must move together (window title + i18n +
test assertions) inside one capability keeps the drift-check
scenario simple.

### Decision 2: "YClaw" as the canonical brand token

The replacement token is the four-character mixed-case string
`YClaw` (Y, C uppercase; l, a, w lowercase). The existing
intro-copy.jsonl uses both styles — `hermes-chan` (lowercase) and
`Hermes at the helm` (uppercase) — and the new copy keeps the
"kawaii" lowercase flavor where appropriate (`yclaw-chan`) while
presenting the brand in title case in user-facing prose
(`YClaw Desktop`, `Update YClaw`, `YClaw is ready`). The spec
requires the headline / body strings to use the `YClaw` token but
does not constrain the surrounding casing beyond that.

### Decision 3: Storage keys, IPC channels, env vars, and CLI commands stay verbatim

This is a non-negotiable design constraint. The "Hermes" token in
the following positions is a load-bearing identifier, not a brand
display:

- `localStorage` keys (`hermes-desktop-theme-v2`,
  `hermes-dashboard-theme`, `hermes-sidebar-collapsed`,
  `hermes-locale`, `hermes-boot-background`, etc.)
- IPC channel names (`hermes:connection`,
  `hermes:backend:touch`, `hermes:window:openSession`,
  `hermes:gateway:ws-url`, etc.)
- Environment variables (`HERMES_HOME`, `HERMES_DASHBOARD_URL`,
  `HERMES_QWEN_`)
- Python module / CLI command names
- Data directory `~/.hermes/` and its subpaths
- HTTP header `X-Hermes-Session-Token`
- Runtime globals `__HERMES_*`
- Skill IDs (`hermes-index`) and the
  `__hermes_memory_builtin__` constant
- The download filename `hermes-config.json`
- The `HERMES_BASE_PATH` URL prefix
- The docs URL `https://hermes-agent.nousresearch.com/docs/`

Renaming any of these would either invalidate user state (storage
keys) or break the cross-process protocol (IPC channels, env vars,
CLI commands, Python modules, HTTP header). The user-facing cost
of leaving them unchanged is zero: users do not see these strings
on screen. The user-facing cost of renaming them is high: existing
themes break, OAuth sessions drop, the updater cannot find the
`hermes update` command, the dashboard cannot authenticate, and
the docs URL 404s.

### Decision 4: Inline CLI command references stay verbatim

In the web dashboard, phrases like
`<code>hermes gateway start</code>`,
`<code>hermes update</code>`, `<code>hermes skills search</code>`,
`<code>hermes memory setup</code>`, `<code>hermes portal</code>`
are real commands the user is expected to copy-paste. The visible
prose around them may be rebranded, but the `<code>...</code>`
content stays byte-for-byte `hermes ...`.

### Decision 5: The Telegram bot `bot_name` field is rebranded

`web/src/pages/ChannelsPage.tsx:565` sends
`api.startTelegramOnboarding({ bot_name: "Hermes Agent" })`. The
"bot_name" is the display name shown to users when they
onboard Telegram, so it is a user-facing string and gets
rebranded to `"YClaw Agent"`. This is the single Telegram
identifier that gets renamed; the underlying BotFather account
URL (`https://t.me/<bot_username>`) and the underlying bot
username are user-supplied and untouched.

### Decision 6: The Matrix example user ID is rebranded

`apps/desktop/src/i18n/en.ts:901` (and the other three locales)
hold a placeholder `@hermes:example.org` in the
`MATRIX_USER_ID` field. This is an example value shown to the
user in the input placeholder, so it is a user-facing string and
gets rebranded to `@yclaw:example.org`. This change is purely
visual; the actual Matrix protocol identifiers (server
`example.org`, the `@` user prefix syntax) are untouched.

### Decision 7: The aria / id / className / log-prefix sweep

A handful of non-brand identifiers happen to contain the
"hermes" token and are visible in devtools or in the DOM:
`aria-labelledby="hermes-sidebar-plugin-nav-heading"`,
`className="hermes-chat-xterm-host"`,
`console.log("[hermes-chat] ...")`. The user is not looking at
these, but they are part of the rebrand sweep because they
contradict the visible brand. They get rebranded in lock-step
with the user-visible strings.

### Decision 8: Theme preset IDs are preserved

`web/src/themes/presets.ts:43` has `label: "Hermes Teal"` and a
matching `id` (e.g. `default`, `nous-blue`, etc.). The spec
rebrands the `label` and `description` to use the "YClaw" token
but does NOT change the `id`. The `id` is the storage key
referenced by `hermes-dashboard-theme`; renaming it would
invalidate the user's existing theme selection.

## Risks / Trade-offs

- **Test drift after rebrand.** The spec mandates updating test
  assertions that name the brand, but it is impossible to
  enumerate every assertion in advance. The build step
  `scripts/run_tests.sh apps/desktop/ -q` and `npm run test` in
  `web/` are the regression net; any failing assertion is a
  scope leak and must be patched in the same PR.

- **Locale drift between desktop and web.** The desktop has 4
  locales, the web has 19. The rebrand touches both, and a
  partial pass would leave, e.g., `en.ts` rebranded but
  `de.ts` still showing "Hermes". The spec's
  "all-locales-in-sync" scenario asserts the brand-token count
  matches across all files in each surface.

- **Logo image is left as-is.** The user accepted this in
  proposal: the visible BrandMark image, the `assets/icon.*`
  files, and `web/public/favicon.ico` are still the old
  Hermes-style artwork. A future PR should swap the artwork; a
  follow-up ticket should track it. The current PR does not
  include the artwork change because the user picked the
  text-only option.

- **Docs URL is left as-is.** The `<DocsPage>` iframe points at
  `https://hermes-agent.nousresearch.com/docs/`, which is the
  real production docs site. Renaming the URL would require
  DNS and backend coordination that is outside this PR's
  scope. The iframe title and the `HERMES_DOCS_URL` constant
  are left untouched.

- **Hot-reload of the i18n bundle.** The Vite dev server picks up
  the i18n changes on save; the production build will be
  triggered by the regular `npm run build` step in
  `apps/desktop/` and `web/`. There is no migration step.

- **Telegram bot name in production.** Existing Telegram bots
  already created with the "Hermes Agent" name will not be
  renamed by this PR — Telegram disallows arbitrary renaming
  of bot names without a new BotFather registration. The
  default value passed to `startTelegramOnboarding` is renamed
  to "YClaw Agent" so *new* onboardings get the new name; users
  with existing bots keep the name they registered. The spec
  acknowledges this implicitly by listing the change in the
  "User-visible brand" section rather than the "preserve
  identifiers" section.
