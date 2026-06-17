import { useState, useEffect } from "react";
import { useStore } from "@nanostores/react";
import { $hermesHome, $launcherConfig, $route, loadLauncherConfig, saveLauncherConfig, type LauncherConfig } from "../store";

interface FormState {
  preferred_channel: string;
  auto_update: boolean;
  skip_network_probe: boolean;
  repo_override: string;
}

const DEFAULT_FORM: FormState = {
  preferred_channel: "stable",
  auto_update: true,
  skip_network_probe: false,
  repo_override: "",
};

function parseRepoOverride(input: string): { owner: string | null; name: string | null; ref_: string | null } {
  if (!input.trim()) {
    return { owner: null, name: null, ref_: null };
  }
  const atIndex = input.indexOf("@");
  const ref = atIndex !== -1 ? input.slice(atIndex + 1) : null;
  const beforeAt = atIndex !== -1 ? input.slice(0, atIndex) : input;
  const slashIndex = beforeAt.indexOf("/");
  const owner = slashIndex !== -1 ? beforeAt.slice(0, slashIndex) : null;
  const name = slashIndex !== -1 ? beforeAt.slice(slashIndex + 1) : null;
  return { owner, name, ref_: ref };
}

function formatRepoOverride(owner: string | null, name: string | null, ref_: string | null): string {
  if (!owner && !name && !ref_) return "";
  const repo = [owner, name].filter(Boolean).join("/");
  if (ref_) return `${repo}@${ref_}`;
  return repo;
}

export default function Settings() {
  const hermesHome = useStore($hermesHome);
  const launcherConfig = useStore($launcherConfig);

  const [form, setForm] = useState<FormState>(DEFAULT_FORM);
  const [saveStatus, setSaveStatus] = useState<"idle" | "saving" | "saved" | "error">("idle");

  useEffect(() => {
    void loadLauncherConfig();
  }, []);

  useEffect(() => {
    if (launcherConfig) {
      setForm({
        preferred_channel: launcherConfig.preferred_channel || "stable",
        auto_update: launcherConfig.auto_update,
        skip_network_probe: launcherConfig.skip_network_probe,
        repo_override: formatRepoOverride(
          launcherConfig.repo.owner,
          launcherConfig.repo.name,
          launcherConfig.repo.ref_
        ),
      });
    }
  }, [launcherConfig]);

  function handleChange(field: keyof FormState, value: string | boolean) {
    setForm((prev) => ({ ...prev, [field]: value }));
    setSaveStatus("idle");
  }

  async function handleSave() {
    setSaveStatus("saving");
    try {
      const repo = parseRepoOverride(form.repo_override);
      const config: LauncherConfig = {
        preferred_channel: form.preferred_channel,
        auto_update: form.auto_update,
        skip_network_probe: form.skip_network_probe,
        repo,
      };
      await saveLauncherConfig(config);
      setSaveStatus("saved");
    } catch {
      setSaveStatus("error");
    }
  }

  async function handleReset() {
    const defaults: LauncherConfig = {
      preferred_channel: "stable",
      auto_update: false,
      skip_network_probe: false,
      repo: { owner: null, name: null, ref_: null },
    };
    setForm(DEFAULT_FORM);
    setSaveStatus("idle");
    try {
      await saveLauncherConfig(defaults);
    } catch {
      setSaveStatus("error");
    }
  }

  return (
    <main className="settings" data-testid="settings">
      <header className="settings__header">
        <button className="back-button" onClick={() => $route.set("home")} aria-label="Back to home">
          ← Home
        </button>
        <h1>Settings</h1>
      </header>

      <section className="settings__section">
        <h2>Launcher Configuration</h2>

        <div className="settings__field">
          <label htmlFor="preferred_channel">Preferred Channel</label>
          <input
            id="preferred_channel"
            type="text"
            value={form.preferred_channel}
            onChange={(e) => handleChange("preferred_channel", e.target.value)}
            placeholder="stable"
          />
        </div>

        <div className="settings__field">
          <label htmlFor="auto_update">
            <input
              id="auto_update"
              type="checkbox"
              checked={form.auto_update}
              onChange={(e) => handleChange("auto_update", e.target.checked)}
            />
            Auto-update
          </label>
        </div>

        <div className="settings__field">
          <label htmlFor="skip_network_probe">
            <input
              id="skip_network_probe"
              type="checkbox"
              checked={form.skip_network_probe}
              onChange={(e) => handleChange("skip_network_probe", e.target.checked)}
            />
            Skip network probe
          </label>
        </div>

        <div className="settings__field">
          <label htmlFor="repo_override">Repository Override</label>
          <input
            id="repo_override"
            type="text"
            value={form.repo_override}
            onChange={(e) => handleChange("repo_override", e.target.value)}
            placeholder="owner/repo@ref"
          />
          <span className="settings__field-hint">e.g., owner/repo@ref</span>
        </div>
      </section>

      <section className="settings__actions">
        <button onClick={() => void handleSave()} disabled={saveStatus === "saving"}>
          {saveStatus === "saving" ? "Saving..." : "Save"}
        </button>
        <button onClick={handleReset}>Reset to defaults</button>
        {saveStatus === "saved" && <span className="settings__status">Saved!</span>}
        {saveStatus === "error" && <span className="settings__status settings__status--error">Failed to save</span>}
      </section>

      <section className="settings__info">
        <div className="settings__info-row">
          <span className="settings__info-label">HERMES_HOME:</span>
          <span className="settings__info-value">{hermesHome ?? "loading..."}</span>
        </div>
        <div className="settings__info-row">
          <span className="settings__info-label">YClaw version:</span>
          <span className="settings__info-value">0.0.1</span>
        </div>
      </section>
    </main>
  );
}
