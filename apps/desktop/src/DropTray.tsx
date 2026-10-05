import { useEffect, useState } from "react";
import type {
  DroppedFileInspection,
  DropIntakeSnapshot,
} from "./bridge/types";
import "./drop.css";

type Props = {
  snapshot: DropIntakeSnapshot;
  hovering: boolean;
  paused: boolean;
  attachedIds: string[];
  onToggleAttach: (dropId: string) => void;
  onAttachAll: () => void;
  onClear: () => Promise<DropIntakeSnapshot>;
  onReveal: (dropId: string) => Promise<string>;
  onInspect: (dropId: string) => Promise<DroppedFileInspection>;
  onAnalyze: (dropIds: string[]) => Promise<unknown>;
  onUseVision: (dropId: string) => Promise<unknown>;
};

function formatBytes(bytes: number) {
  if (bytes >= 1_000_000_000) return `${(bytes / 1_000_000_000).toFixed(2)} GB`;
  if (bytes >= 1_000_000) return `${(bytes / 1_000_000).toFixed(1)} MB`;
  if (bytes >= 1_000) return `${(bytes / 1_000).toFixed(1)} KB`;
  return `${bytes} B`;
}

function kindGlyph(kind: string) {
  switch (kind) {
    case "image":
      return "▣";
    case "video":
      return "▶";
    case "audio":
      return "♪";
    case "document":
      return "▤";
    case "archive":
      return "◇";
    default:
      return "□";
  }
}

function contextCapability(item: DropIntakeSnapshot["items"][number]) {
  if (item.canPreviewText) {
    return {
      label: "Text context",
      detail: "Bounded local text can be used by AURA.",
      tone: "ready",
    };
  }

  if (item.kind === "image") {
    return {
      label: "Image metadata",
      detail: "Dimensions now; Vision is available explicitly.",
      tone: "partial",
    };
  }

  return {
    label: "Metadata only",
    detail: "Contents stay unopened in this Beta step.",
    tone: "metadata",
  };
}

export default function DropTray({
  snapshot,
  hovering,
  paused,
  attachedIds,
  onToggleAttach,
  onAttachAll,
  onClear,
  onReveal,
  onInspect,
  onAnalyze,
  onUseVision,
}: Props) {
  const [inspections, setInspections] = useState<
    Record<string, DroppedFileInspection>
  >({});
  const [inspectionErrors, setInspectionErrors] = useState<
    Record<string, string>
  >({});
  const [busyIds, setBusyIds] = useState<Record<string, boolean>>({});
  const [analyzing, setAnalyzing] = useState(false);
  const [batchError, setBatchError] = useState<string | null>(null);
  const visible = hovering || snapshot.items.length > 0;
  const allAttached =
    snapshot.items.length > 0
    && snapshot.items.every((item) => attachedIds.includes(item.id));

  useEffect(() => {
    setInspections({});
    setInspectionErrors({});
    setBusyIds({});
    setAnalyzing(false);
    setBatchError(null);
  }, [snapshot.refreshedAtMs]);

  if (!visible) return null;

  async function inspectOne(dropId: string) {
    setBusyIds((current) => ({ ...current, [dropId]: true }));
    setInspectionErrors((current) => {
      const next = { ...current };
      delete next[dropId];
      return next;
    });

    try {
      const inspection = await onInspect(dropId);
      setInspections((current) => ({
        ...current,
        [dropId]: inspection,
      }));
    } catch (error) {
      setInspectionErrors((current) => ({
        ...current,
        [dropId]: String(error),
      }));
    } finally {
      setBusyIds((current) => ({ ...current, [dropId]: false }));
    }
  }

  async function inspectAll() {
    await Promise.all(snapshot.items.map((item) => inspectOne(item.id)));
  }

  async function analyzeAll() {
    setAnalyzing(true);
    setBatchError(null);
    try {
      await onAnalyze(snapshot.items.map((item) => item.id));
    } catch (error) {
      setBatchError(String(error));
    } finally {
      setAnalyzing(false);
    }
  }

  return (
    <>
      {hovering && (
        <div className="drop-target-overlay" aria-hidden="true">
          <div className="drop-target-card">
            <span className="drop-target-icon">↓</span>
            <strong>Drop files into AURA</strong>
            <span>
              Metadata only. Nothing is opened, executed or analyzed automatically.
            </span>
          </div>
        </div>
      )}

      {snapshot.items.length > 0 && !hovering && (
        <aside className="drop-tray" aria-live="polite">
          <div className="drop-tray-heading">
            <div>
              <span className="feature-kicker">FILE CONTEXT V2</span>
              <strong>
                {snapshot.items.length} local file
                {snapshot.items.length === 1 ? "" : "s"} ready
              </strong>
            </div>
            <div className="drop-tray-heading-actions">
              <button
                type="button"
                className="feature-secondary-button"
                disabled={
                  paused
                  || snapshot.items.length === 0
                  || analyzing
                  || allAttached
                }
                onClick={onAttachAll}
              >
                {allAttached ? "All attached" : "Attach all to Chat"}
              </button>
              <button
                type="button"
                className="feature-primary-button"
                disabled={paused || snapshot.items.length === 0 || analyzing}
                onClick={() => void analyzeAll()}
              >
                {analyzing ? "Analyzing…" : "Analyze with AURA"}
              </button>
              <button
                type="button"
                className="feature-secondary-button"
                disabled={paused || snapshot.items.length === 0 || analyzing}
                onClick={() => void inspectAll()}
              >
                Inspect all
              </button>
              <button
                type="button"
                className="drop-tray-close"
                aria-label="Dismiss dropped files"
                onClick={() => {
                  setInspections({});
                  setInspectionErrors({});
                  void onClear();
                }}
              >
                ×
              </button>
            </div>
          </div>

          {(snapshot.rejectedCount > 0 || snapshot.truncated) && (
            <div className="drop-tray-warning">
              {snapshot.rejectedCount > 0
                ? `${snapshot.rejectedCount} item${snapshot.rejectedCount === 1 ? "" : "s"} could not be accepted. `
                : ""}
              {snapshot.truncated
                ? "AURA accepts at most 8 files in one drop."
                : ""}
            </div>
          )}

          {batchError && (
            <div className="drop-tray-warning">
              Analyze with AURA failed: {batchError}
            </div>
          )}

          <div className="drop-tray-items">
            {snapshot.items.map((item) => {
              const inspection = inspections[item.id];
              const inspectionError = inspectionErrors[item.id];
              const busy = Boolean(busyIds[item.id]);
              const attached = attachedIds.includes(item.id);

              return (
                <article className="drop-file-card" key={item.id}>
                  <div className="drop-file-icon" aria-hidden="true">
                    {kindGlyph(item.kind)}
                  </div>
                  <div className="drop-file-copy">
                    <strong title={item.name}>{item.name}</strong>
                    <span>
                      {item.kind}
                      {item.extension ? ` · .${item.extension}` : ""}
                      {" · "}
                      {formatBytes(item.sizeBytes)}
                    </span>
                    {(() => {
                      const capability = contextCapability(item);
                      return (
                        <span
                          className={`drop-context-capability ${capability.tone}`}
                          title={capability.detail}
                        >
                          {capability.label}
                        </span>
                      );
                    })()}
                    {item.modifiedAtMs > 0 && (
                      <small>
                        Modified {new Date(item.modifiedAtMs).toLocaleString()}
                      </small>
                    )}
                  </div>
                  <div className="drop-file-actions">
                    <button
                      type="button"
                      className={attached ? "feature-primary-button" : "feature-secondary-button"}
                      disabled={paused || analyzing}
                      onClick={() => onToggleAttach(item.id)}
                    >
                      {attached ? "Attached" : "Attach to Chat"}
                    </button>
                    {item.canInspect && (
                      <button
                        type="button"
                        className="feature-secondary-button"
                        disabled={paused || busy || analyzing}
                        onClick={() => void inspectOne(item.id)}
                      >
                        {busy ? "Inspecting…" : "Inspect"}
                      </button>
                    )}
                    <button
                      type="button"
                      className="feature-secondary-button"
                      disabled={paused || analyzing}
                      onClick={() => void onAnalyze([item.id])}
                    >
                      Analyze
                    </button>
                    {item.canUseVision && (
                      <button
                        type="button"
                        className="feature-primary-button"
                        disabled={paused}
                        onClick={() => void onUseVision(item.id)}
                      >
                        Use in Vision
                      </button>
                    )}
                    {item.canReveal && (
                      <button
                        type="button"
                        className="feature-secondary-button"
                        disabled={paused}
                        onClick={() => void onReveal(item.id)}
                      >
                        Reveal
                      </button>
                    )}
                  </div>

                  {inspection && (
                    <div className="drop-file-inspection">
                      <div className="drop-file-inspection-heading">
                        <strong>Local inspection</strong>
                        <span>{inspection.contentMode}</span>
                      </div>
                      <p>{inspection.summary}</p>

                      {inspection.imageWidth && inspection.imageHeight && (
                        <span className="drop-file-inspection-meta">
                          {inspection.imageWidth} × {inspection.imageHeight}px
                        </span>
                      )}

                      {inspection.note && (
                        <span className="drop-file-inspection-note">
                          {inspection.note}
                        </span>
                      )}

                      {inspection.textPreview && (
                        <div className="drop-file-preview">
                          <div>
                            <strong>Bounded text preview</strong>
                            {inspection.previewTruncated && <span>truncated</span>}
                          </div>
                          <pre>{inspection.textPreview}</pre>
                        </div>
                      )}
                    </div>
                  )}

                  {inspectionError && (
                    <div className="drop-file-inspection-error">
                      {inspectionError}
                    </div>
                  )}
                </article>
              );
            })}
          </div>

          <div className="drop-tray-privacy">
            <strong>Temporary session context</strong>
            <span>
              File paths remain inside AURA Core memory. Attach to Chat only adds
              an opaque temporary reference to the next desktop message. Text
              context is budgeted fairly across all attachments, remains bounded
              and ephemeral for that model turn, and is not added to Memory,
              diagnostics, Agents or Automations. Metadata-only files stay unopened.
            </span>
          </div>
        </aside>
      )}
    </>
  );
}
