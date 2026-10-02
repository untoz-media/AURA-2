import { FormEvent, useEffect, useState } from "react";
import { listenToOpenSettings } from "./bridge/aura";
import { useAuraBridge } from "./bridge/useAuraBridge";
import {
  AuraMark,
  NavItem,
  SectionLabel,
  ShortcutKey,
  StatusPill,
  Surface,
} from "./design-system/components";
import Settings, { type SettingsSection } from "./Settings";

type AppView = "home" | "settings";

const modules = [
  { name: "Computer", description: "Windows control", milestone: "M003", glyph: "⌁" },
  { name: "Director", description: "OBS control", milestone: "M004", glyph: "◉" },
  { name: "Memory", description: "Local context", milestone: "M005", glyph: "◇" },
  { name: "Voice", description: "Natural interaction", milestone: "M006", glyph: "∿" },
  { name: "Vision", description: "Screen understanding", milestone: "M007", glyph: "◎" },
  { name: "Agents", description: "Multi-step actions", milestone: "M008", glyph: "✦" },
];

function App() {
  const [command, setCommand] = useState("");
  const [view, setView] = useState<AppView>("home");
  const [settingsSection, setSettingsSection] = useState<SettingsSection>("general");
  const {
    status,
    activity,
    appStatus,
    runtimeState,
    permissionPolicy,
    bridgeError,
    submitCommand,
    setPaused,
    setBackgroundMode,
    setAutostart,
    setPermission,
    resetPermissions,
  } = useAuraBridge();

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let cancelled = false;

    listenToOpenSettings(() => {
      setSettingsSection("general");
      setView("settings");
    }).then((cleanup) => {
      if (cancelled) {
        cleanup();
      } else {
        unlisten = cleanup;
      }
    });

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);

  async function handleSubmit(event: FormEvent) {
    event.preventDefault();
    const value = command.trim();
    if (!value) return;

    setCommand("");
    await submitCommand(value);
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
          <NavItem
            active={view === "home"}
            icon="⌂"
            onClick={() => setView("home")}
          >
            Home
          </NavItem>
          <NavItem icon="✦">Actions</NavItem>
          <NavItem icon="↻">Automations</NavItem>
          <NavItem icon="◇">Memory</NavItem>
        </nav>

        <div className="sidebar-bottom">
          <NavItem
            active={view === "settings"}
            icon="⚙"
            onClick={() => setView("settings")}
          >
            Settings
          </NavItem>
          <div className="local-badge">
            <span className="local-dot" />
            <span className={runtimeState.paused ? "runtime-paused" : ""}>
              {runtimeState.paused
                ? "Paused"
                : appStatus?.localFirst === false
                  ? "Hybrid"
                  : "Local-first"}
            </span>
          </div>
        </div>
      </aside>

      <section className="workspace">
        <header className="topbar" data-tauri-drag-region>
          <div>
            <span className="eyebrow">
              {appStatus ? `${appStatus.name} · ${appStatus.stage}` : "AURA-2 DESKTOP"}
            </span>
            <h1>{view === "settings" ? "Control AURA." : "Good evening."}</h1>
          </div>
          <StatusPill status={status} />
        </header>

        {view === "home" ? (
          <>
            <section className="hero">
              <div className={`aura-presence ${status.toLowerCase()}`}>
                <AuraMark />
              </div>

              <p className="hero-kicker">YOUR PC. NOW IT UNDERSTANDS YOU.</p>
              <h2>What do you want to do?</h2>

              <form className="command-bar" onSubmit={handleSubmit}>
                <span className="command-spark" aria-hidden="true">✦</span>
                <input
                  autoFocus
                  value={command}
                  onChange={(event) => setCommand(event.target.value)}
                  placeholder={
                    runtimeState.paused
                      ? "AURA is paused…"
                      : "Ask AURA to do something on this computer…"
                  }
                  aria-label="AURA command"
                  disabled={status === "Working" || runtimeState.paused}
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
                <SectionLabel
                  trailing={
                    <span
                      className={bridgeError ? "activity-error-dot" : "activity-live-dot"}
                      aria-label={bridgeError ? "Bridge error" : "Bridge connected"}
                    />
                  }
                >
                  Current activity
                </SectionLabel>
                <p>{activity}</p>
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
          </>
        ) : (
          <Settings
            activeSection={settingsSection}
            onSectionChange={setSettingsSection}
            appStatus={appStatus}
            runtimeState={runtimeState}
            onPausedChange={setPaused}
            onBackgroundChange={setBackgroundMode}
            onAutostartChange={setAutostart}
            permissionPolicy={permissionPolicy}
            onPermissionChange={setPermission}
            onResetPermissions={resetPermissions}
          />
        )}
      </section>
    </main>
  );
}

export default App;
