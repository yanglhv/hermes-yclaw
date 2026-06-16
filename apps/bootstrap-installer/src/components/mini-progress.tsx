interface MiniProgressProps {
  stage: string;
  progress: number;
  appId?: string;
}

export function MiniProgress({ stage, progress, appId }: MiniProgressProps) {
  if (progress < 0 || !stage) return null;

  return (
    <div
      className="mini-progress"
      data-testid="mini-progress"
      data-app-id={appId}
    >
      <span className="mini-progress__stage">{stage}</span>
      <span className="mini-progress__percent">{Math.min(100, Math.max(0, progress))}%</span>
      <div className="mini-progress__bar">
        <div
          className="mini-progress__fill"
          style={{ width: `${Math.min(100, Math.max(0, progress))}%` }}
        />
      </div>
    </div>
  );
}
