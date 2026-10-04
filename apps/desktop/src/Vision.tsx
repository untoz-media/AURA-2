import { useEffect, useMemo, useState } from "react";
import type {
  ManagedRuntimeStatus,
  ModelCatalog,
  ModelStatus,
  PermissionDecision,
  VisionAnalysisPayload,
  VisionCapture,
  VisionEvent,
  VisionHistorySnapshot,
  VisionPreferences,
  VisionRegionRequest,
  VisionRuntimeStatus,
} from "./bridge/types";
import { ShortcutKey } from "./design-system/components";
import "./feature-pages.css";
import "./vision.css";

type ModelOperation =
  | "download"
  | "pause"
  | "resume"
  | "cancel"
  | "activate"
  | "remove";

type Props = {
  catalog: ModelCatalog;
  managedRuntime: ManagedRuntimeStatus;
  runtime: VisionRuntimeStatus;
  history: VisionHistorySnapshot;
  capture: VisionCapture | null;
  event: VisionEvent | null;
  readPermission: PermissionDecision;
  onModelOperation: (
    operation: ModelOperation,
    modelId: string,
  ) => Promise<ModelCatalog>;
  onCaptureScreen: () => Promise<VisionCapture>;
  onCaptureActiveWindow: () => Promise<VisionCapture>;
  onCaptureRegion: (request: VisionRegionRequest) => Promise<VisionCapture>;
  onAnalyze: (prompt: string) => Promise<VisionAnalysisPayload>;
  onPreferencesChange: (
    preferences: VisionPreferences,
  ) => Promise<VisionHistorySnapshot>;
  onHistoryClear: () => Promise<VisionHistorySnapshot>;
  onCaptureClear: () => Promise<void>;
  onRuntimeRefresh: () => Promise<VisionRuntimeStatus>;
};

function modelStateLabel(model?: ModelStatus) {
  if (!model) return "Unavailable";
  switch (model.state) {
    case "notInstalled":
      return model.bytesDownloaded > 0 ? "Partial download" : "Not installed";
    case "downloading":
      return "Downloading";
    case "paused":
      return "Paused";
    case "installed":
      return "Ready";
    case "failed":
      return "Needs attention";
    case "unavailable":
      return "Unavailable";
  }
}

function formatSize(bytes?: number) {
  if (!bytes) return "—";
  if (bytes >= 1_000_000_000) return `${(bytes / 1_000_000_000).toFixed(2)} GB`;
  return `${(bytes / 1_000_000).toFixed(0)} MB`;
}

function captureLabel(capture: VisionCapture) {
  if (capture.kind === "activeWindow") return "Active window";
  if (capture.kind === "region") return "Selected region";
  return "Full screen";
}

export default function Vision({
  catalog,
  managedRuntime,
  runtime,
  history,
  capture,
  event,
  readPermission,
  onModelOperation,
  onCaptureScreen,
  onCaptureActiveWindow,
  onCaptureRegion,
  onAnalyze,
  onPreferencesChange,
  onHistoryClear,
  onCaptureClear,
  onRuntimeRefresh,
}: Props) {
  const model = useMemo(
    () => catalog.models.find((item) => item.id === "vision-smolvlm2-500m"),
    [catalog.models],
  );
  const [prompt, setPrompt] = useState(
    "Describe the visible interface. If there is an error or warning, explain it and identify the relevant visible controls.",
  );
  const [busy, setBusy] = useState<string | null>(null);
  const [localError, setLocalError] = useState<string | null>(null);
  const [analysis, setAnalysis] = useState<string | null>(
    runtime.lastAnalysis ?? null,
  );
  const [region, setRegion] = useState<VisionRegionRequest>({
    x: 0,
    y: 0,
    width: 800,
    height: 600,
  });

  useEffect(() => {
    if (event?.analysis) setAnalysis(event.analysis);
  }, [event?.analysis]);

  useEffect(() => {
    if (runtime.lastAnalysis) setAnalysis(runtime.lastAnalysis);
  }, [runtime.lastAnalysis]);

  const visionBlocked = readPermission === "never";

  const visionBusy =
    busy !== null ||
    runtime.state === "loading" ||
    runtime.state === "analyzing" ||
    event?.phase === "capturing" ||
    event?.phase === "analyzing";

  const screenAccessActive =
    event?.phase === "capturing" ||
    event?.phase === "captured" ||
    event?.phase === "regionSelecting" ||
    event?.phase === "analyzing";

  async function runModel(operation: ModelOperation) {
    if (!model) return;
    if (
      operation === "remove" &&
      !window.confirm(
        "Remove the AURA Vision model from this computer? Its downloaded model files will be deleted.",
      )
    ) {
      return;
    }

    setBusy(`model:${operation}`);
    setLocalError(null);
    try {
      await onModelOperation(operation, model.id);
      await onRuntimeRefresh().catch(() => undefined);
    } catch (error) {
      setLocalError(String(error));
    } finally {
      setBusy(null);
    }
  }

  async function captureNow(
    kind: "screen" | "activeWindow" | "region",
  ) {
    setBusy(`capture:${kind}`);
    setLocalError(null);
    try {
      if (kind === "screen") {
        await onCaptureScreen();
      } else if (kind === "activeWindow") {
        await onCaptureActiveWindow();
      } else {
        await onCaptureRegion(region);
      }
      setAnalysis(null);
    } catch (error) {
      setLocalError(String(error));
    } finally {
      setBusy(null);
    }
  }

  async function analyze() {
    if (!capture || !prompt.trim()) return;
    setBusy("analyze");
    setLocalError(null);
    try {
      const result = await onAnalyze(prompt.trim());
      setAnalysis(result.analysis);
    } catch (error) {
      setLocalError(String(error));
    } finally {
      setBusy(null);
    }
  }

  async function updatePreferences(patch: Partial<VisionPreferences>) {
    setBusy("preferences");
    setLocalError(null);
    try {
      await onPreferencesChange({
        ...history.preferences,
        ...patch,
      });
    } catch (error) {
      setLocalError(String(error));
    } finally {
      setBusy(null);
    }
  }

  return (
    <section className="feature-page vision-page">
      <header className="feature-hero vision-hero">
        <div>
          <span className="feature-kicker">AURA VISION</span>
          <h2>Let AURA understand what is visible on your PC.</h2>
          <p>
            Capture only when you ask. Screenshots are analyzed locally and are
            deleted after analysis unless you explicitly enable local Vision
            history.
          </p>
        </div>
        <div
          className={`vision-access-indicator ${screenAccessActive ? "active" : ""} ${visionBlocked ? "blocked" : ""}`}
        >
          <span />
          <div>
            <strong>
              {visionBlocked
                ? "SCREEN ACCESS BLOCKED"
                : screenAccessActive
                  ? "SCREEN ACCESS ACTIVE"
                  : "SCREEN ACCESS IDLE"}
            </strong>
            <small>
              {visionBlocked
                ? "Read permission is set to Never"
                : screenAccessActive
                  ? event?.message
                  : `Read permission: ${readPermission}`}
            </small>
          </div>
        </div>
      </header>

      <article className="vision-model-card">
        <div className="vision-model-heading">
          <div>
            <span className="feature-kicker">LOCAL VISION MODEL</span>
            <h3>{model?.name ?? "AURA Vision"}</h3>
            <p>
              SmolVLM2 500M analyzes screenshots locally. It is separate from
              the main AURA reasoning model.
            </p>
          </div>
          <span className={`feature-badge ${model?.state ?? "notInstalled"}`}>
            {modelStateLabel(model)}
          </span>
        </div>

        <div className="vision-model-meta">
          <span>{model?.sourceRepo ?? "HuggingFaceTB/SmolVLM2-500M-Video-Instruct"}</span>
          <span>{model?.license ?? "Apache-2.0"}</span>
          <span>{formatSize(model?.estimatedSizeBytes)}</span>
          <span>
            {runtime.cuda
              ? runtime.device ?? "CUDA"
              : runtime.device ?? "CPU / not loaded"}
          </span>
        </div>

        {(model?.state === "downloading" || model?.state === "paused") && (
          <div className="vision-download">
            <div className="model-progress-track">
              <span style={{ width: `${model.progressPercent}%` }} />
            </div>
            <small>{model.progressPercent.toFixed(1)}%</small>
          </div>
        )}

        <div className="vision-actions">
          {model?.state === "notInstalled" && (
            <button
              className="feature-primary-button"
              type="button"
              disabled={visionBusy || visionBlocked}
              onClick={() => void runModel("download")}
            >
              Install Vision model
            </button>
          )}
          {model?.state === "downloading" && (
            <>
              <button
                className="feature-secondary-button"
                type="button"
                onClick={() => void runModel("pause")}
              >
                Pause
              </button>
              <button
                className="feature-secondary-button"
                type="button"
                onClick={() => void runModel("cancel")}
              >
                Cancel
              </button>
            </>
          )}
          {model?.state === "paused" && (
            <button
              className="feature-primary-button"
              type="button"
              onClick={() => void runModel("resume")}
            >
              Resume
            </button>
          )}
          {model?.state === "installed" && (
            <button
              className="feature-secondary-button"
              type="button"
              disabled={visionBusy || visionBlocked}
              onClick={() => void runModel("remove")}
            >
              Remove model
            </button>
          )}
          <button
            className="feature-secondary-button"
            type="button"
            disabled={visionBusy || visionBlocked}
            onClick={() => void onRuntimeRefresh()}
          >
            Refresh
          </button>
        </div>

        {managedRuntime.state !== "ready" && (
          <div className="feature-note">
            <strong>Managed runtime required</strong>
            <span>
              Install or repair AURA's local AI runtime from Models before
              running visual understanding.
            </span>
          </div>
        )}
      </article>

      <div className="vision-grid">
        <article className="vision-panel">
          <div className="vision-panel-heading">
            <div>
              <span className="feature-kicker">CAPTURE</span>
              <h3>Choose exactly what AURA may see.</h3>
            </div>
            {capture && (
              <button
                type="button"
                className="feature-secondary-button"
                onClick={() => void onCaptureClear()}
              >
                Clear capture
              </button>
            )}
          </div>

          <div className="vision-capture-actions">
            <button
              type="button"
              className="feature-primary-button"
              disabled={visionBusy || visionBlocked}
              onClick={() => void captureNow("activeWindow")}
            >
              Capture active window
            </button>
            <button
              type="button"
              className="feature-secondary-button"
              disabled={visionBusy || visionBlocked}
              onClick={() => void captureNow("screen")}
            >
              Capture full screen
            </button>
          </div>

          <div className="vision-region-shortcut">
            <div>
              <strong>Quick region selection</strong>
              <span>
                Put the cursor on one corner, press the shortcut, move to the
                opposite corner and press it again.
              </span>
            </div>
            <div className="vision-shortcut">
              <ShortcutKey>Ctrl</ShortcutKey>
              <span>+</span>
              <ShortcutKey>Shift</ShortcutKey>
              <span>+</span>
              <ShortcutKey>F9</ShortcutKey>
            </div>
          </div>

          <div className="vision-region-grid">
            {(["x", "y", "width", "height"] as const).map((key) => (
              <label key={key}>
                <span>{key.toUpperCase()}</span>
                <input
                  type="number"
                  value={region[key]}
                  min={key === "width" || key === "height" ? 8 : undefined}
                  onChange={(input) =>
                    setRegion((current) => ({
                      ...current,
                      [key]: Number(input.target.value),
                    }))
                  }
                />
              </label>
            ))}
            <button
              type="button"
              className="feature-secondary-button"
              disabled={visionBusy || visionBlocked}
              onClick={() => void captureNow("region")}
            >
              Capture coordinates
            </button>
          </div>

          {capture ? (
            <div className="vision-capture-card">
              <span>{captureLabel(capture)}</span>
              <strong>
                {capture.rect.width} × {capture.rect.height}
              </strong>
              <small>
                {capture.windowTitle ??
                  `x ${capture.rect.x}, y ${capture.rect.y}`}
              </small>
              <em>Stored temporarily in AURA cache until analysis/clear.</em>
            </div>
          ) : (
            <div className="vision-empty">
              <strong>No screenshot selected.</strong>
              <span>AURA cannot see your screen until you explicitly capture it.</span>
            </div>
          )}
        </article>

        <article className="vision-panel">
          <div className="vision-panel-heading">
            <div>
              <span className="feature-kicker">UNDERSTAND</span>
              <h3>Ask about the visible interface.</h3>
            </div>
            <span className={`feature-badge ${runtime.state}`}>
              {runtime.state}
            </span>
          </div>

          <label className="vision-prompt">
            <span>Question for AURA Vision</span>
            <textarea
              value={prompt}
              onChange={(input) => setPrompt(input.target.value)}
              placeholder="What is wrong here? Where is the relevant control?"
            />
          </label>

          <button
            type="button"
            className="feature-primary-button vision-analyze-button"
            disabled={
              visionBusy ||
              visionBlocked ||
              !capture ||
              !prompt.trim() ||
              model?.state !== "installed" ||
              managedRuntime.state !== "ready"
            }
            onClick={() => void analyze()}
          >
            {runtime.state === "analyzing" || busy === "analyze"
              ? "Analyzing locally…"
              : "Analyze screenshot"}
          </button>

          {analysis ? (
            <div className="vision-analysis-card">
              <span>VISION RESULT</span>
              <p>{analysis}</p>
              <small>
                Visual understanding only. Suggested clicks/actions still go
                through AURA Computer Control and permissions.
              </small>
            </div>
          ) : (
            <div className="vision-empty">
              <strong>No visual analysis yet.</strong>
              <span>Capture something and ask AURA what it can see.</span>
            </div>
          )}

          {(localError || runtime.lastError || event?.phase === "error") && (
            <p className="vision-error">
              {localError || runtime.lastError || event?.message}
            </p>
          )}
        </article>
      </div>

      <article className="vision-panel">
        <div className="vision-panel-heading">
          <div>
            <span className="feature-kicker">PRIVACY & HISTORY</span>
            <h3>Nothing is remembered unless you choose it.</h3>
          </div>
          <span className="feature-badge">
            {history.preferences.historyEnabled
              ? `${history.items.length} saved`
              : "History off"}
          </span>
        </div>

        <div className="vision-history-settings">
          <label>
            <input
              type="checkbox"
              checked={history.preferences.historyEnabled}
              disabled={busy === "preferences"}
              onChange={(input) =>
                void updatePreferences({
                  historyEnabled: input.target.checked,
                })
              }
            />
            <div>
              <strong>Save Vision analysis history</strong>
              <span>Stores prompts and text results locally.</span>
            </div>
          </label>

          <label>
            <input
              type="checkbox"
              checked={history.preferences.retainImages}
              disabled={
                !history.preferences.historyEnabled || busy === "preferences"
              }
              onChange={(input) =>
                void updatePreferences({
                  retainImages: input.target.checked,
                })
              }
            />
            <div>
              <strong>Retain screenshots</strong>
              <span>
                Off by default. When off, screenshots are deleted after
                analysis even if text history is enabled.
              </span>
            </div>
          </label>

          <label className="vision-history-limit">
            <span>Maximum history</span>
            <select
              value={history.preferences.maxHistory}
              disabled={
                !history.preferences.historyEnabled || busy === "preferences"
              }
              onChange={(input) =>
                void updatePreferences({
                  maxHistory: Number(input.target.value),
                })
              }
            >
              {[5, 10, 20, 50].map((amount) => (
                <option value={amount} key={amount}>
                  {amount}
                </option>
              ))}
            </select>
          </label>

          <button
            type="button"
            className="feature-secondary-button"
            disabled={history.items.length === 0 || visionBusy}
            onClick={() => void onHistoryClear()}
          >
            Clear Vision history
          </button>
        </div>

        {history.items.length > 0 && (
          <div className="vision-history-list">
            {history.items.slice(0, 6).map((item) => (
              <article key={item.id}>
                <div>
                  <span>{item.captureKind}</span>
                  <time>
                    {new Date(item.createdAtMs).toLocaleString()}
                  </time>
                </div>
                <strong>{item.prompt}</strong>
                <p>{item.analysis}</p>
                <small>
                  {item.imagePath ? "Screenshot retained locally" : "Text only"}
                </small>
              </article>
            ))}
          </div>
        )}
      </article>
    </section>
  );
}
