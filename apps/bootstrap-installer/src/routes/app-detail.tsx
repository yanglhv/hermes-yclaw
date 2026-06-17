import { useState } from "react";
import { MiniProgress } from "../components/mini-progress";
import { PendingUpdateBanner } from "../components/pending-update-banner";
import {
  $bootstrapByApp,
  launchApp,
  repairApp,
  uninstallApp,
  applyPendingUpdate,
  openAppSettings,
  type LaunchableApp,
} from "../store";

interface AppDetailProps {
  app: LaunchableApp;
  onAction: (action: string) => void;
  onBack?: () => void;
}

export function AppDetail({ app, onAction, onBack }: AppDetailProps) {
  const bootstrapByApp = $bootstrapByApp.get();
  const bootstrap = bootstrapByApp[app.descriptor.id];

  const [showUninstallConfirm, setShowUninstallConfirm] = useState(false);

  const { descriptor, installed, pending, launcher_too_old } = app;
  const isRunning = bootstrap?.status === "running";
  const progressPercent = isRunning
    ? Math.round(
        (bootstrap.stageOrder.filter((name) => {
          const s = bootstrap.stages[name]?.state;
          return s === "succeeded" || s === "skipped" || s === "failed";
        }).length /
          Math.max(1, bootstrap.stageOrder.length)) *
          100
      )
    : 0;

  function handleAction(action: string) {
    switch (action) {
      case "launch":
        void launchApp(app.descriptor.id);
        break;
      case "update":
        void applyPendingUpdate(app.descriptor.id);
        break;
      case "repair":
        void repairApp(app.descriptor.id);
        break;
      case "uninstall":
        setShowUninstallConfirm(true);
        break;
      case "uninstall-light":
        void uninstallApp(app.descriptor.id, "light");
        setShowUninstallConfirm(false);
        break;
      case "uninstall-full":
        void uninstallApp(app.descriptor.id, "full");
        setShowUninstallConfirm(false);
        break;
      case "settings":
        void openAppSettings(app.descriptor.id);
        break;
      default:
        onAction(action);
        break;
    }
  }

  const iconLetter = descriptor.icon
    ? descriptor.icon
    : descriptor.display_name.slice(0, 1).toUpperCase();

  return (
    <main className="app-detail" data-testid="app-detail">
      {onBack && (
        <header className="app-detail__header">
          <button className="back-button" onClick={onBack} aria-label="Back to home">
            ← Back
          </button>
        </header>
      )}

      <div className="app-detail__info">
        <div className="app-detail__icon">{iconLetter}</div>
        <h1 className="app-detail__name">{descriptor.display_name}</h1>
        <p className="app-detail__version">
          {installed ? `Version ${descriptor.version}` : "Not installed"}
        </p>
        {installed && (
          <p className="app-detail__last-updated">
            {installed.installed_ref_name} @ {installed.installed_commit.slice(0, 7)} · via {installed.installed_via}
          </p>
        )}
        {installed && (
          <p className="app-detail__install-root" title={installed.install_root}>
            {installed.install_root}
          </p>
        )}
      </div>

      {pending && pending.status !== "failed" && (
        <PendingUpdateBanner
          appId={app.descriptor.id}
          pending={pending}
          onInstall={() => handleAction("update")}
          onDismiss={() => {}}
        />
      )}

      {isRunning && (
        <MiniProgress
          stage={bootstrap.currentStage ?? "Working..."}
          progress={progressPercent}
          appId={app.descriptor.id}
        />
      )}

      <div className="app-detail__actions">
        {launcher_too_old ? (
          <button disabled>Install (update launcher)</button>
        ) : installed ? (
          <button onClick={() => handleAction("launch")}>Launch</button>
        ) : (
          <button onClick={() => handleAction("update")}>Install</button>
        )}

        {installed && pending?.status === "ready" && (
          <button onClick={() => handleAction("update")}>Update</button>
        )}

        {installed && (
          <button onClick={() => handleAction("repair")}>Repair</button>
        )}

        {installed && descriptor.uninstall_supported && (
          <button onClick={() => handleAction("uninstall")}>Uninstall</button>
        )}

        {descriptor.app_settings_url && (
          <button onClick={() => handleAction("settings")}>
            Open app settings
          </button>
        )}
      </div>

      {showUninstallConfirm && (
        <div className="uninstall-confirm-dialog" role="dialog" aria-modal="true">
          <p>Are you sure you want to uninstall {descriptor.display_name}?</p>
          <div className="uninstall-confirm-dialog__actions">
            <button onClick={() => handleAction("uninstall-light")}>
              Uninstall (user data)
            </button>
            <button onClick={() => handleAction("uninstall-full")}>
              Uninstall (all data)
            </button>
            <button onClick={() => setShowUninstallConfirm(false)}>Cancel</button>
          </div>
        </div>
      )}
    </main>
  );
}