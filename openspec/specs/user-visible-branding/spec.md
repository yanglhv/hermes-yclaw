# user-visible-branding Specification

## Purpose
TBD - created by archiving change rebrand-to-yclaw. Update Purpose after archive.
## Requirements
### Requirement: Desktop app shows the "YClaw" brand in window title and About panel

The system SHALL render the desktop window `<title>` and the About panel
with the "YClaw" brand string, replacing the existing "Hermes" literal in
those locations. Window title is set in the renderer entry HTML; the
About panel content is rendered by the settings/About component and
driven by i18n strings.

#### Scenario: Browser window title shows "YClaw" on first paint

- **WHEN** the desktop window loads the renderer
- **THEN** `apps/desktop/index.html` `<title>` element reads `YClaw`
- **AND** the OS-level window title (sourced from the same `<title>`)
  displays `YClaw` in the title bar

#### Scenario: About panel header reads "YClaw Desktop"

- **WHEN** the user opens Settings → About
- **THEN** the panel heading reads "YClaw Desktop" in every supported
  locale (en, ja, zh, zh-hant)
- **AND** the version line reads "YClaw Desktop v<x.y.z>"

### Requirement: Desktop i18n strings replace "Hermes" with "YClaw" across all four locales

The system SHALL replace every user-facing literal containing the brand
"Hermes" / "Hermes Desktop" / "Hermes Agent" / "Hermes Gateway" with
the corresponding "YClaw" / "YClaw Desktop" / "YClaw Agent" / "YClaw
Gateway" string in `apps/desktop/src/i18n/{en,ja,zh,zh-hant}.ts`. The
replacement must be in sync across all four locale files: any string
that mentions "Hermes" in `en.ts` must have a "YClaw" counterpart in
`ja.ts` / `zh.ts` / `zh-hant.ts`, and vice versa.

#### Scenario: All four locale files updated with no drift

- **WHEN** the rebrand task completes
- **THEN** the count of brand-token replacements in `en.ts` equals the
  count in `ja.ts`, `zh.ts`, and `zh-hant.ts` (±0)
- **AND** none of the four files contains the literal "Hermes" in any
  user-facing string value

#### Scenario: Designer / developer surface strings still contain "Hermes"

- **WHEN** the task is applied
- **THEN** non-user-facing occurrences in i18n (type names, function
  names, comment-only references) MAY still contain "Hermes" — only
  string values (right-hand side of `:` in object literals) are
  replaced

### Requirement: Desktop intro copy uses "YClaw" personality headlines

The system SHALL replace the brand name in every personality's
`headline` and `body` field of
`apps/desktop/src/components/chat/intro-copy.jsonl`. The five
personalities (`kawaii`, `pirate`, `noir`, `uwu`, `none`) each ship a
distinct headline and body; all five must be updated.

#### Scenario: All five personality lines are rebranded

- **WHEN** the renderer reads `intro-copy.jsonl`
- **THEN** every `headline` value contains the "YClaw" token and no
  `headline` retains "Hermes" / "hermes"
- **AND** the JSONL shape (5 records, fields `personality`, `headline`,
  `body`) is unchanged

### Requirement: Desktop thinking / loading / notification strings use "YClaw"

The system SHALL replace the brand "Hermes" with "YClaw" in:

- The "thinking" label rendered in
  `apps/desktop/src/components/assistant-ui/thread.tsx`
- The "loading a response" status label rendered via
  `@assistant-ui/react-streamdown` (asserted in
  `streaming.test.tsx`)
- The "is ready" notification title dispatched from
  `apps/desktop/src/store/onboarding.ts`
- The "backend did not become ready" boot-error text asserted in
  `gateway-connecting-overlay.test.tsx`

#### Scenario: Thinking / loading / notification text reflects new brand

- **WHEN** the agent is mid-turn
- **THEN** the thinking label reads "YClaw is thinking"
- **AND** the loading response status reads "YClaw is loading a response"
- **WHEN** onboarding finishes successfully
- **THEN** the success notification title reads "YClaw is ready"
- **WHEN** the backend fails to come up
- **THEN** the boot error text reads "YClaw backend did not become ready"

### Requirement: Desktop theme preset labels use the new brand while keeping internal IDs stable

The system SHALL update the user-facing theme preset labels in
`apps/desktop/src/themes/presets.ts` (and any presets surfaced through
the skin command pipeline) from "Hermes Teal" / "Hermes Desktop" to
"YClaw Teal" / "YClaw Desktop", but SHALL keep all internal theme IDs,
`localStorage` keys, and skin-command keys (`hermes: 'nous'` in
`use-skin-command.ts`) unchanged. The rebrand affects display strings
only.

#### Scenario: Theme list shows new brand labels with persistent storage

- **WHEN** the user opens the theme picker
- **THEN** the displayed labels read "YClaw Teal" (and any related
  variants) instead of "Hermes Teal"
- **WHEN** the user reopens the app after switching themes
- **THEN** the previously-selected theme resolves to the same
  internal ID and is still selected — no migration needed

### Requirement: Desktop tests assert against the "YClaw" brand

The system SHALL update every desktop test that asserts on a brand
literal (`'Hermes is thinking'`, `'Hermes is loading a response'`,
`'Hermes is ready'`, `'Hermes backend did not become ready'`, and the
storage-key / onboarding-flow assertions) to assert against the
"YClaw" equivalent. The storage key literals (`hermes-desktop-onboarded-v1`,
`hermes-onboarding-skipped-v1`, etc.) remain unchanged because they are
not user-facing brand strings.

#### Scenario: `npm run test:ui` (vitest) passes after rebrand

- **WHEN** `scripts/run_tests.sh apps/desktop/ -q` runs
- **THEN** every previously-passing test still passes
- **AND** every brand-literal assertion in the test files reflects
  "YClaw" rather than "Hermes"

### Requirement: Web dashboard `<title>` and top-bar brand show "YClaw"

The system SHALL update `web/index.html` `<title>` from "Hermes Agent -
Dashboard" to "YClaw - Dashboard", and SHALL update the top-bar brand
label rendered in `web/src/App.tsx:580` from "Hermes" to "YClaw".

#### Scenario: Browser tab and top-bar display the new brand

- **WHEN** the dashboard renders
- **THEN** the document title reads "YClaw - Dashboard"
- **AND** the top-bar brand label renders as "YClaw"

### Requirement: Web i18n strings replace "Hermes" with "YClaw" across all 19 locales

The system SHALL replace every user-facing literal containing the
brand "Hermes" / "Hermes Agent" / "Hermes Gateway" with the
corresponding "YClaw" / "YClaw Agent" / "YClaw Gateway" string in
`web/src/i18n/{af,de,en,es,fr,ga,hu,it,ja,ko,pt,ru,tr,uk,zh,zh-hant}.ts`.
The replacement must be in sync across all 19 locale files.

#### Scenario: All 19 locale files updated with no drift

- **WHEN** the rebrand task completes
- **THEN** every locale file's brand-token count is the same as the
  English baseline (±0)
- **AND** no locale file's string values contain the "Hermes" brand
  literal
- **AND** non-string-value occurrences (type names, function names,
  comments) MAY still reference "Hermes" without affecting this
  check

### Requirement: Web theme preset labels and store keys reflect the new brand

The system SHALL update the user-facing theme labels in
`web/src/themes/presets.ts` from "Hermes Teal" / "Hermes Teal (Large)"
/ "the canonical Hermes look" to "YClaw Teal" / "YClaw Teal (Large)" /
"the canonical YClaw look". The system SHALL keep all `localStorage`
keys (`hermes-dashboard-theme`, `hermes-dashboard-font`,
`hermes-theme-custom-css`, etc.) unchanged, because they are storage
identifiers and renaming them would invalidate existing user state.

#### Scenario: Theme picker labels rebrand; user state is preserved

- **WHEN** the user opens the theme picker
- **THEN** the labels read "YClaw Teal" / "YClaw Teal (Large)"
- **WHEN** the user reopens the dashboard after rebrand
- **THEN** the previously-selected theme resolves to the same internal
  ID and is still selected — no `localStorage` migration needed

### Requirement: Web error messages and inline CLI hints use the new brand

The system SHALL replace the brand literal in user-visible error
messages and inline help text in:

- `web/src/lib/gatewayClient.ts:130` — "page must be served by the
  Hermes dashboard" → "page must be served by the YClaw dashboard"
- `web/src/pages/ChatPage.tsx:137` — the visible sentence "Open this
  page through `hermes dashboard`, not directly." keeps the CLI
  command `hermes dashboard` as a code-formatted substring (a real
  command) and replaces the surrounding English text "the Hermes
  dashboard" → "the YClaw dashboard"
- `web/src/pages/SystemPage.tsx:423,527,530,531` — "Hermes updates
  are managed outside this dashboard." and the "Update Hermes?"
  dialog title and body — replaced to "YClaw updates..." / "Update
  YClaw?"; the inline `'hermes update'` command references remain
  as code-formatted substrings

The system SHALL NOT replace the brand inside code-formatted
substrings (e.g. `<code>hermes gateway start</code>`,
`<code>hermes update</code>`, `<code>hermes skills search</code>`,
`<code>hermes memory setup</code>`, `<code>hermes portal</code>`,
`<code>hermes tools</code>`) because those are real CLI commands.
It SHALL also not change skill IDs (`hermes-index`),
download-filename defaults (`hermes-config.json`),
`HERMES_*` env-var names, the `~/.hermes/` directory references, the
`X-Hermes-Session-Token` HTTP header, the `__HERMES_*` runtime
globals, or the `hermes-agent.nousresearch.com` docs URL.

#### Scenario: User-visible messages reflect new brand; CLI commands remain intact

- **WHEN** the user reads an error / system / update page
- **THEN** every English prose token that was "Hermes" is now "YClaw"
- **AND** every code-formatted CLI command (`hermes ...`) is preserved
  byte-for-byte

### Requirement: Web accessibility `id` / `aria-labelledby` strings track the new brand

The system SHALL update the two coupled strings in
`web/src/App.tsx:634,644` — the `aria-labelledby` on the plugin-nav
region and the matching `id` on its heading — from the
`hermes-sidebar-plugin-nav-heading` token to the
`yclaw-sidebar-plugin-nav-heading` token. Both strings must be
updated together so the `aria-labelledby → id` reference remains
valid.

#### Scenario: Sidebar plugin nav aria reference resolves to a real element

- **WHEN** the sidebar plugin nav region renders
- **THEN** the `aria-labelledby` attribute value equals the heading
  element's `id` attribute value
- **AND** the value contains the "yclaw" token and not the "hermes"
  token

### Requirement: Web dev / debug log prefixes use the new brand

The system SHALL update the dev-mode log prefix in
`web/src/pages/ChatPage.tsx:473` from `[hermes-chat] ...` to
`[yclaw-chat] ...`. The system SHALL also update the xterm host
className in `web/src/pages/ChatPage.tsx:895` from
`hermes-chat-xterm-host` to `yclaw-chat-xterm-host`. Both are
internal identifiers, not user-facing brand strings, but they
appear in browser devtools and are part of the rebrand sweep.

#### Scenario: Devtools logs and DOM class names reflect the new brand

- **WHEN** a developer opens devtools
- **THEN** the chat page's WebGL fallback log reads `[yclaw-chat] ...`
- **AND** the xterm host element carries the
  `yclaw-chat-xterm-host` className

### Requirement: Web tests assert against the "YClaw" brand

The system SHALL update every web test that asserts on a brand
literal to assert against the "YClaw" equivalent. Non-brand
identifiers (storage keys, env-var names, runtime globals, download
filenames, skill IDs, CLI command strings) keep their existing
"Hermes" tokens.

#### Scenario: Web vitest suite passes after rebrand

- **WHEN** `npm run test` (or `npm run test:ui`) runs in `web/`
- **THEN** every previously-passing test still passes
- **AND** every brand-literal assertion in the test files reflects
  "YClaw" rather than "Hermes"

### Requirement: Backend identifiers, storage keys, IPC channels, and data directories are preserved

The system SHALL NOT change any of the following, because each is a
load-bearing identifier that, if renamed, would break cross-version
compatibility, dashboard ↔ backend auth, or existing user data:

- `apps/desktop/package.json` `name` (`hermes`), `productName`
  (`Hermes`), `appId` (`com.nousresearch.hermes`), `executableName`
  (`Hermes`)
- `apps/desktop/electron-builder` config:
  - protocol scheme `hermes://`
  - `appId: com.nousresearch.hermes`
  - macOS `CFBundleDisplayName` / `CFBundleExecutable` /
    `CFBundleName` = `Hermes`
  - artifact naming `Hermes-${version}-${os}-${arch}.${ext}`
- IPC channel prefix `hermes:*` (e.g. `hermes:connection`,
  `hermes:backend:touch`, `hermes:window:openSession`, etc.)
- All `localStorage` keys containing the `hermes` token
  (`hermes-boot-background`, `hermes-desktop-theme-v2`,
  `hermes-sidebar-collapsed`, `hermes-locale`,
  `hermes-dashboard-theme`, `hermes-desktop-user-themes-v1`, etc.)
- Environment variable names `HERMES_*` (e.g. `HERMES_HOME`,
  `HERMES_DASHBOARD_URL`, `HERMES_QWEN_`)
- Python module / package names (`hermes_cli.*`, `import hermes_*`)
- CLI command names (`hermes`, `hermes desktop`, `hermes dashboard`,
  `hermes tools`, `hermes update`, `hermes auth`, `hermes gateway`,
  `hermes portal`, `hermes memory setup`, `hermes skills search`)
- Data directory `~/.hermes/` and all subpaths
  (`~/.hermes/profiles/*`, `~/.hermes/.env`,
  `~/.hermes/config.yaml`, `~/.hermes/skills/`,
  `~/.hermes/dashboard-themes/`, `~/.hermes/plugins/`)
- HTTP header `X-Hermes-Session-Token`
- Web runtime globals `__HERMES_SESSION_TOKEN__`,
  `__HERMES_AUTH_REQUIRED__`, `__HERMES_BASE_PATH__`,
  `__HERMES_PLUGIN_SDK__`, `__HERMES_PLUGINS__`,
  `__HERMES_DASHBOARD_EMBEDDED_CHAT__`
- The `HERMES_BASE_PATH` URL prefix in the dashboard
- The `hermes-config.json` download filename in
  `web/src/pages/ConfigPage.tsx`
- Skill IDs `hermes-index` and the constant
  `MEMORY_PROVIDER_BUILTIN = "__hermes_memory_builtin__"`
- The docs URL `https://hermes-agent.nousresearch.com/docs/`
- Bot display name `bot_name: "Hermes Agent"` in
  `web/src/pages/ChannelsPage.tsx:565` — **EXCEPTION**: this is
  user-facing and SHOULD be rebranded to `bot_name: "YClaw Agent"`
  to match the visible brand
- Telegram bot name placeholders in the desktop i18n files
  (`@hermes:example.org`) — **EXCEPTION**: these are example
  values shown to the user, so they SHOULD be rebranded to
  `@yclaw:example.org` in the desktop i18n `MATRIX_USER_ID`
  placeholder

#### Scenario: No backend identifier or storage key is renamed

- **WHEN** the rebrand task completes
- **THEN** a recursive `rg -l "hermes" apps/desktop/{electron,src}/` +
  `rg -l "hermes" web/src/` audit produces a list of files that
  contains ONLY:
  - The `hermes*` storage-key / IPC-channel / env-var / runtime
    globals (must keep)
  - The new i18n comment annotations that mention "Hermes" in a
    deprecation note (allowed)
  - Test fixtures and unit-test names that exercise the unchanged
    identifier surface
- **AND** every other occurrence in the user-visible surface is
  either "YClaw" or part of an explicitly-preserved identifier
  per the list above

#### Scenario: Telegram bot display name is rebranded in the visible Channels page

- **WHEN** the user opens Channels → Telegram onboarding
- **THEN** the default `bot_name` field reads `YClaw Agent`
- **AND** the visible channel page heading reflects the new bot
  name in subsequent onboarding steps

#### Scenario: Matrix example user ID is rebranded in desktop i18n placeholder

- **WHEN** the user opens the Matrix provider configuration in
  the desktop settings
- **THEN** the `MATRIX_USER_ID` placeholder reads
  `@yclaw:example.org` in all four desktop locales

