import { AuraMark } from "./design-system/components";
import "./feature-pages.css";

const models = [
  {
    id: "aura-1",
    name: "AURA-1",
    subtitle: "Fast · Lightweight · Local",
    description:
      "The original AURA model. Designed for local conversations and lightweight computer assistance.",
    generation: "1st generation",
  },
  {
    id: "aura-2",
    name: "AURA-2",
    subtitle: "Personal Computer Assistant",
    description:
      "The next AURA model line, built for context, tool use and deeper computer assistance.",
    generation: "2nd generation",
  },
];

export default function Models() {
  return (
    <section className="feature-page">
      <header className="feature-hero">
        <div>
          <span className="feature-kicker">AURA MODEL MANAGER</span>
          <h2>Choose the intelligence that runs AURA.</h2>
          <p>
            Download local models once, switch between them inside the app and
            keep model files on this computer.
          </p>
        </div>
        <div className="feature-hero-mark">
          <AuraMark />
        </div>
      </header>

      <div className="model-grid">
        {models.map((model) => (
          <article className="model-card" key={model.id}>
            <div className="model-card-top">
              <div className="model-orb" aria-hidden="true">
                <AuraMark compact />
              </div>
              <span className="feature-badge">{model.generation}</span>
            </div>
            <div>
              <h3>{model.name}</h3>
              <strong>{model.subtitle}</strong>
              <p>{model.description}</p>
            </div>
            <div className="model-meta">
              <span>Local model</span>
              <span>Not installed</span>
            </div>
            <button
              type="button"
              className="feature-primary-button"
              disabled
              title="The download engine is the next Model Manager milestone."
            >
              Download
            </button>
          </article>
        ))}
      </div>

      <div className="feature-note">
        <strong>Model download engine is next.</strong>
        <span>
          This page is the new permanent home for model downloads, install
          progress, verification, activation, updates and removal. Download
          buttons remain disabled until the downloader is wired, so the UI never
          pretends a model was installed.
        </span>
      </div>
    </section>
  );
}
