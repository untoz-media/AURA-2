import { useState } from "react";
import "./feature-pages.css";

type CreateMode = "image" | "video";

export default function Create() {
  const [mode, setMode] = useState<CreateMode>("image");
  const [prompt, setPrompt] = useState("");

  return (
    <section className="feature-page">
      <header className="feature-hero compact">
        <div>
          <span className="feature-kicker">AURA CREATE</span>
          <h2>Create without leaving your assistant.</h2>
          <p>
            A shared workspace for local or connected image generation and,
            where supported, video generation.
          </p>
        </div>
      </header>

      <div className="create-workspace">
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
          </button>
        </div>

        <label className="create-prompt">
          <span>Prompt</span>
          <textarea
            value={prompt}
            rows={6}
            placeholder={
              mode === "image"
                ? "Describe the image you want AURA to create…"
                : "Describe the video you want AURA to create…"
            }
            onChange={(event) => setPrompt(event.target.value)}
          />
        </label>

        <div className="create-options">
          <div>
            <span>Mode</span>
            <strong>{mode === "image" ? "Image generation" : "Video generation"}</strong>
          </div>
          <div>
            <span>Execution</span>
            <strong>Engine not connected</strong>
          </div>
          <div>
            <span>Privacy</span>
            <strong>Local when supported</strong>
          </div>
        </div>

        <button
          type="button"
          className="feature-primary-button create-generate"
          disabled
        >
          Generate
        </button>
      </div>

      <div className="feature-note">
        <strong>{mode === "image" ? "Image comes first." : "Video is supported by the architecture."}</strong>
        <span>
          {mode === "image"
            ? "The next Create milestone will connect an actual generation engine and gallery. The UI is intentionally disabled until generation is real."
            : "Video models are much heavier, so AURA will support compatible local hardware and optional connected engines without making video-local a requirement."}
        </span>
      </div>
    </section>
  );
}
