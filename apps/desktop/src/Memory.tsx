import { useMemo, useState } from "react";
import type {
  MemoryCreateResult,
  MemoryRecord,
  MemorySnapshot,
} from "./bridge/types";
import "./memory.css";

type Props = {
  memory: MemorySnapshot;
  onRefresh: () => Promise<MemorySnapshot>;
  onCreate: (content: string) => Promise<MemoryCreateResult>;
  onDelete: (memoryId: string) => Promise<MemoryRecord>;
};

function formatMemoryDate(timestampMs: number) {
  if (!timestampMs) return "Unknown";

  return new Intl.DateTimeFormat(undefined, {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(new Date(timestampMs));
}

export default function Memory({
  memory,
  onRefresh,
  onCreate,
  onDelete,
}: Props) {
  const [draft, setDraft] = useState("");
  const [query, setQuery] = useState("");
  const [saving, setSaving] = useState(false);
  const [deletingId, setDeletingId] = useState<string | null>(null);
  const [localError, setLocalError] = useState<string | null>(null);

  const filtered = useMemo(() => {
    const normalized = query.trim().toLowerCase();
    if (!normalized) return memory.records;

    return memory.records.filter((record) =>
      record.content.toLowerCase().includes(normalized),
    );
  }, [memory.records, query]);

  async function handleCreate() {
    const content = draft.trim();
    if (!content) return;

    setSaving(true);
    setLocalError(null);
    try {
      await onCreate(content);
      setDraft("");
    } catch (error) {
      setLocalError(String(error));
    } finally {
      setSaving(false);
    }
  }

  async function handleDelete(record: MemoryRecord) {
    if (!window.confirm(`Forget “${record.content}”?`)) return;

    setDeletingId(record.id);
    setLocalError(null);
    try {
      await onDelete(record.id);
    } catch (error) {
      setLocalError(String(error));
    } finally {
      setDeletingId(null);
    }
  }

  return (
    <section className="memory-page">
      <div className="memory-hero">
        <div>
          <span className="memory-kicker">M005 · LOCAL CONTEXT</span>
          <h2>Memory</h2>
          <p>
            Explicit memories stored locally on this PC. AURA only saves entries
            here when you ask it to remember something or add it manually.
          </p>
        </div>
        <div className="memory-count">
          <strong>{memory.records.length}</strong>
          <span>saved {memory.records.length === 1 ? "memory" : "memories"}</span>
        </div>
      </div>

      <div className="memory-create-card">
        <div className="memory-create-copy">
          <strong>Add a memory</strong>
          <span>
            Keep it specific and useful. Duplicate text is refreshed instead of
            saved twice.
          </span>
        </div>
        <textarea
          value={draft}
          maxLength={2000}
          rows={3}
          placeholder="Example: My default browser is Brave."
          onChange={(event) => setDraft(event.target.value)}
        />
        <div className="memory-create-footer">
          <span>{draft.length}/2000</span>
          <button
            type="button"
            className="memory-primary-button"
            disabled={saving || !draft.trim()}
            onClick={() => void handleCreate()}
          >
            {saving ? "Saving…" : "Remember"}
          </button>
        </div>
      </div>

      <div className="memory-toolbar">
        <div>
          <strong>Saved memories</strong>
          <span>
            Search, inspect and remove anything AURA has stored explicitly.
          </span>
        </div>
        <div className="memory-toolbar-actions">
          <input
            type="search"
            value={query}
            placeholder="Search memory"
            aria-label="Search memory"
            onChange={(event) => setQuery(event.target.value)}
          />
          <button
            type="button"
            className="memory-secondary-button"
            onClick={() => void onRefresh()}
          >
            Refresh
          </button>
        </div>
      </div>

      {filtered.length > 0 ? (
        <div className="memory-list">
          {filtered.map((record) => (
            <article className="memory-record" key={record.id}>
              <div className="memory-record-main">
                <p>{record.content}</p>
                <div className="memory-record-meta">
                  <span>{record.source === "command" ? "Command" : "Manual"}</span>
                  <span>Updated {formatMemoryDate(record.updatedAtMs)}</span>
                  {record.createdAtMs !== record.updatedAtMs && (
                    <span>Created {formatMemoryDate(record.createdAtMs)}</span>
                  )}
                </div>
              </div>
              <button
                type="button"
                className="memory-delete-button"
                disabled={deletingId !== null}
                onClick={() => void handleDelete(record)}
              >
                {deletingId === record.id ? "Forgetting…" : "Forget"}
              </button>
            </article>
          ))}
        </div>
      ) : (
        <div className="memory-empty">
          {memory.records.length === 0
            ? "No explicit memories yet. Add one above or tell AURA “remember that…”."
            : "No saved memories match this search."}
        </div>
      )}

      <div className="memory-privacy-note">
        <strong>Local-first</strong>
        <span>
          M005.1 stores explicit memory in AURA&apos;s local app configuration.
          Automatic context capture is not enabled in this milestone.
        </span>
      </div>

      {localError && <p className="memory-error">{localError}</p>}
    </section>
  );
}
