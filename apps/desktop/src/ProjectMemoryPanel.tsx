import { FormEvent, useMemo, useState } from "react";
import type {
  ProjectMemory,
  ProjectMemorySnapshot,
  SaveProjectRequest,
  UserRoutine,
} from "./bridge/types";

type Props = {
  snapshot: ProjectMemorySnapshot;
  routines: UserRoutine[];
  onRefresh: () => Promise<ProjectMemorySnapshot>;
  onSave: (request: SaveProjectRequest) => Promise<ProjectMemory>;
  onDelete: (projectId: string) => Promise<void>;
  onSetActive: (projectId?: string) => Promise<ProjectMemorySnapshot>;
};

export default function ProjectMemoryPanel({
  snapshot,
  routines,
  onRefresh,
  onSave,
  onDelete,
  onSetActive,
}: Props) {
  const [editingId, setEditingId] = useState<string | undefined>();
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [aliases, setAliases] = useState("");
  const [notes, setNotes] = useState("");
  const [routineIds, setRoutineIds] = useState<string[]>([]);
  const [busy, setBusy] = useState(false);

  const activeProject = useMemo(
    () =>
      snapshot.projects.find((project) => project.id === snapshot.activeProjectId) ??
      null,
    [snapshot],
  );

  function resetEditor() {
    setEditingId(undefined);
    setName("");
    setDescription("");
    setAliases("");
    setNotes("");
    setRoutineIds([]);
  }

  function editProject(project: ProjectMemory) {
    setEditingId(project.id);
    setName(project.name);
    setDescription(project.description);
    setAliases(project.aliases.join(", "));
    setNotes(project.notes.join("\n"));
    setRoutineIds(project.routineIds);
  }

  async function submit(event: FormEvent) {
    event.preventDefault();
    if (!name.trim()) return;

    setBusy(true);
    try {
      await onSave({
        id: editingId,
        name: name.trim(),
        description: description.trim(),
        aliases: aliases
          .split(",")
          .map((value) => value.trim())
          .filter(Boolean),
        notes: notes
          .split("\n")
          .map((value) => value.trim())
          .filter(Boolean),
        routineIds,
      });
      resetEditor();
    } finally {
      setBusy(false);
    }
  }

  return (
    <section className="project-memory-section">
      <div className="project-memory-heading">
        <div>
          <span className="memory-kicker">M005.6 · PROJECT MEMORY</span>
          <h3>Projects</h3>
          <p>
            Keep project-specific context separate from your general AURA memory.
          </p>
        </div>
        <button
          type="button"
          className="memory-secondary-button"
          onClick={() => void onRefresh()}
          disabled={busy}
        >
          Refresh
        </button>
      </div>

      <div className="project-active-card">
        <span>ACTIVE PROJECT</span>
        <strong>{activeProject?.name ?? "None"}</strong>
        <p>
          {activeProject?.description ||
            "Choose a project to include its context in local AI conversations."}
        </p>
        {activeProject && (
          <button
            type="button"
            className="memory-secondary-button"
            onClick={() => void onSetActive(undefined)}
            disabled={busy}
          >
            Clear active project
          </button>
        )}
      </div>

      <div className="project-memory-grid">
        <form className="project-editor-card" onSubmit={submit}>
          <div className="project-card-heading">
            <strong>{editingId ? "Edit project" : "New project"}</strong>
            {editingId && (
              <button type="button" onClick={resetEditor}>
                Cancel
              </button>
            )}
          </div>

          <label>
            <span>Name</span>
            <input
              value={name}
              maxLength={80}
              placeholder="Artemis Documentary"
              onChange={(event) => setName(event.target.value)}
            />
          </label>

          <label>
            <span>Description</span>
            <textarea
              value={description}
              maxLength={600}
              rows={3}
              placeholder="Untoz+ documentary about Artemis II."
              onChange={(event) => setDescription(event.target.value)}
            />
          </label>

          <label>
            <span>Aliases</span>
            <input
              value={aliases}
              placeholder="Artemis 2, Moon documentary"
              onChange={(event) => setAliases(event.target.value)}
            />
          </label>

          <label>
            <span>Notes · one per line</span>
            <textarea
              value={notes}
              rows={5}
              placeholder={"Premiere project is the main edit\nTarget runtime: 60–90 min\nPremium 4K"}
              onChange={(event) => setNotes(event.target.value)}
            />
          </label>

          <div className="project-routine-links">
            <span>Linked routines</span>
            {routines.length === 0 ? (
              <small>No routines available yet.</small>
            ) : (
              routines.map((routine) => (
                <label key={routine.id}>
                  <input
                    type="checkbox"
                    checked={routineIds.includes(routine.id)}
                    onChange={(event) =>
                      setRoutineIds((current) =>
                        event.target.checked
                          ? [...current, routine.id]
                          : current.filter((id) => id !== routine.id),
                      )
                    }
                  />
                  <span>{routine.name}</span>
                </label>
              ))
            )}
          </div>

          <button
            type="submit"
            className="memory-primary-button"
            disabled={busy || !name.trim()}
          >
            {editingId ? "Save changes" : "Create project"}
          </button>
        </form>

        <div className="project-list-card">
          <div className="project-card-heading">
            <strong>Saved projects</strong>
            <span>{snapshot.projects.length}</span>
          </div>

          {snapshot.projects.length === 0 ? (
            <div className="memory-empty">No project memory yet.</div>
          ) : (
            <div className="project-list">
              {snapshot.projects.map((project) => {
                const active = project.id === snapshot.activeProjectId;
                return (
                  <article className={active ? "active" : ""} key={project.id}>
                    <div>
                      <span>{active ? "ACTIVE" : "PROJECT"}</span>
                      <strong>{project.name}</strong>
                      <p>{project.description || "No description"}</p>
                      <small>
                        {project.notes.length} notes · {project.routineIds.length} routines
                      </small>
                    </div>
                    <div className="project-actions">
                      {!active && (
                        <button
                          type="button"
                          onClick={() => void onSetActive(project.id)}
                          disabled={busy}
                        >
                          Use
                        </button>
                      )}
                      <button
                        type="button"
                        onClick={() => editProject(project)}
                        disabled={busy}
                      >
                        Edit
                      </button>
                      <button
                        type="button"
                        onClick={() => void onDelete(project.id)}
                        disabled={busy}
                      >
                        Delete
                      </button>
                    </div>
                  </article>
                );
              })}
            </div>
          )}
        </div>
      </div>
    </section>
  );
}
