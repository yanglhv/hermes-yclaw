import { useEffect } from "react";
import { useStore } from "@nanostores/react";
import { AppTile } from "../components/app-tile";
import { PendingUpdateBanner } from "../components/pending-update-banner";
import { $networkStatus, $hermesHome, loadAppsList } from "../store";
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

  const pendingApps = apps.filter((a) => a.pending?.status === "ready");
  const sorted = [...apps].sort((a, b) => Number(b.descriptor.default) - Number(a.descriptor.default));

  return (
    <main data-testid="home">
      <header>
        <h1>YClaw</h1>
        <span className="network-status" title={`Latency: ${networkStatus.latencyMs ?? "?"}ms`}>
          {networkStatus.hasInternet ? "Online" : "Offline"}
        </span>
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
              <AppTile app={app} onAction={() => {}} />
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