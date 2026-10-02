import { FormEvent, useState } from "react";
import {
  AuraMark,
  NavItem,
  SectionLabel,
  ShortcutKey,
  StatusPill,
  Surface,
  type AuraStatus,
} from "./design-system/components";

const modules = [
  { name: "Computer", description: "Windows control", milestone: "M003", glyph: "⌁" },
  { name: "Director", description: "OBS control", milestone: "M004", glyph: "◉" },
  { name: "Memory", description: "Local context", milestone: "M005", glyph: "◇" },
  { name: "Voice", description: "Natural interaction", milestone: "M006", glyph: "∿" },
  { name: "Vision", description: "Screen understanding", milestone: "M007", glyph: "◎" },
  { name: "Agents", description: "Multi-step actions", milestone: "M008", glyph: "✦" },
];

function App() {
  const [status, setStatus] = useState<AuraStatus>("Idle");
  const [command, setCommand] = useState("");
  const [lastAction, setLastAction] = useState(
    "Desktop foundation online. Action routing arrives in M003.",
  );

  function submitCommand(event: FormEvent) {
    event.preventDefault();
    const value = command.trim();

    if (!value) return;

    setStatus("Thinking");
    setLastAction(`Received: “${value}”`);
    setCommand("");

    window.setTimeout(() => {
      setStatus("Idle");
      setLastAction(
        "Command captured locally. System execution will be connected in M003.",
      );
    }, 650);
  }

  return (
    <main className="app-shell">
      <aside className="sidebar">
        <div className="brand-lockup">
          <AuraMark compact />
          <div className="brand-copy">
            <strong>AURA</strong>
            <span>2 · pre-Beta</span>
          </div>
        </div>

        <nav className="primary-nav" aria-label="Primary navigation">
          <NavItem active icon="⌂">Home</NavItem>
          <NavItem icon="✦">Actions</NavItem>
          <NavItem icon="↻">Automations</NavItem>
          <NavItem icon="◇">Memory</NavItem>
        </nav>

        <div className="sidebar-bottom">
          <NavItem icon="⚙">Settings</NavItem>
          <div className="local-badge">
            <span className="local-dot" />
            <span>Local-first</span>
          </div>
        </div>
      </aside>

      <section className="workspace">
        <header className="topbar" data-tauri-drag-region>
          <div>
            <span className="eyebrow">AURA-2 DESKTOP</span>
            <h1>Good evening.</h1>
          </div>
          <StatusPill status={status} />
        </header>

        <section className="hero">
          <div className={`aura-presence ${status.toLowerCase()}`}>
            <AuraMark />
          </div>

          <p className="hero-kicker">YOUR PC. NOW IT UNDERSTANDS YOU.</p>
          <h2>What do you want to do?</h2>

          <form className="command-bar" onSubmit={submitCommand}>
            <span className="command-spark" aria-hidden="true">✦</span>
            <input
              autoFocus
              value={command}
              onChange={(event) => setCommand(event.target.value)}
              placeholder="Ask AURA to do something on this computer…"
              aria-label="AURA command"
            />
            <ShortcutKey>Enter</ShortcutKey>
          </form>

          <div className="shortcut-hint">
            Press <ShortcutKey>Ctrl</ShortcutKey> + <ShortcutKey>Shift</ShortcutKey> +{" "}
            <ShortcutKey>Space</ShortcutKey> from anywhere
          </div>
        </section>

        <section className="lower-grid">
          <Surface className="activity-card">
            <SectionLabel trailing={<span className="activity-live-dot" />}>
              Current activity
            </SectionLabel>
            <p>{lastAction}</p>
          </Surface>

          <Surface className="modules-card">
            <SectionLabel trailing={<span>{modules.length} modules</span>}>
              AURA modules
            </SectionLabel>
            <div className="module-grid">
              {modules.map((module) => (
                <div className="module-tile" key={module.name}>
                  <span className="module-glyph" aria-hidden="true">{module.glyph}</span>
                  <div className="module-copy">
                    <strong>{module.name}</strong>
                    <span>{module.description}</span>
                  </div>
                  <small>{module.milestone}</small>
                </div>
              ))}
            </div>
          </Surface>
        </section>
      </section>
    </main>
  );
}

export default App;
