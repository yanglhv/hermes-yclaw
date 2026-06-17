const assert = require('node:assert/strict')
const test = require('node:test')
const fs = require('node:fs')
const os = require('node:os')
const path = require('node:path')

const {
  runBootstrap,
  resolveInstallScript,
  installedAgentInstallScript,
  cachedScriptPath
} = require('./bootstrap-runner.cjs')

const SCRIPT_NAME = process.platform === 'win32' ? 'install.ps1' : 'install.sh'

function mkTmpHome() {
  return fs.mkdtempSync(path.join(os.tmpdir(), 'hermes-bootstrap-test-'))
}

test('runBootstrap bails immediately when the signal is already aborted', async () => {
  const controller = new AbortController()
  controller.abort()

  const events = []
  const result = await runBootstrap({
    installStamp: null,
    activeRoot: '/tmp/hermes-runner-test',
    sourceRepoRoot: null,
    hermesHome: '/tmp/hermes-runner-test',
    logRoot: '/tmp/hermes-runner-test',
    onEvent: ev => events.push(ev),
    abortSignal: controller.signal
  })

  // Cancelled before any install script is spawned.
  assert.deepEqual(result, { ok: false, cancelled: true })
  assert.ok(
    events.some(ev => ev.type === 'failed' && /cancelled/i.test(ev.error)),
    'should emit a cancelled failure event'
  )
})

test('installedAgentInstallScript resolves the installer in the agent checkout', () => {
  const home = mkTmpHome()
  try {
    assert.equal(installedAgentInstallScript(home), null, 'absent before the checkout exists')

    const scriptsDir = path.join(home, 'hermes-agent', 'scripts')
    fs.mkdirSync(scriptsDir, { recursive: true })
    const scriptPath = path.join(scriptsDir, SCRIPT_NAME)
    fs.writeFileSync(scriptPath, '#!/bin/sh\necho hi\n')

    assert.equal(installedAgentInstallScript(home), scriptPath)
    assert.equal(installedAgentInstallScript(null), null, 'null home -> null')
  } finally {
    fs.rmSync(home, { recursive: true, force: true })
  }
})

test('resolveInstallScript prefers a cached script without touching the network', async () => {
  const home = mkTmpHome()
  try {
    const commit = 'a'.repeat(40)
    const cached = cachedScriptPath(home, commit)
    fs.mkdirSync(path.dirname(cached), { recursive: true })
    fs.writeFileSync(cached, '#!/bin/sh\necho cached\n')

    const logs = []
    const result = await resolveInstallScript({
      installStamp: { commit },
      sourceRepoRoot: null,
      hermesHome: home,
      emit: ev => logs.push(ev)
    })

    assert.equal(result.source, 'cache')
    assert.equal(result.path, cached)
  } finally {
    fs.rmSync(home, { recursive: true, force: true })
  }
})

test('resolveInstallScript falls back to the installed agent checkout on a 404', async () => {
  const home = mkTmpHome()
  try {
    const commit = 'a'.repeat(40)
    // Seed the installed agent checkout so the fallback has something to resolve.
    const scriptsDir = path.join(home, 'hermes-agent', 'scripts')
    fs.mkdirSync(scriptsDir, { recursive: true })
    const installed = path.join(scriptsDir, SCRIPT_NAME)
    fs.writeFileSync(installed, '#!/bin/sh\necho fallback\n')

    const logs = []
    const result = await resolveInstallScript({
      installStamp: { commit },
      sourceRepoRoot: null,
      hermesHome: home,
      emit: ev => logs.push(ev),
      // Simulate GitHub returning a 404 for the pinned commit.
      _download: async () => {
        throw new Error('Failed to download install.sh: HTTP 404')
      }
    })

    assert.equal(result.source, 'installed-agent')
    // It should have copied the installer into the bootstrap cache.
    assert.equal(result.path, cachedScriptPath(home, commit))
    assert.ok(fs.existsSync(result.path), 'fallback script copied into cache')
    assert.ok(
      logs.some(ev => /falling back to installed agent/.test(ev.line || '')),
      'emits a fallback log line'
    )
  } finally {
    fs.rmSync(home, { recursive: true, force: true })
  }
})

test('resolveInstallScript rethrows when the 404 fallback is unavailable', async () => {
  const home = mkTmpHome()
  try {
    const commit = 'a'.repeat(40)
    // No installed agent checkout seeded -> nothing to fall back to.
    await assert.rejects(
      resolveInstallScript({
        installStamp: { commit },
        sourceRepoRoot: null,
        hermesHome: home,
        emit: () => {},
        _download: async () => {
          throw new Error('Failed to download install.sh: HTTP 404')
        }
      }),
      /HTTP 404|Failed to download/
    )
  } finally {
    fs.rmSync(home, { recursive: true, force: true })
  }
})

test('spawnPowerShell forwards HERMES_INSTALL_USE_LOCAL_REPO from parent env', async () => {
  // We need to test that the env var bubbles through. Since spawnPowerShell
  // is internal, we test it via runBootstrap or via a small refactor: we
  // import the internal helper if exported, else test via a mock.
  //
  // Strategy: use a one-shot module cache reset by re-requiring the file
  // with a stubbed child_process.spawn that captures the env arg.

  const { spawn } = require('node:child_process')
  const originalSpawn = spawn
  const captured = []
  const fakeSpawn = (...args) => {
    captured.push(args)
    // Return a minimal EventEmitter-like child so the await doesn't hang
    const { EventEmitter } = require('node:events')
    const child = new EventEmitter()
    child.stdout = new EventEmitter()
    child.stderr = new EventEmitter()
    child.kill = () => {}
    return child
  }

  // Simulate: parent process has HERMES_INSTALL_USE_LOCAL_REPO=/foo
  const prevVal = process.env.HERMES_INSTALL_USE_LOCAL_REPO
  process.env.HERMES_INSTALL_USE_LOCAL_REPO = '/foo/bar'

  try {
    require('node:child_process').spawn = fakeSpawn
    // Force a fresh require of bootstrap-runner so the env override is read
    delete require.cache[require.resolve('./bootstrap-runner.cjs')]
    const fresh = require('./bootstrap-runner.cjs')

    const home = mkTmpHome()
    // runBootstrap opens a log file via openRunLog whose write stream keeps
    // flushing after the promise resolves. If the log dir is inside `home`,
    // `rmSync(home, ...)` will race with the stream and trigger an
    // uncaughtException (ENOENT on the deleted log file). Put the log dir
    // outside `home` so cleanup of the test sandbox can't collide with the
    // stream's async flush. The log file leaks in os.tmpdir() — acceptable
    // for a single-run test.
    const logRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'hermes-bootstrap-log-'))
    try {
      // Trigger spawnPowerShell via a small script that wraps a stage
      // invocation. We use the runBootstrap path, but it needs installStamp
      // + sourceRepoRoot. The simplest test is to call the exported
      // resolveInstallScript path with a local script and let it spawn.
      const scriptsDir = path.join(home, 'hermes-agent', 'scripts')
      fs.mkdirSync(scriptsDir, { recursive: true })
      const scriptPath = path.join(scriptsDir, SCRIPT_NAME)
      fs.writeFileSync(scriptPath, '#!/bin/sh\necho hi\n')

      await fresh.runBootstrap({
        installStamp: null,
        activeRoot: home,
        sourceRepoRoot: path.dirname(scriptsDir),
        hermesHome: home,
        logRoot,
        onEvent: () => {}
      }).catch(() => {})  // ignore errors from the fake spawn returning no data

      // Find the spawn call that invoked our fake script
      const powerShellCall = captured.find(args =>
        args[1] && args[1].some && args[1].some(arg => arg && arg.includes && arg.includes(SCRIPT_NAME))
      )
      assert.ok(powerShellCall, 'spawn was called with the install script')
      if (powerShellCall) {
        const env = powerShellCall[2].env
        assert.equal(
          env.HERMES_INSTALL_USE_LOCAL_REPO,
          '/foo/bar',
          'parent HERMES_INSTALL_USE_LOCAL_REPO is forwarded to child env'
        )
      }
    } finally {
      fs.rmSync(home, { recursive: true, force: true })
    }
  } finally {
    require('node:child_process').spawn = originalSpawn
    if (prevVal === undefined) {
      delete process.env.HERMES_INSTALL_USE_LOCAL_REPO
    } else {
      process.env.HERMES_INSTALL_USE_LOCAL_REPO = prevVal
    }
  }
})

test('spawnPowerShell passes empty string for HERMES_INSTALL_USE_LOCAL_REPO when parent env unset', async () => {
  const { spawn } = require('node:child_process')
  const originalSpawn = spawn
  const captured = []
  const fakeSpawn = (...args) => {
    captured.push(args)
    const { EventEmitter } = require('node:events')
    const child = new EventEmitter()
    child.stdout = new EventEmitter()
    child.stderr = new EventEmitter()
    child.kill = () => {}
    return child
  }

  // Unset in parent
  const prevVal = process.env.HERMES_INSTALL_USE_LOCAL_REPO
  delete process.env.HERMES_INSTALL_USE_LOCAL_REPO

  try {
    require('node:child_process').spawn = fakeSpawn
    delete require.cache[require.resolve('./bootstrap-runner.cjs')]
    const fresh = require('./bootstrap-runner.cjs')

    const home = mkTmpHome()
    // runBootstrap opens a log file via openRunLog whose write stream keeps
    // flushing after the promise resolves. If the log dir is inside `home`,
    // `rmSync(home, ...)` will race with the stream and trigger an
    // uncaughtException (ENOENT on the deleted log file). Put the log dir
    // outside `home` so cleanup of the test sandbox can't collide with the
    // stream's async flush. The log file leaks in os.tmpdir() — acceptable
    // for a single-run test.
    const logRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'hermes-bootstrap-log-'))
    try {
      const scriptsDir = path.join(home, 'hermes-agent', 'scripts')
      fs.mkdirSync(scriptsDir, { recursive: true })
      const scriptPath = path.join(scriptsDir, SCRIPT_NAME)
      fs.writeFileSync(scriptPath, '#!/bin/sh\necho hi\n')

      await fresh.runBootstrap({
        installStamp: null,
        activeRoot: home,
        sourceRepoRoot: path.dirname(scriptsDir),
        hermesHome: home,
        logRoot,
        onEvent: () => {}
      }).catch(() => {})

      const powerShellCall = captured.find(args =>
        args[1] && args[1].some && args[1].some(arg => arg && arg.includes && arg.includes(SCRIPT_NAME))
      )
      assert.ok(powerShellCall, 'spawn was called with the install script')
      if (powerShellCall) {
        const env = powerShellCall[2].env
        assert.equal(
          env.HERMES_INSTALL_USE_LOCAL_REPO,
          '',
          'child env has empty string HERMES_INSTALL_USE_LOCAL_REPO when parent unset'
        )
      }
    } finally {
      fs.rmSync(home, { recursive: true, force: true })
    }
  } finally {
    require('node:child_process').spawn = originalSpawn
    if (prevVal !== undefined) process.env.HERMES_INSTALL_USE_LOCAL_REPO = prevVal
  }
})
