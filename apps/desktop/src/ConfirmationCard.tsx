import type { PendingConfirmation } from "./bridge/types";

type Props = {
  confirmation: PendingConfirmation;
  compact?: boolean;
  onAllow: (id: string) => void;
  onCancel: (id: string) => void;
};

export default function ConfirmationCard({
  confirmation,
  compact = false,
  onAllow,
  onCancel,
}: Props) {
  return (
    <section className={`confirmation-card ${compact ? "compact" : ""}`}>
      <div className="confirmation-copy">
        <span className="confirmation-kicker">CONFIRM ACTION</span>
        <strong>{confirmation.command}</strong>
        <p>{confirmation.message}</p>
      </div>

      <div className="confirmation-actions">
        <button
          type="button"
          className="confirmation-button secondary"
          onClick={() => onCancel(confirmation.id)}
        >
          Cancel
        </button>
        <button
          type="button"
          className="confirmation-button primary"
          onClick={() => onAllow(confirmation.id)}
        >
          Allow
        </button>
      </div>
    </section>
  );
}
