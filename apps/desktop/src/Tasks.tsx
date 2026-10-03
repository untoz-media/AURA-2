import "./feature-pages.css";

export default function Tasks() {
  return (
    <section className="feature-page">
      <header className="feature-hero compact">
        <div>
          <span className="feature-kicker">AURA TASKS</span>
          <h2>Work that continues after the prompt.</h2>
          <p>
            Long-running actions, scheduled routines and completed work will
            live here as AURA moves toward agents and automation.
          </p>
        </div>
      </header>

      <div className="task-columns">
        <article className="task-column">
          <span>RUNNING</span>
          <div className="feature-empty">No active tasks.</div>
        </article>
        <article className="task-column">
          <span>SCHEDULED</span>
          <div className="feature-empty">No scheduled tasks.</div>
        </article>
        <article className="task-column">
          <span>RECENT</span>
          <div className="feature-empty">Completed task history will appear here.</div>
        </article>
      </div>

      <div className="feature-note">
        <strong>Prepared for M008.</strong>
        <span>
          The page is part of the permanent desktop navigation now, but no fake
          background task engine is exposed before the real multi-step action
          system exists.
        </span>
      </div>
    </section>
  );
}
