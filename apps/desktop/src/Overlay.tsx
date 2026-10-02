import { FormEvent, useEffect, useRef, useState } from "react";
import { hideOverlay, openMainWindow } from "./bridge/aura";
import { useAuraBridge } from "./bridge/useAuraBridge";
import {
  AuraMark,
  ShortcutKey,
  StatusPill,
} from "./design-system/components";

export default function Overlay() {
  const [command, setCommand] = useState("");
  const inputRef = useRef<HTMLInputElement>(null);
  const { status, activity, runtimeState, submitCommand } = useAuraBridge();

  useEffect(() => {
    const focusInput = () => inputRef.current?.focus();
    focusInput();

    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        void hideOverlay();
      }
    };

    window.addEventListener("keydown", onKeyDown);
    window.addEventListener("focus", focusInput);

    return () => {
      window.removeEventListener("keydown", onKeyDown);
      window.removeEventListener("focus", focusInput);
    };
  }, []);

  async function handleSubmit(event: FormEvent) {
    event.preventDefault();

    const value = command.trim();
    if (!value || runtimeState.paused) return;

    setCommand("");
    await submitCommand(value, "overlay");
  }

  return (
    <main className="overlay-root">
      <section className={`overlay-panel ${status.toLowerCase()}`}>
        <div className="overlay-top">
          <div className="overlay-brand">
            <AuraMark compact />
            <div>
              <strong>AURA</strong>
              <span>{runtimeState.paused ? "Paused" : activity}</span>
            </div>
          </div>

          <div className="overlay-actions">
            <StatusPill status={status} />
            <button
              className="overlay-open-full"
              type="button"
              onClick={() => void openMainWindow()}
            >
              Open app
            </button>
          </div>
        </div>

        <form className="overlay-command" onSubmit={handleSubmit}>
          <span className="overlay-spark" aria-hidden="true">✦</span>
          <input
            ref={inputRef}
            value={command}
            onChange={(event) => setCommand(event.target.value)}
            placeholder={
              runtimeState.paused
                ? "AURA is paused"
                : "What do you want to do?"
            }
            disabled={runtimeState.paused || status === "Working"}
            aria-label="AURA quick command"
          />
          <ShortcutKey>Enter</ShortcutKey>
        </form>

        <div className="overlay-hint">
          <span><ShortcutKey>Esc</ShortcutKey> close</span>
          <span><ShortcutKey>Ctrl</ShortcutKey> + <ShortcutKey>Shift</ShortcutKey> + <ShortcutKey>Space</ShortcutKey></span>
        </div>
      </section>
    </main>
  );
}
