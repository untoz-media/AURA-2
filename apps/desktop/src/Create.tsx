import { useMemo, useState } from "react";
import type {
  ImageGenerationRequest,
  ImageGenerationResult,
  ImageRuntimeStatus,
  ManagedRuntimeStatus,
  ModelCatalog,
  ModelStatus,
} from "./bridge/types";
import "./feature-pages.css";

type CreateMode = "image" | "video";
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
  imageRuntime: ImageRuntimeStatus;
  lastImage: ImageGenerationResult | null;
  onModelOperation: (
    operation: ModelOperation,
    modelId: string,
  ) => Promise<ModelCatalog>;
  onRuntimeAction: (
    action: "install" | "repair" | "remove",
  ) => Promise<ManagedRuntimeStatus>;
  onRuntimeRefresh: () => Promise<ImageRuntimeStatus>;
  onGenerateImage: (
    request: ImageGenerationRequest,
  ) => Promise<ImageGenerationResult>;
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

export default function Create({
  catalog,
  managedRuntime,
  imageRuntime,
  lastImage,
  onModelOperation,
  onRuntimeAction,
  onRuntimeRefresh,
  onGenerateImage,
}: Props) {
  const [mode, setMode] = useState<CreateMode>("image");
  const [prompt, setPrompt] = useState("");
  const [negativePrompt, setNegativePrompt] = useState("");
  const [aspectRatio, setAspectRatio] =
    useState<"square" | "landscape" | "portrait">("square");
  const [steps, setSteps] = useState(20);
  const [seed, setSeed] = useState("");
  const [busy, setBusy] = useState<string | null>(null);
  const [localError, setLocalError] = useState<string | null>(null);

  const model = useMemo(
    () => catalog.models.find((item) => item.id === "create-tiny-sd"),
    [catalog.models],
  );

  const runtimeBusy = [
    "preparing",
    "downloadingPython",
    "verifyingInstaller",
    "installingPython",
    "preparingPackages",
    "installingPackages",
    "verifying",
  ].includes(managedRuntime.state);

  const generationBusy =
    busy === "generate" ||
    imageRuntime.state === "loading" ||
    imageRuntime.state === "generating";

  const imageReady =
    mode === "image" &&
    model?.state === "installed" &&
    managedRuntime.state === "ready";

  async function runModel(operation: ModelOperation) {
    if (!model) return;
    if (
      operation === "remove" &&
      !window.confirm(
        "Remove the AURA Create image model from this computer? Generated images will not be deleted.",
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

  async function runRuntime(action: "install" | "repair") {
    setBusy(`runtime:${action}`);
    setLocalError(null);
    try {
      await onRuntimeAction(action);
      await onRuntimeRefresh().catch(() => undefined);
    } catch (error) {
      setLocalError(String(error));
    } finally {
      setBusy(null);
    }
  }

  async function generate() {
    if (!imageReady || !prompt.trim() || generationBusy) return;

    const parsedSeed = seed.trim() === "" ? undefined : Number(seed);
    if (
      parsedSeed !== undefined &&
      (!Number.isInteger(parsedSeed) ||
        parsedSeed < 0 ||
        parsedSeed > 4_294_967_295)
    ) {
      setLocalError("Seed must be a whole number between 0 and 4294967295.");
      return;
    }

    setBusy("generate");
    setLocalError(null);
    try {
      await onGenerateImage({
        prompt: prompt.trim(),
        negativePrompt: negativePrompt.trim() || null,
        aspectRatio,
        steps,
        seed: parsedSeed ?? null,
      });
    } catch (error) {
      setLocalError(String(error));
    } finally {
      setBusy(null);
    }
  }

  return (
    <section className="feature-page create-page">
      <header className="feature-hero compact">
        <div>
          <span className="feature-kicker">AURA CREATE</span>
          <h2>Create locally without leaving your assistant.</h2>
          <p>
            Generate images on your own computer with a dedicated local model.
            The model is downloaded once; generation then runs through AURA's
            managed AI runtime without uploading your prompt or output.
          </p>
        </div>
      </header>

      <div className="create-mode-switch" role="tablist" aria-label="Create mode">
        <button
          type="button"
          className={mode === "image" ? "active" : ""}
          onClick={() => setMode("image")}
        >
          Image
        </button>
        <button
          type="button"
          className={mode === "video" ? "active" : ""}
          onClick={() => setMode("video")}
        >
          Video
          <span className="create-mode-beta">Later</span>
        </button>
      </div>

      {mode === "image" ? (
        <>
          <article className="create-engine-card">
            <div className="create-engine-heading">
              <div>
                <span className="feature-kicker">LOCAL IMAGE ENGINE</span>
                <h3>{model?.name ?? "AURA Create · Image"}</h3>
                <p>
                  Lightweight local text-to-image generation. Model weights and
                  the Python runtime are managed independently so either can be
                  repaired without deleting your generated images.
                </p>
              </div>
              <span className={`feature-badge ${model?.state ?? "notInstalled"}`}>
                {modelStateLabel(model)}
              </span>
            </div>

            <div className="create-engine-meta">
              <div>
                <span>Model</span>
                <strong>{model?.sourceRepo ?? "segmind/tiny-sd"}</strong>
              </div>
              <div>
                <span>Download</span>
                <strong>{formatSize(model?.estimatedSizeBytes)}</strong>
              </div>
              <div>
                <span>Runtime</span>
                <strong>
                  {managedRuntime.state === "ready"
                    ? "Ready"
                    : managedRuntime.state === "needsRepair"
                      ? "Needs repair"
                      : managedRuntime.state === "notInstalled"
                        ? "Not installed"
                        : runtimeBusy
                          ? "Setting up"
                          : "Needs attention"}
                </strong>
              </div>
              <div>
                <span>Generation</span>
                <strong>
                  {imageRuntime.state === "generating"
                    ? "Generating"
                    : imageRuntime.cuda
                      ? imageRuntime.device ?? "CUDA"
                      : imageRuntime.device ?? "Local / not loaded"}
                </strong>
              </div>
            </div>

            {(model?.state === "downloading" || model?.state === "paused") && (
              <div className="create-download">
                <div className="model-progress-track">
                  <span
                    style={{
                      width: `${Math.max(0, Math.min(100, model.progressPercent))}%`,
                    }}
                  />
                </div>
                <small>
                  {model.progressPercent.toFixed(1)}%
                  {model.currentFile ? ` · ${model.currentFile}` : ""}
                </small>
              </div>
            )}

            <div className="create-engine-actions">
              {managedRuntime.state === "notInstalled" && (
                <button
                  type="button"
                  className="feature-primary-button"
                  disabled={busy !== null}
                  onClick={() => void runRuntime("install")}
                >
                  Install AURA Runtime
                </button>
              )}
              {(managedRuntime.state === "needsRepair" ||
                managedRuntime.state === "error") && (
                <button
                  type="button"
                  className="feature-primary-button"
                  disabled={busy !== null}
                  onClick={() => void runRuntime("repair")}
                >
                  Repair AURA Runtime
                </button>
              )}

              {model?.state === "notInstalled" && (
                <button
                  type="button"
                  className="feature-primary-button"
                  disabled={busy !== null}
                  onClick={() => void runModel("download")}
                >
                  {model.bytesDownloaded > 0
                    ? "Resume model download"
                    : "Download image model"}
                </button>
              )}
              {model?.state === "downloading" && (
                <>
                  <button
                    type="button"
                    className="feature-secondary-button"
                    onClick={() => void runModel("pause")}
                  >
                    Pause
                  </button>
                  <button
                    type="button"
                    className="feature-secondary-button"
                    onClick={() => void runModel("cancel")}
                  >
                    Cancel
                  </button>
                </>
              )}
              {model?.state === "paused" && (
                <>
                  <button
                    type="button"
                    className="feature-primary-button"
                    onClick={() => void runModel("resume")}
                  >
                    Resume
                  </button>
                  <button
                    type="button"
                    className="feature-secondary-button"
                    onClick={() => void runModel("cancel")}
                  >
                    Cancel
                  </button>
                </>
              )}
              {model?.state === "failed" && (
                <button
                  type="button"
                  className="feature-primary-button"
                  disabled={busy !== null}
                  onClick={() => void runModel("download")}
                >
                  Retry download
                </button>
              )}
              {model?.state === "installed" && (
                <button
                  type="button"
                  className="feature-secondary-button"
                  disabled={generationBusy || busy !== null}
                  onClick={() => void runModel("remove")}
                >
                  Remove model
                </button>
              )}

              <button
                type="button"
                className="feature-secondary-button"
                disabled={generationBusy}
                onClick={() => void onRuntimeRefresh()}
              >
                Refresh engine
              </button>
            </div>
          </article>

          <div className="create-grid">
            <article className="create-workspace">
              <div className="create-workspace-heading">
                <div>
                  <span className="feature-kicker">GENERATE</span>
                  <h3>Describe what you want to see.</h3>
                </div>
                <span
                  className={`feature-badge ${imageReady ? "installed" : "notInstalled"}`}
                >
                  {imageReady ? "Local ready" : "Setup required"}
                </span>
              </div>

              <label className="create-prompt">
                <span>Prompt</span>
                <textarea
                  value={prompt}
                  maxLength={1000}
                  rows={5}
                  placeholder="A cinematic photograph of..."
                  onChange={(event) => setPrompt(event.target.value)}
                  disabled={generationBusy}
                />
                <small>{prompt.length}/1000</small>
              </label>

              <label className="create-prompt create-negative-prompt">
                <span>Negative prompt <em>optional</em></span>
                <textarea
                  value={negativePrompt}
                  maxLength={1000}
                  rows={3}
                  placeholder="blurry, distorted, low detail..."
                  onChange={(event) => setNegativePrompt(event.target.value)}
                  disabled={generationBusy}
                />
              </label>

              <div className="create-control-grid">
                <div>
                  <span>Aspect ratio</span>
                  <div className="create-choice-row">
                    {(["square", "landscape", "portrait"] as const).map((ratio) => (
                      <button
                        key={ratio}
                        type="button"
                        className={aspectRatio === ratio ? "active" : ""}
                        disabled={generationBusy}
                        onClick={() => setAspectRatio(ratio)}
                      >
                        {ratio === "square"
                          ? "1:1"
                          : ratio === "landscape"
                            ? "5:3"
                            : "3:5"}
                      </button>
                    ))}
                  </div>
                </div>

                <label>
                  <span>Steps</span>
                  <div className="create-range-row">
                    <input
                      type="range"
                      min={4}
                      max={40}
                      value={steps}
                      disabled={generationBusy}
                      onChange={(event) => setSteps(Number(event.target.value))}
                    />
                    <strong>{steps}</strong>
                  </div>
                </label>

                <label>
                  <span>Seed <em>optional</em></span>
                  <input
                    className="create-seed-input"
                    inputMode="numeric"
                    value={seed}
                    disabled={generationBusy}
                    placeholder="Random"
                    onChange={(event) =>
                      setSeed(event.target.value.replace(/[^0-9]/g, "").slice(0, 10))
                    }
                  />
                </label>
              </div>

              <button
                type="button"
                className="feature-primary-button create-generate"
                disabled={!imageReady || !prompt.trim() || generationBusy}
                onClick={() => void generate()}
              >
                {imageRuntime.state === "loading"
                  ? "Loading local image model…"
                  : imageRuntime.state === "generating" || busy === "generate"
                    ? "Generating locally…"
                    : "Generate image"}
              </button>

              {!imageReady && (
                <p className="create-setup-hint">
                  Install/repair the AURA Runtime and download the image model
                  above before generating.
                </p>
              )}

              {localError && <p className="model-error">{localError}</p>}
              {imageRuntime.lastError && !localError && (
                <p className="model-error">{imageRuntime.lastError}</p>
              )}
            </article>

            <article className="create-preview">
              <div className="create-preview-heading">
                <div>
                  <span className="feature-kicker">OUTPUT</span>
                  <h3>Latest image</h3>
                </div>
                {lastImage && (
                  <span className="feature-badge installed">
                    {lastImage.width}×{lastImage.height}
                  </span>
                )}
              </div>

              {lastImage ? (
                <>
                  <div className="create-image-frame">
                    <img src={lastImage.dataUrl} alt={lastImage.prompt} />
                  </div>
                  <div className="create-output-meta">
                    <div>
                      <span>Seed</span>
                      <strong>{lastImage.seed}</strong>
                    </div>
                    <div>
                      <span>Device</span>
                      <strong>{lastImage.device ?? "Local"}</strong>
                    </div>
                    <div className="create-output-path">
                      <span>Saved locally</span>
                      <strong title={lastImage.path}>{lastImage.path}</strong>
                    </div>
                  </div>
                </>
              ) : (
                <div className="create-empty-preview">
                  <span>◈</span>
                  <strong>No image generated yet</strong>
                  <p>
                    Your first result will appear here and will also be saved
                    locally under Pictures → AURA Create when available.
                  </p>
                </div>
              )}
            </article>
          </div>

          <div className="feature-note">
            <strong>Private after setup</strong>
            <span>
              The model download requires internet once. Image inference uses
              only the installed local files; prompts and generated images are
              not sent through an AURA cloud generation service.
            </span>
          </div>
        </>
      ) : (
        <article className="create-video-planned">
          <span className="feature-kicker">AURA CREATE · VIDEO</span>
          <h3>Video generation is next, not simulated.</h3>
          <p>
            The Create workspace already separates image and video engines, but
            the Beta will not pretend video generation exists until a real,
            hardware-aware backend is connected and validated.
          </p>
          <span className="feature-badge">Planned</span>
        </article>
      )}
    </section>
  );
}
