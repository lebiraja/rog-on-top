import { useState } from "react";
import type { Preset } from "./api";

interface Props {
  presets: readonly Preset[];
  defaultId: string | null;
  onSave: (name: string) => void;
  onApply: (id: string) => void;
  onToggleFavourite: (preset: Preset) => void;
  onRename: (id: string, name: string) => void;
  onDelete: (id: string) => void;
  onSetDefault: (id: string | null) => void;
}

const MAX_NAME = 40;

function Swatches({ preset }: { preset: Preset }) {
  const e = preset.look.effect;
  const base = e.effect === "rainbow" ? null : e.colour;
  const painted = Object.values(preset.look.paint).slice(0, 6);
  const background = base
    ? painted.length ? `linear-gradient(90deg, ${[base, ...painted].join(", ")})` : base
    : "linear-gradient(90deg,#f00,#ff0,#0f0,#0ff,#00f,#f0f)";
  return <span className="h-8 w-14 shrink-0 border border-line" style={{ background }} aria-hidden="true" />;
}

export function PresetPanel({ presets, defaultId, onSave, onApply, onToggleFavourite, onRename, onDelete, onSetDefault }: Props) {
  const [name, setName] = useState("");
  const [editing, setEditing] = useState<string | null>(null);
  const [draft, setDraft] = useState("");
  const sorted = [...presets].sort((a, b) => Number(b.favourite) - Number(a.favourite) || a.name.localeCompare(b.name));

  const submit = (e: React.FormEvent) => {
    e.preventDefault();
    const trimmed = name.trim();
    if (!trimmed) return;
    onSave(trimmed);
    setName("");
  };

  return (
    <div className="flex flex-col gap-4">
      <form onSubmit={submit} className="flex gap-2">
        <input
          type="text"
          aria-label="Preset name"
          placeholder="Name this look"
          value={name}
          maxLength={MAX_NAME}
          onChange={(e) => setName(e.target.value)}
          className="min-w-0 flex-1 border border-line bg-panel px-3 py-2 text-sm"
        />
        <button type="submit" disabled={!name.trim()} className="cut bg-rog px-4 text-xs font-bold uppercase tracking-widest text-white disabled:bg-line disabled:text-mute">
          Save
        </button>
      </form>

      {sorted.length === 0 && <p className="text-sm text-mute">No presets yet. Set up a look, name it, save it.</p>}

      <ul className="flex flex-col">
        {sorted.map((p) => (
          <li key={p.id} className="flex items-center gap-2 border-b border-line py-2">
            <button
              type="button"
              aria-label={p.favourite ? `Unfavourite ${p.name}` : `Favourite ${p.name}`}
              aria-pressed={p.favourite}
              onClick={() => onToggleFavourite(p)}
              className={`w-7 text-lg ${p.favourite ? "text-rog" : "text-mute hover:text-white"}`}
            >
              {p.favourite ? "★" : "☆"}
            </button>
            <Swatches preset={p} />
            {editing === p.id ? (
              <form
                className="flex min-w-0 flex-1 gap-1"
                onSubmit={(e) => {
                  e.preventDefault();
                  if (draft.trim()) onRename(p.id, draft.trim());
                  setEditing(null);
                }}
              >
                <input
                  autoFocus
                  aria-label={`Rename ${p.name}`}
                  value={draft}
                  maxLength={MAX_NAME}
                  onChange={(e) => setDraft(e.target.value)}
                  onBlur={() => setEditing(null)}
                  className="min-w-0 flex-1 border border-line bg-panel px-2 py-1 text-sm"
                />
              </form>
            ) : (
              <button type="button" onClick={() => onApply(p.id)} className="min-w-0 flex-1 truncate text-left text-sm uppercase tracking-wide hover:text-rog">
                {p.name}
                {defaultId === p.id && <span className="ml-2 text-[10px] tracking-widest text-rog">BOOT</span>}
              </button>
            )}
            <button
              type="button"
              onClick={() => onSetDefault(defaultId === p.id ? null : p.id)}
              className="px-1 text-[10px] uppercase tracking-widest text-mute hover:text-white"
            >
              {defaultId === p.id ? "Unboot" : "Boot"}
            </button>
            <button
              type="button"
              onClick={() => {
                setDraft(p.name);
                setEditing(p.id);
              }}
              className="px-1 text-[10px] uppercase tracking-widest text-mute hover:text-white"
            >
              Rename
            </button>
            <button type="button" onClick={() => onDelete(p.id)} className="px-1 text-[10px] uppercase tracking-widest text-mute hover:text-rog">
              Delete
            </button>
          </li>
        ))}
      </ul>
    </div>
  );
}
