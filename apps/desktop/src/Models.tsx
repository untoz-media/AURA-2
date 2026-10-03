import { useState } from "react";
import type { ModelCatalog, ModelStatus } from "./bridge/types";
import { AuraMark } from "./design-system/components";
import "./feature-pages.css";

type ModelOperation =
  | "download"
  | "pause"
  | "resume"
  | "cancel"
  | "activate"
  | "remove";

type Props = {
  catalog: ModelCatalog;
  onRefresh: () => Promise<ModelCatalog>;
  onOperation: (
    operation: ModelOperation,
    modelId: string,
  ) => Promise<ModelCatalog>;
};

function formatBytes(bytes?: number) {
  if (bytes == null) return "Unknown";
  if (bytes >= 1_000_000_000) return `${(bytes / 1_000_000_000).toFixed(2)} GB`;
  if (bytes >= 1_000_000) return `${(bytes / 1_000_000).toFixed(1)} MB`;
  if (bytes >= 1_000) return `${(bytes / 1_000).toFixed(1)} KB`;
  return `${bytes} B`;
}

function stateLabel(model: ModelStatus) {
  if (model.active) return "Active";
  switch (model.state) {
    case "unavailable":
      return "Not released";
    case "notInstalled":
      return model.bytesDownloaded > 0 ? "Partial download" : "Not installed";
    case "downloading":
      return "Downloading";
    case "paused":
      return "Paused";
    case "installed":
      return "Installed";
    case "failed":
      return "Needs attention";
  }
}

function progressDetail(model: ModelStatus) {
  const downloaded = formatBytes(model.bytesDownloaded);
  const total = formatBytes(model.totalBytes ?? model.estimatedSizeBytes);

  if (model.totalBytes || model.estimatedSizeBytes) {
    return `${downloaded} / ${total}`;
  }

  return downloaded;
}

export default function Models({
  catalog,
  onRefresh,
  onOperation,
}: Props) {
  const [busy, setBusy] = useState<string | null>(null);
  const [localError, setLocalError] = useState<string | null>(null);

  async function run(operation: ModelOperation, model: ModelStatus) {
    if (
      operation === "remove" &&
      !window.confirm(
        `Remove ${model.name} from this computer? The downloaded model files will be deleted.`,
      )
    ) {
      return;
    }

    setBusy(`${model.id}:${operation}`);
    setLocalError(null);

    try {
      await onOperation(operation, model.id);
    } catch (error) {
      setLocalError(String(error));
    } finally {
      setBusy(null);
    }
  }

  return (
    <section className="feature-page">
      <header className="feature-hero">
        <div>
          <span className="feature-kicker">AURA MODEL MANAGER</span>
          <h2>Choose the intelligence that runs AURA.</h2>
          <p>
            Download local models once, pause or resume large transfers, verify
            the installation and choose which installed model AURA should use.
          </p>
        </div>
        <div className="feature-hero-mark">
          <AuraMark />
        </div>
      </header>

      <div className="model-manager-toolbar">
        <div>
          <span>Model storage</span>
          <strong>{catalog.modelsRoot || "Resolving local model directory…"}</strong>
        </div>
        <button
          type="button"
          className="feature-secondary-button"
          onClick={() => void onRefresh()}
        >
          Refresh
        </button>
      </div>

      {catalog.models.length > 0 ? (
        <div className="model-grid">
          {catalog.models.map((model) => {
            const isTransferring =
              model.state === "downloading" || model.state === "paused";
            const busyForModel = busy?.startsWith(`${model.id}:`) ?? false;

            return (
              <article
                className={`model-card ${model.active ? "active" : ""}`}
                key={model.id}
              >
                <div className="model-card-top">
                  <div className="model-orb" aria-hidden="true">
                    <AuraMark compact />
                  </div>
                  <div className="model-card-badges">
                    <span className="feature-badge">{model.generation}</span>
                    <span
                      className={`feature-badge model-state ${model.state} ${model.active ? "active" : ""}`}
                    >
                      {stateLabel(model)}
                    </span>
                  </div>
                </div>

                <div>
                  <h3>{model.name}</h3>
                  <strong>{model.subtitle}</strong>
                  <p>{model.description}</p>
                </div>

                <div className="model-facts">
                  <div>
                    <span>Size</span>
                    <strong>
                      {formatBytes(
                        model.totalBytes ?? model.estimatedSizeBytes,
                      )}
                    </strong>
                  </div>
                  <div>
                    <span>Source</span>
                    <strong>{model.sourceRepo ?? "Awaiting AURA release"}</strong>
                  </div>
                  <div>
                    <span>License</span>
                    <strong>{model.license ?? "To be announced"}</strong>
                  </div>
                </div>

                {(isTransferring ||
                  model.state === "failed" ||
                  model.bytesDownloaded > 0) && (
                  <div className="model-progress-block">
                    <div className="model-progress-copy">
                      <strong>
                        {model.state === "failed"
                          ? "Download interrupted"
                          : model.state === "paused"
                            ? "Download paused"
                            : model.state === "installed"
                              ? "Installed"
                              : "Downloading model"}
                      </strong>
                      <span>
                        {progressDetail(model)}
                        {model.bytesPerSecond
                          ? ` · ${formatBytes(model.bytesPerSecond)}/s`
                          : ""}
                      </span>
                    </div>
                    <div
                      className="model-progress-track"
                      role="progressbar"
                      aria-valuemin={0}
                      aria-valuemax={100}
                      aria-valuenow={Math.round(model.progressPercent)}
                    >
                      <span
                        style={{
                          width: `${Math.max(0, Math.min(100, model.progressPercent))}%`,
                        }}
                      />
                    </div>
                    <div className="model-progress-meta">
                      <span>{model.progressPercent.toFixed(1)}%</span>
                      <span>{model.currentFile ?? "Preparing files…"}</span>
                    </div>
                  </div>
                )}

                {model.error && (
                  <p className="model-error">{model.error}</p>
                )}

                {model.availabilityMessage && (
                  <p className="model-availability">
                    {model.availabilityMessage}
                  </p>
                )}

                <div className="model-actions">
                  {model.state === "unavailable" && (
                    <button
                      type="button"
                      className="feature-primary-button"
                      disabled
                    >
                      Download
                    </button>
                  )}

                  {model.state === "notInstalled" && (
                    <button
                      type="button"
                      className="feature-primary-button"
                      disabled={busyForModel || !model.downloadAvailable}
                      onClick={() => void run("download", model)}
                    >
                      {model.bytesDownloaded > 0 ? "Resume download" : "Download"}
                    </button>
                  )}

                  {model.state === "downloading" && (
                    <>
                      <button
                        type="button"
                        className="feature-secondary-button"
                        disabled={busyForModel}
                        onClick={() => void run("pause", model)}
                      >
                        Pause
                      </button>
                      <button
                        type="button"
                        className="feature-secondary-button danger"
                        disabled={busyForModel}
                        onClick={() => void run("cancel", model)}
                      >
                        Cancel
                      </button>
                    </>
                  )}

                  {model.state === "paused" && (
                    <>
                      <button
                        type="button"
                        className="feature-primary-button"
                        disabled={busyForModel}
                        onClick={() => void run("resume", model)}
                      >
                        Resume
                      </button>
                      <button
                        type="button"
                        className="feature-secondary-button danger"
                        disabled={busyForModel}
                        onClick={() => void run("cancel", model)}
                      >
                        Cancel
                      </button>
                    </>
                  )}

                  {model.state === "failed" && (
                    <>
                      {model.downloadAvailable && (
                        <button
                          type="button"
                          className="feature-primary-button"
                          disabled={busyForModel}
                          onClick={() => void run("download", model)}
                        >
                          Retry
                        </button>
                      )}
                      <button
                        type="button"
                        className="feature-secondary-button danger"
                        disabled={busyForModel}
                        onClick={() => void run("remove", model)}
                      >
                        Clear files
                      </button>
                    </>
                  )}

                  {model.state === "installed" && (
                    <>
                      {!model.active && (
                        <button
                          type="button"
                          className="feature-primary-button"
                          disabled={busyForModel}
                          onClick={() => void run("activate", model)}
                        >
                          Use model
                        </button>
                      )}
                      {model.active && (
                        <button
                          type="button"
                          className="feature-primary-button"
                          disabled
                        >
                          Active
                        </button>
                      )}
                      <button
                        type="button"
                        className="feature-secondary-button danger"
                        disabled={busyForModel}
                        onClick={() => void run("remove", model)}
                      >
                        Remove
                      </button>
                    </>
                  )}
                </div>

                <div className="model-meta">
                  <span>
                    {model.sourceRevision
                      ? `Revision · ${model.sourceRevision}`
                      : "AURA-managed model"}
                  </span>
                  <span>{stateLabel(model)}</span>
                </div>
              </article>
            );
          })}
        </div>
      ) : (
        <div className="feature-empty">Loading the local model catalog…</div>
      )}

      <div className="feature-note">
        <strong>AURA-1 is available now.</strong>
        <span>
          Its upstream runtime is Qwen/Qwen3-4B-Instruct-2507. AURA-2 remains
          visible in the manager but cannot be downloaded until its real
          checkpoint is defined; the app will not invent or silently substitute
          another model.
        </span>
      </div>

      {localError && <p className="model-error global">{localError}</p>}
    </section>
  );
}
