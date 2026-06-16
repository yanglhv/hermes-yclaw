import type { PendingUpdate } from "../lib/launcher-mode";

export function PendingUpdateBanner({
  appId,
  pending,
  onInstall,
  onDismiss,
}: {
  appId: string;
  pending: PendingUpdate;
  onInstall: () => void;
  onDismiss: () => void;
}) {
  if (!pending || pending.status === "failed") return null;

  const displayName = appId
    .split(/[-_]/)
    .map((w) => w.charAt(0).toUpperCase() + w.slice(1))
    .join(" ");

  const currentVersion = pending.installed_ref_name ?? pending.installed_commit?.slice(0, 7) ?? "unknown";
  const newVersion = pending.latest_ref_name ?? pending.latest_commit?.slice(0, 7) ?? "unknown";

  return (
    <div className="pending-update-banner" data-testid="pending-update-banner">
      <span>
        Update available for {displayName}: v{currentVersion} → v{newVersion}
      </span>
      <button onClick={onInstall}>Install now</button>
      <button onClick={onDismiss}>Dismiss</button>
    </div>
  );
}
