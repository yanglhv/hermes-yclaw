import type { LaunchableApp } from "../lib/launcher-mode";

function deriveBadge(app: LaunchableApp): { label: string; cls: string } {
  if (app.launcher_too_old) return { label: "Update launcher", cls: "badge-warn" };
  if (app.pending?.status === "ready") return { label: "Update available", cls: "badge-update" };
  if (app.pending?.status === "downloading") return { label: "Downloading", cls: "badge-progress" };
  if (app.installed) return { label: "Installed", cls: "badge-ok" };
  return { label: "Not installed", cls: "badge-default" };
}

function primaryAction(app: LaunchableApp): { label: string; disabled: boolean } {
  if (app.launcher_too_old) return { label: "Install", disabled: true };
  if (app.pending?.status === "ready") return { label: "Update", disabled: false };
  if (app.installed) return { label: "Launch", disabled: false };
  return { label: "Install", disabled: false };
}

export function AppTile({ app, onAction }: { app: LaunchableApp; onAction: (action: string) => void }) {
  const badge = deriveBadge(app);
  const primary = primaryAction(app);
  return (
    <article className="app-tile">
      <div className="app-tile__icon">{app.descriptor.display_name.slice(0, 1).toUpperCase()}</div>
      <h3 className="app-tile__name">{app.descriptor.display_name}</h3>
      <span className={`app-tile__badge ${badge.cls}`}>{badge.label}</span>
      {app.installed && app.installed.installed_commit && (
        <small>{app.installed.installed_ref_name} @ {app.installed.installed_commit.slice(0, 7)}</small>
      )}
      <button
        disabled={primary.disabled}
        onClick={(e) => {
          e.stopPropagation();
          onAction(primary.label.toLowerCase());
        }}
      >
        {primary.label}
      </button>
      <details className="app-tile__menu">
        <summary>⋯</summary>
        <button onClick={(e) => { e.stopPropagation(); onAction("repair"); }}>Repair</button>
        <button onClick={(e) => { e.stopPropagation(); onAction("uninstall"); }}>Uninstall</button>
        {app.descriptor.app_settings_url && (
          <button onClick={(e) => { e.stopPropagation(); onAction("settings"); }}>Open app settings</button>
        )}
        <button onClick={(e) => { e.stopPropagation(); onAction("details"); }}>View details</button>
      </details>
    </article>
  );
}