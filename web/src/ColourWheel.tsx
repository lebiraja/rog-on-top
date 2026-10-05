import { useEffect, useRef, useState } from "react";
import { hexToHsv, hexToRgb, hsvToHex, isHex, rgbToHex, type Hsv } from "./colour";

const SIZE = 208;

interface Props {
  value: string;
  onChange: (hex: string) => void;
}

export function ColourWheel({ value, onChange }: Props) {
  const [hsv, setHsv] = useState<Hsv>(() => hexToHsv(value));
  const [text, setText] = useState(value);
  const wheel = useRef<HTMLDivElement>(null);

  // Follow outside changes (preset applied, swatch clicked) without losing hue at zero saturation.
  useEffect(() => {
    if (isHex(value) && hsvToHex(hsv) !== value.toLowerCase()) setHsv(hexToHsv(value));
    setText(value);
    // eslint-disable-next-line react-hooks/exhaustive-deps -- only react to the external value
  }, [value]);

  const commit = (next: Hsv) => {
    setHsv(next);
    onChange(hsvToHex(next));
  };

  const pick = (e: React.PointerEvent) => {
    const box = wheel.current?.getBoundingClientRect();
    if (!box) return;
    const dx = e.clientX - (box.left + box.width / 2);
    const dy = e.clientY - (box.top + box.height / 2);
    const r = Math.min(1, Math.hypot(dx, dy) / (box.width / 2));
    const h = ((Math.atan2(dy, dx) * 180) / Math.PI + 360 + 90) % 360;
    commit({ ...hsv, h, s: r });
  };

  const angle = ((hsv.h - 90) * Math.PI) / 180;
  const knobX = SIZE / 2 + Math.cos(angle) * hsv.s * (SIZE / 2);
  const knobY = SIZE / 2 + Math.sin(angle) * hsv.s * (SIZE / 2);
  const [r, g, b] = isHex(value) ? hexToRgb(value) : [0, 0, 0];
  const setChannel = (i: 0 | 1 | 2, n: number) => {
    const next: [number, number, number] = [r, g, b];
    next[i] = Math.min(255, Math.max(0, Math.round(n) || 0));
    onChange(rgbToHex(...next));
  };

  return (
    <div className="flex flex-col items-center gap-4">
      <div
        ref={wheel}
        role="slider"
        aria-label="Colour wheel"
        aria-valuetext={value}
        aria-valuenow={Math.round(hsv.h)}
        aria-valuemin={0}
        aria-valuemax={360}
        tabIndex={0}
        onPointerDown={(e) => {
          e.currentTarget.setPointerCapture(e.pointerId);
          pick(e);
        }}
        onPointerMove={(e) => e.buttons === 1 && pick(e)}
        onKeyDown={(e) => {
          const step = e.shiftKey ? 15 : 3;
          if (e.key === "ArrowRight") commit({ ...hsv, h: (hsv.h + step) % 360 });
          if (e.key === "ArrowLeft") commit({ ...hsv, h: (hsv.h - step + 360) % 360 });
          if (e.key === "ArrowUp") commit({ ...hsv, s: Math.min(1, hsv.s + 0.04) });
          if (e.key === "ArrowDown") commit({ ...hsv, s: Math.max(0, hsv.s - 0.04) });
        }}
        className="relative shrink-0 touch-none cursor-crosshair rounded-full"
        style={{
          width: SIZE,
          height: SIZE,
          background: `radial-gradient(circle, #fff 0%, transparent 70%), conic-gradient(from 0deg, #f00, #ff0, #0f0, #0ff, #00f, #f0f, #f00)`,
          filter: `brightness(${0.25 + hsv.v * 0.75})`,
        }}
      >
        <div
          className="pointer-events-none absolute h-5 w-5 -translate-x-1/2 -translate-y-1/2 rounded-full border-2 border-white"
          style={{ left: knobX, top: knobY, background: value, boxShadow: "0 0 0 2px #000" }}
        />
      </div>

      <label className="flex w-full items-center gap-3 text-xs uppercase tracking-widest text-mute">
        Value
        <input
          type="range"
          aria-label="Colour value"
          min={0}
          max={100}
          value={Math.round(hsv.v * 100)}
          onChange={(e) => commit({ ...hsv, v: Number(e.target.value) / 100 })}
          style={{ "--track": `linear-gradient(90deg, #000, ${hsvToHex({ ...hsv, v: 1 })})` } as React.CSSProperties}
          className="min-w-0 flex-1"
        />
      </label>

      <div className="grid w-full grid-cols-[1fr_repeat(3,3.5rem)] gap-2">
        <input
          type="text"
          aria-label="Hex colour"
          value={text}
          maxLength={7}
          spellCheck={false}
          onChange={(e) => {
            setText(e.target.value);
            if (isHex(e.target.value)) onChange(e.target.value.toLowerCase());
          }}
          className="min-w-0 border border-line bg-panel px-2 py-2 text-sm uppercase"
        />
        {([r, g, b] as const).map((n, i) => (
          <input
            key={i}
            type="number"
            aria-label={["Red", "Green", "Blue"][i]}
            min={0}
            max={255}
            value={n}
            onChange={(e) => setChannel(i as 0 | 1 | 2, Number(e.target.value))}
            className="min-w-0 border border-line bg-panel px-1 py-2 text-center text-sm"
          />
        ))}
      </div>
    </div>
  );
}
