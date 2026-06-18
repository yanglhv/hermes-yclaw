# Rebrand to YClaw — Tasks

## 1. Desktop window title and About panel

- [x] 1.1 Update `apps/desktop/index.html:11` `<title>` from
      `Hermes` to `YClaw`
- [x] 1.2 Update `apps/desktop/src/app/settings/about-settings.tsx`
      and the `i18n/types.ts` / `en.ts` / `ja.ts` / `zh.ts` /
      `zh-hant.ts` strings that drive the About panel heading
      and version line ("Hermes Desktop" → "YClaw Desktop",
      "About Hermes Desktop" → "About YClaw Desktop")
- [x] 1.3 Verify: launch desktop, check window title and About
      panel reads "YClaw" / "YClaw Desktop" in all four locales

## 2. Desktop i18n rebrand (4 locales in lock-step)

- [x] 2.1 Sweep `apps/desktop/src/i18n/en.ts` for every brand
      literal ("Hermes", "Hermes Desktop", "Hermes Agent",
      "Hermes Gateway", "Hermes couldn't", "Hermes is",
      "Hermes will", "Hermes can", "Hermes cannot", "Hermes
      needs", "Hermes is restarting", "Restart Hermes",
      "Restarting Hermes", "Update Hermes", "Loading Hermes",
      "About Hermes", "Starting Hermes", "exit hermes",
      "hermes update", "hermes gateway", "hermes model",
      "hermes tools", "hermes desktop", "hermes portal",
      "hermes memory setup", "hermes skills search") in
      string values only; replace with the "YClaw" / "yclaw"
      equivalent while preserving tone, punctuation, and
      trailing period
- [x] 2.2 Mirror the same replacements in
      `apps/desktop/src/i18n/ja.ts`,
      `apps/desktop/src/i18n/zh.ts`, and
      `apps/desktop/src/i18n/zh-hant.ts`. The brand-token
      replacement count must match `en.ts` exactly
- [x] 2.3 In `en.ts:901` (`MATRIX_USER_ID` placeholder) and its
      three locale mirrors, change `@hermes:example.org` →
      `@yclaw:example.org`
- [x] 2.4 Verify: run `rg "Hermes" apps/desktop/src/i18n/*.ts`
      and confirm no brand-literal hits remain in string
      values (left-hand-side keys, type names, function
      references are allowed to keep the token)

## 3. Desktop intro copy (`intro-copy.jsonl`)

- [x] 3.1 Update the 5 personality records in
      `apps/desktop/src/components/chat/intro-copy.jsonl`:
      `kawaii` headline `hermes-chan is here! <3` →
      `yclaw-chan is here! :3`; `pirate` headline `Hermes at
      the helm, arrr` → `YClaw at the helm, arrr`; `noir`
      headline `Hermes. Code investigator.` → `YClaw. Code
      investigator.`; `uwu` headline `hermes-san is
      wistening` → `yclaw-san is wistening`; `none` headline
      `Hermes Agent is ready.` → `YClaw is ready.`; update
      the body strings to drop the brand reference where
      they make a brand-name joke (e.g. the pirate / noir /
      uwu bodies) and otherwise leave the body content alone
- [x] 3.2 Verify: open desktop, trigger the chat empty state;
      confirm all 5 personalities display without "Hermes"
      in either headline or body

## 4. Desktop thinking / loading / notification strings

- [x] 4.1 In
      `apps/desktop/src/components/assistant-ui/thread.tsx:413`,
      change the literal `'Hermes is thinking'` →
      `'YClaw is thinking'`
- [x] 4.2 In
      `apps/desktop/src/components/assistant-ui/streaming.test.tsx:389,397`,
      change the assertion targets `'Hermes is loading a
      response'` → `'YClaw is loading a response'`
- [x] 4.3 In `apps/desktop/src/store/onboarding.ts:192`, change
      `title: 'Hermes is ready'` → `title: 'YClaw is ready'`
- [x] 4.4 In
      `apps/desktop/src/components/gateway-connecting-overlay.test.tsx:63`,
      change the fixture string `'Hermes backend did not
      become ready'` → `'YClaw backend did not become ready'`
- [x] 4.5 Verify: `cd apps/desktop && npm run test:ui` and
      `scripts/run_tests.sh apps/desktop/ -q` both pass

## 5. Desktop theme preset labels

- [x] 5.1 In `apps/desktop/src/themes/presets.ts`, sweep the
      `label` / `description` / `tooltip` / `name` fields of
      each preset for the brand literal and replace "Hermes
      Teal" / "Hermes desktop identity" → "YClaw Teal" /
      "YClaw desktop identity". Do NOT change any preset `id`
      or `localStorage` key
- [x] 5.2 Verify: open desktop theme picker, confirm labels
      read "YClaw Teal" and switching themes still persists
      the existing internal ID

## 6. Web index.html title and top-bar brand

- [x] 6.1 In `web/index.html:10`, change
      `<title>Hermes Agent - Dashboard</title>` →
      `<title>YClaw - Dashboard</title>`
- [x] 6.2 In `web/src/App.tsx:580`, change the top-bar brand
      label literal `Hermes` → `YClaw`
- [x] 6.3 Verify: open dashboard, check the browser tab title
      and the top-bar brand read "YClaw"

## 7. Web i18n rebrand (19 locales in lock-step)

- [x] 7.1 Sweep
      `web/src/i18n/en.ts` for every brand literal in string
      values: "Hermes Agent", "Update Hermes", "Updating
      Hermes", "Hermes plugins", "Hermes-inproppe",
      "Hermes-bővítmények", "Hermes plugins",
      "Hermes プラグイン", "Hermes Achievements",
      "Hermes コレクタブル", "Hermes-kentekens",
      "Hermes-jelvény", "Hermes のセッション履歴",
      "Hermes のコレクタブル", "Hermes-sessiegeskiedenis",
      "Hermes を更新", etc. Replace each with the "YClaw"
      equivalent (Japanese keeps the trailing "を" particle,
      Chinese keeps "的", etc.)
- [x] 7.2 Mirror the replacements in
      `web/src/i18n/{af,de,es,fr,ga,hu,it,ja,ko,pt,ru,tr,uk,zh,zh-hant}.ts`
      and the 3 remaining locale files. The brand-token
      replacement count must match `en.ts` per file
- [x] 7.3 Verify: `rg "Hermes" web/src/i18n/*.ts` and confirm
      no string-value hits remain (type names and function
      references may keep the token)

## 8. Web theme preset labels

- [x] 8.1 In `web/src/themes/presets.ts:43,44,288,289`, change
      `label: "Hermes Teal"` → `label: "YClaw Teal"`,
      `description: "Classic dark teal — the canonical
      Hermes look"` → `description: "Classic dark teal —
      the canonical YClaw look"`, and the matching
      "(Large)" variant. Do NOT change the preset `id`s
- [x] 8.2 Verify: open dashboard theme picker; confirm labels
      read "YClaw Teal" / "YClaw Teal (Large)" and the
      previously-selected theme persists

## 9. Web error messages, system page, channels page

- [x] 9.1 In `web/src/lib/gatewayClient.ts:130`, change
      "page must be served by the Hermes dashboard" →
      "page must be served by the YClaw dashboard"
- [x] 9.2 In `web/src/pages/ChatPage.tsx:137`, change the
      visible prose "Open this page through `hermes
      dashboard`, not directly." → "Open this page through
      `hermes dashboard`, not directly." where the prose
      "the Hermes dashboard" is replaced with "the YClaw
      dashboard" but the inline `<code>hermes
      dashboard</code>` substring stays verbatim (the inline
      code is a real CLI command)
- [x] 9.3 In `web/src/pages/SystemPage.tsx:423,527,530,531`,
      change "Hermes updates are managed outside this
      dashboard." → "YClaw updates are managed outside
      this dashboard."; "Update Hermes?" → "Update YClaw?";
      the two prose passages that mention "Hermes" around
      the `'hermes update'` command → "YClaw" (keeping the
      `<code>hermes update</code>` substrings verbatim); the
      `<div>Hermes</div>` label and `<span>v{...}</span>`
      heading on the Hermes version card → "YClaw"
- [x] 9.4 In `web/src/pages/ChannelsPage.tsx:565`, change
      `bot_name: "Hermes Agent"` → `bot_name: "YClaw Agent"`.
      Do NOT change the `<code>hermes gateway start</code>`
      or `~/.hermes/.env` references in the same file
- [x] 9.5 Verify: open dashboard, trigger each error path
      (boot failure, channel onboarding, system update
      dialog) and confirm the prose reads "YClaw" while
      any inline `<code>hermes ...</code>` command
      references stay intact

## 10. Web aria / id / className / dev-log prefix sweep

- [x] 10.1 In `web/src/App.tsx:634,644`, change
      `aria-labelledby="hermes-sidebar-plugin-nav-heading"`
      and the matching `id="hermes-sidebar-plugin-nav-heading"`
      both to `yclaw-sidebar-plugin-nav-heading` (keep the
      two strings in lock-step)
- [x] 10.2 In `web/src/pages/ChatPage.tsx:473,895`, change
      the dev-log prefix `[hermes-chat] ...` → `[yclaw-chat]
      ...` and the className `"hermes-chat-xterm-host"` →
      `"yclaw-chat-xterm-host"`
- [x] 10.3 Verify: open dashboard, inspect the DOM for
      `aria-labelledby` / `id` values; open devtools, confirm
      the `[yclaw-chat]` log prefix and the
      `yclaw-chat-xterm-host` className

## 11. Web tests rebrand sweep

- [x] 11.1 Run `rg "Hermes" web/src/**/*.test.{ts,tsx}` and
      identify all brand-literal assertions
- [x] 11.2 Update each assertion's expected string to use the
      "YClaw" equivalent. Do NOT change assertions that
      target preserved identifiers (storage keys, env vars,
      runtime globals, download filenames, skill IDs, CLI
      commands, HTTP header, docs URL, data directory)
- [x] 11.3 Verify: `cd web && npm run test:ui` (vitest) and
      `npm run lint` both pass

## 12. Desktop tests rebrand sweep

- [x] 12.1 Run `rg "Hermes" apps/desktop/src/**/*.test.{ts,tsx}`
      and identify all brand-literal assertions
- [x] 12.2 Update each assertion's expected string to use the
      "YClaw" equivalent. Do NOT change assertions that
      target preserved identifiers (storage keys, IPC
      channel names, env vars, runtime globals, install
      paths, package metadata, plist keys, the
      `~/.hermes-bootstrap-complete` flag, etc.)
- [x] 12.3 Verify: `cd apps/desktop && npm run test:ui` and
      `scripts/run_tests.sh apps/desktop/ -q` both pass

## 13. Cross-surface audit

- [x] 13.1 Run `rg -n "[Hh]ermes" apps/desktop/{src,electron}/`
      and confirm every remaining hit is one of the
      preserved identifiers (storage key, IPC channel, env
      var, runtime global, install path, package name, plist
      key, bot URL, data directory, test fixture that
      exercises a preserved identifier) or a comment that
      documents the preserved identifier
- [x] 13.2 Run `rg -n "[Hh]ermes" web/src/` and confirm every
      remaining hit is one of the preserved identifiers
- [x] 13.3 Run `rg -n "[Yy]Claw" apps/desktop/src/ web/src/`
      and confirm the rebrand reached both surfaces
- [x] 13.4 Verify: `scripts/run_tests.sh apps/desktop/ -q` and
      `cd web && npm run test:ui` and `cd web && npm run
      typecheck` and `cd apps/desktop && npm run typecheck`
      and `cd apps/desktop && npm run lint` all pass
- [x] 13.5 Verify: launch desktop (`cd apps/desktop && npm
      run dev`), open dashboard (`cd web && npm run dev`),
      walk through: window title, About, intro state,
      thinking label, theme picker, error path, system
      page, channels page, matrix provider config — all
      show "YClaw" in prose and keep "hermes" only in
      preserved identifiers and inline `<code>hermes
      ...</code>` CLI substrings
