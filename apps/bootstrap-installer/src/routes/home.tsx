import { useEffect } from "react";
import { useStore } from "@nanostores/react";
import { AppTile } from "../components/app-tile";
import { PendingUpdateBanner } from "../components/pending-update-banner";
import {
  $networkStatus,
  $hermesHome,
  $route,
  loadAppsList,
  launchApp,
  applyPendingUpdate,
  repairApp,
  openAppSettings,
  startInstall,
} from "../store";
import type { LaunchableApp } from "../lib/launcher-mode";

interface HomeProps {
  apps: LaunchableApp[];
  onSelect: (id: string) => void;
  onInstallAllPending: () => void;
}

export function Home({ apps, onSelect, onInstallAllPending }: HomeProps) {
  const networkStatus = useStore($networkStatus);
  const hermesHome = useStore($hermesHome);

  useEffect(() => {
    void loadAppsList();
  }, []);

  // Dispatch tile primary-button + ⋯ menu actions. Actions that need a
  // confirmation dialog or a richer surface (uninstall, view details) fall
  // through to AppDetail via onSelect.
  function handleTileAction(id: string, action: string) {
    switch (action) {
      case "launch":
        void launchApp(id);
        break;
      case "update":
        void applyPendingUpdate(id);
        break;
      case "install":
        void startInstall();
        break;
      case "repair":
        void repairApp(id);
        break;
      case "settings":
        void openAppSettings(id);
        break;
      case "uninstall":
      case "details":
      default:
        onSelect(id);
        break;
    }
  }

  const pendingApps = apps.filter((a) => a.pending?.status === "ready");
  const sorted = [...apps].sort((a, b) => Number(b.descriptor.default) - Number(a.descriptor.default));

  return (
    <main data-testid="home">
      <header>
        <h1>YClaw</h1>
        <span className="network-status" title={`Latency: ${networkStatus.latencyMs ?? "?"}ms`}>
          {networkStatus.hasInternet ? "Online" : "Offline"}
        </span>
        <button className="settings-link" onClick={() => $route.set("settings")}>
          Settings
        </button>
      </header>

      {pendingApps.length > 0 && (
        <PendingUpdateBanner
          appId={pendingApps[0].descriptor.id}
          pending={pendingApps[0].pending!}
          onInstall={onInstallAllPending}
          onDismiss={() => {}}
        />
      )}

      {apps.length === 0 ? (
        <div className="empty-state">
          <p>No apps installed</p>
        </div>
      ) : (
        <section className="tile-grid">
          {sorted.map((app) => (
            <div
              key={app.descriptor.id}
              data-testid="app-tile"
              onClick={() => onSelect(app.descriptor.id)}
            >
              <AppTile
                app={app}
                onAction={(action) => handleTileAction(app.descriptor.id, action)}
              />
            </div>
          ))}
        </section>
      )}

      <footer>
        <span>HERMES_HOME: {hermesHome ?? "loading..."}</span>
      </footer>
    </main>
  );
}