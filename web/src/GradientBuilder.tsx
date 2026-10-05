import { useState } from "react";
import { gradient } from "./colour";

const MIN_STOPS = 2;
const MAX_STOPS = 4;

interface Props {
  /** Colour the next added stop starts from. */
  current: string;
  selectionCount: number;
  onApply: (colours: string[]) => void;
}

export function GradientBuilder({ current, selectionCount, onApply }: Props) {
  const [stops, setStops] = useState<string[]>(["#ff1744", "#2f6bff"]);
  const preview = `linear-gradient(90deg, ${stops.join(", ")})`;

  const setStop = (i: number, colour: string) => setStops((s) => s.map((c, j) => (j === i ? colour : c)));

  return (
    <div className="flex flex-col gap-3">
      <div className="h-6 border border-line" style={{ background: preview }} />
      <div className="flex flex-wrap items-center gap-2">
        {stops.map((c, i) => (
          <input
            key={i}
            type="color"
            aria-label={`Gradient stop ${i + 1}`}
            value={c}
            onChange={(e) => setStop(i, e.target.value)}
            className="h-9 w-12"
          />
        ))}
        <button
          type="button"
          disabled={stops.length >= MAX_STOPS}
          onClick={() => setStops((s) => [...s, current])}
          className="h-9 border border-line px-3 text-sm disabled:opacity-30"
        >
          +
        </button>
        <button
          type="button"
          disabled={stops.length <= MIN_STOPS}
          onClick={() => setStops((s) => s.slice(0, -1))}
          className="h-9 border border-line px-3 text-sm disabled:opacity-30"
        >
          −
        </button>
      </div>
      <button
        type="button"
        disabled={selectionCount === 0}
        onClick={() => onApply(gradient(stops, selectionCount))}
        className="cut bg-rog px-3 py-2 text-xs font-bold uppercase tracking-widest text-white disabled:bg-line disabled:text-mute"
      >
        {selectionCount === 0 ? "Select LEDs first" : `Apply across ${selectionCount} LED${selectionCount === 1 ? "" : "s"}`}
      </button>
    </div>
  );
}
