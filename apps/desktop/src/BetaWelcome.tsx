import type { AppStatus, BetaStatus } from "./bridge/types";
import "./beta.css";

type Props = {
  status: BetaStatus;
  appStatus: AppStatus | null;
  onComplete: () => Promise<void>;
  onReviewPermissions: () => void;
};

export default function BetaWelcome({
  status,
  appStatus,
  onComplete,
  onReviewPermissions,
}: Props) {
  return (
    <div className="beta-welcome-backdrop" role="dialog" aria-modal="true">
      <section className="beta-welcome">
        <div className="beta-welcome-mark">AURA-2</div>
        <span className="feature-kicker">PUBLIC BETA</span>
        <h2>Your PC. Now it understands you.</h2>
        <p className="beta-welcome-intro">
          AURA-2 Beta is local-first and action-oriented. Voice, Vision, Memory,
          Computer Control and Agents stay behind explicit permission rules.
        </p>

        {status.previousSessionUnclean && (
          <div className="beta-recovery-note">
            <strong>AURA recovered from an unclean previous session.</strong>
            <span>
              No crash report was uploaded. You can export local diagnostics
              later from Settings if you want to inspect or share them.
            </span>
          </div>
        )}

        <div className="beta-welcome-grid">
          <article>
            <span>01</span>
            <div>
              <strong>Local-first</strong>
              <p>
                Your assistant models and supported voice/vision processing run
                locally after you explicitly install them.
              </p>
            </div>
          </article>
          <article>
            <span>02</span>
            <div>
              <strong>Permissions stay in control</strong>
              <p>
                Read, Act, Modify, Sensitive and Destructive actions follow the
                policy you choose. Sensitive actions are never permanently
                allowed.
              </p>
            </div>
          </article>
          <article>
            <span>03</span>
            <div>
              <strong>No automatic telemetry</strong>
              <p>
                This Beta does not upload analytics or crash reports. Diagnostics
                are generated locally only when you ask.
              </p>
            </div>
          </article>
          <article>
            <span>04</span>
            <div>
              <strong>Beta software</strong>
              <p>
                Features, models and workflows can still change. Review every
                Agent plan before approving higher-risk actions.
              </p>
            </div>
          </article>
        </div>

        <div className="beta-welcome-actions">
          <button
            type="button"
            className="feature-secondary-button"
            onClick={onReviewPermissions}
          >
            Review permissions
          </button>
          <button
            type="button"
            className="feature-primary-button"
            onClick={() => void onComplete()}
          >
            Enter AURA-2 Beta
          </button>
        </div>

        <small className="beta-welcome-version">
          {appStatus?.version ?? "AURA-2 Beta"} · telemetry off · local
          diagnostics only
        </small>
      </section>
    </div>
  );
}
