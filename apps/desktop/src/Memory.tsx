import { useMemo, useState } from "react";
import type {
  MemoryCreateResult,
  MemoryRecord,
  MemorySnapshot,
} from "./bridge/types";
import ProjectMemoryPanel from "./ProjectMemoryPanel";
import type { ProjectMemory, ProjectMemorySnapshot, SaveProjectRequest, UserRoutine } from "./bridge/types";
import "./memory.css";

type Props = {
  memory: MemorySnapshot;
  onRefresh: () => Promise<MemorySnapshot>;
  onCreate: (content: string) => Promise<MemoryCreateResult>;
  onUpdate: (memoryId: string, content: string) => Promise<MemoryRecord>;
  onDelete: (memoryId: string) => Promise<MemoryRecord>;
  projectMemory: ProjectMemorySnapshot;
  routines: UserRoutine[];
  onProjectRefresh: () => Promise<ProjectMemorySnapshot>;
  onProjectSave: (request: SaveProjectRequest) => Promise<ProjectMemory>;
  onProjectDelete: (projectId: string) => Promise<void>;
  onProjectSetActive: (projectId?: string) => Promise<ProjectMemorySnapshot>;
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
  onUpdate,
  onDelete,
  projectMemory,
  routines,
  onProjectRefresh,
  onProjectSave,
  onProjectDelete,
  onProjectSetActive,
}: Props) {
  const [draft, setDraft] = useState("");
  const [query, setQuery] = useState("");
  const [saving, setSaving] = useState(false);
  const [deletingId, setDeletingId] = useState<string | null>(null);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editDraft, setEditDraft] = useState("");
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

  async function handleUpdate(record: MemoryRecord) {
    const content = editDraft.trim();
    if (!content) return;

    setSaving(true);
    setLocalError(null);
    try {
      await onUpdate(record.id, content);
      setEditingId(null);
      setEditDraft("");
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
          <span className="memory-kicker">LOCAL MEMORY</span>
          <h2>Memory</h2>
          <p>
            Explicit memories stored locally on this PC. AURA only writes this
            list when you ask it to remember something or add it manually.
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
            Search, edit and remove anything AURA has stored explicitly.
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
                {editingId === record.id ? (
                  <textarea
                    value={editDraft}
                    maxLength={2000}
                    rows={3}
                    aria-label="Edit memory"
                    onChange={(event) => setEditDraft(event.target.value)}
                  />
                ) : (
                  <p>{record.content}</p>
                )}
                <div className="memory-record-meta">
                  <span>{record.source === "command" ? "Command" : record.source === "ui" ? "Manual" : record.source}</span>
                  <span>Updated {formatMemoryDate(record.updatedAtMs)}</span>
                  {record.createdAtMs !== record.updatedAtMs && (
                    <span>Created {formatMemoryDate(record.createdAtMs)}</span>
                  )}
                </div>
              </div>
              <div className="memory-record-actions">
                {editingId === record.id ? (
                  <>
                    <button
                      type="button"
                      className="memory-primary-button"
                      disabled={saving || !editDraft.trim()}
                      onClick={() => void handleUpdate(record)}
                    >
                      {saving ? "Saving…" : "Save"}
                    </button>
                    <button
                      type="button"
                      className="memory-secondary-button"
                      disabled={saving}
                      onClick={() => {
                        setEditingId(null);
                        setEditDraft("");
                      }}
                    >
                      Cancel
                    </button>
                  </>
                ) : (
                  <button
                    type="button"
                    className="memory-secondary-button"
                    disabled={deletingId !== null || saving}
                    onClick={() => {
                      setEditingId(record.id);
                      setEditDraft(record.content);
                      setLocalError(null);
                    }}
                  >
                    Edit
                  </button>
                )}
                <button
                  type="button"
                  className="memory-delete-button"
                  disabled={deletingId !== null || saving}
                  onClick={() => void handleDelete(record)}
                >
                  {deletingId === record.id ? "Forgetting…" : "Forget"}
                </button>
              </div>
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

      <ProjectMemoryPanel
        snapshot={projectMemory}
        routines={routines}
        onRefresh={onProjectRefresh}
        onSave={onProjectSave}
        onDelete={onProjectDelete}
        onSetActive={onProjectSetActive}
      />

      <div className="memory-privacy-note">
        <strong>Local-first</strong>
        <span>
          Explicit memories stay in AURA&apos;s local configuration and are not
          uploaded. Chat can also see metadata for the current turn: the
          foreground app, recent file names, and the active project. That
          context is not written into this list and does not include file
          contents, screenshots, or clipboard text.
        </span>
      </div>

      {localError && <p className="memory-error">{localError}</p>}
    </section>
  );
}
