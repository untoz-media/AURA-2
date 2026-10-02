import { FormEvent, useState } from "react";

type AuraStatus = "Idle" | "Listening" | "Thinking" | "Working" | "Waiting";

const modules = [
  { name: "Computer", description: "Windows control", milestone: "M003" },
  { name: "Director", description: "OBS control", milestone: "M004" },
  { name: "Memory", description: "Local context", milestone: "M005" },
  { name: "Voice", description: "Natural interaction", milestone: "M006" },
  { name: "Vision", description: "Screen understanding", milestone: "M007" },
  { name: "Agents", description: "Multi-step actions", milestone: "M008" },
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
    <main className="shell">
      <aside className="sidebar">
        <div className="brand">
          <div className="aura-mark">A</div>
          <div>
            <strong>AURA</strong>
            <span>2 · pre-Beta</span>
          </div>
        </div>

        <nav>
          <button className="nav-item active">Home</button>
          <button className="nav-item">Actions</button>
          <button className="nav-item">Automations</button>
          <button className="nav-item">Memory</button>
        </nav>

        <div className="sidebar-bottom">
          <button className="nav-item">Settings</button>
          <div className="local-badge">
            <span className="dot" />
            Local-first
          </div>
        </div>
      </aside>

      <section className="workspace">
        <header className="topbar" data-tauri-drag-region>
          <div>
            <span className="eyebrow">AURA-2 DESKTOP</span>
            <h1>Good evening.</h1>
          </div>
          <div className="status-pill">
            <span className={`status-dot ${status.toLowerCase()}`} />
            {status}
          </div>
        </header>

        <section className="hero">
          <div className="orb" aria-hidden="true">
            <div className="orb-core" />
          </div>

          <p className="hero-kicker">YOUR PC. NOW IT UNDERSTANDS YOU.</p>
          <h2>What do you want to do?</h2>

          <form className="command-bar" onSubmit={submitCommand}>
            <span className="spark">✦</span>
            <input
              autoFocus
              value={command}
              onChange={(event) => setCommand(event.target.value)}
              placeholder="Ask AURA to do something on this computer…"
              aria-label="AURA command"
            />
            <kbd>Enter</kbd>
          </form>

          <div className="shortcut">
            Press <kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>Space</kbd> from anywhere
          </div>
        </section>

        <section className="lower-grid">
          <article className="activity-card">
            <div className="card-heading">
              <span>Current activity</span>
              <span className="live-dot" />
            </div>
            <p>{lastAction}</p>
          </article>

          <article className="modules-card">
            <div className="card-heading">
              <span>AURA modules</span>
              <span>{modules.length}</span>
            </div>
            <div className="module-grid">
              {modules.map((module) => (
                <div className="module" key={module.name}>
                  <div>
                    <strong>{module.name}</strong>
                    <span>{module.description}</span>
                  </div>
                  <small>{module.milestone}</small>
                </div>
              ))}
            </div>
          </article>
        </section>
      </section>
    </main>
  );
}

export default App;
