import { useState } from "react";
import { kelvinToHex } from "./colour";

export const PALETTES: readonly { name: string; colours: readonly string[] }[] = [
  { name: "ROG", colours: ["#ff1744", "#ff6b00", "#ffffff", "#1a1a1a", "#7a0019"] },
  { name: "Neon", colours: ["#00ffd5", "#ff2bd6", "#7b2dff", "#c6ff00", "#2f6bff"] },
  { name: "Sunset", colours: ["#ff2a1f", "#ff7a00", "#ffb46b", "#ff3ea5", "#7a1fff"] },
  { name: "Ice", colours: ["#e8f4ff", "#8fd3ff", "#2f6bff", "#00ffd5", "#b9a3ff"] },
];

const RECENT_KEY = "rog-on-top.recent-colours";
const RECENT_MAX = 12;

export function loadRecents(): string[] {
  try {
    const parsed: unknown = JSON.parse(localStorage.getItem(RECENT_KEY) ?? "[]");
    return Array.isArray(parsed) ? parsed.filter((c): c is string => typeof c === "string").slice(0, RECENT_MAX) : [];
  } catch {
    return [];
  }
}

export function pushRecent(recents: readonly string[], colour: string): string[] {
  const next = [colour, ...recents.filter((c) => c !== colour)].slice(0, RECENT_MAX);
  try {
    localStorage.setItem(RECENT_KEY, JSON.stringify(next));
  } catch {
    // storage unavailable: recents just won't persist
  }
  return next;
}

function Swatch({ colour, onPick }: { colour: string; onPick: (c: string) => void }) {
  return (
    <button
      type="button"
      aria-label={`Use ${colour}`}
      title={colour}
      onClick={() => onPick(colour)}
      className="h-7 flex-1 border border-line hover:border-white"
      style={{ background: colour }}
    />
  );
}

interface Props {
  recents: readonly string[];
  onPick: (colour: string) => void;
}

export function ColourTools({ recents, onPick }: Props) {
  const [kelvin, setKelvin] = useState(4000);
  const warm = kelvinToHex(kelvin);

  return (
    <div className="flex flex-col gap-5">
      <section>
        <h3 className="mb-2 text-xs uppercase tracking-widest text-mute">White temperature {kelvin}K</h3>
        <div className="flex items-center gap-3">
          <input
            type="range"
            aria-label="White temperature"
            min={2000}
            max={9000}
            step={100}
            value={kelvin}
            onChange={(e) => {
              setKelvin(Number(e.target.value));
              onPick(kelvinToHex(Number(e.target.value)));
            }}
            style={{ "--track": `linear-gradient(90deg, ${kelvinToHex(2000)}, ${kelvinToHex(5500)}, ${kelvinToHex(9000)})` } as React.CSSProperties}
            className="min-w-0 flex-1"
          />
          <span className="h-7 w-7 shrink-0 border border-line" style={{ background: warm }} />
        </div>
      </section>

      <section>
        <h3 className="mb-2 text-xs uppercase tracking-widest text-mute">Recent</h3>
        <div className="flex min-h-7 flex-wrap gap-1">
          {recents.length === 0 && <span className="text-xs text-mute">Colours you use show up here</span>}
          {recents.map((c) => (
            <div key={c} className="flex w-7">
              <Swatch colour={c} onPick={onPick} />
            </div>
          ))}
        </div>
      </section>

      <section>
        <h3 className="mb-2 text-xs uppercase tracking-widest text-mute">Palettes</h3>
        <div className="flex flex-col gap-2">
          {PALETTES.map((p) => (
            <div key={p.name} className="flex items-center gap-2">
              <span className="w-14 shrink-0 text-xs uppercase text-mute">{p.name}</span>
              <div className="flex flex-1 gap-1">
                {p.colours.map((c) => (
                  <Swatch key={c} colour={c} onPick={onPick} />
                ))}
              </div>
            </div>
          ))}
        </div>
      </section>
    </div>
  );
}
