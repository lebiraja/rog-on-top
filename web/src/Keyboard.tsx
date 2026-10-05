import { BAR_LEDS, KEY_LEDS, layout, zoneOf } from "./zones";

const PAD = 0.2;
const BAR_GAP = 1.6;
const BAR_H = 0.8;
const LABEL_H = 1.1;
const VIEW_W = layout.width + PAD * 2;
const VIEW_H = layout.height + PAD + BAR_GAP + BAR_H + LABEL_H;
const ZONE_W = layout.width / 4;
/** bar1 is the right-hand LED in the firmware order, so draw them right to left. */
const BAR_DRAW_ORDER = [...BAR_LEDS].reverse();

interface Props {
  /** Live colour of each LED by name (key1..key4, bar1..bar6). */
  colours: Readonly<Record<string, string>>;
  selected: ReadonlySet<string>;
  onSelect: (leds: readonly string[], additive: boolean) => void;
}

function readable(hex: string): string {
  const n = Number.parseInt(hex.slice(1), 16);
  const lum = (0.299 * ((n >> 16) & 255) + 0.587 * ((n >> 8) & 255) + 0.114 * (n & 255)) / 255;
  return lum > 0.55 ? "#111" : "#eee";
}

export function Keyboard({ colours, selected, onSelect }: Props) {
  const activate = (leds: readonly string[]) => (e: React.MouseEvent | React.KeyboardEvent) => {
    if ("key" in e && e.key !== "Enter" && e.key !== " ") return;
    e.preventDefault();
    onSelect(leds, e.shiftKey || e.ctrlKey || e.metaKey);
  };

  return (
    <svg viewBox={`0 0 ${VIEW_W} ${VIEW_H}`} role="group" aria-label="Keyboard and lightbar" className="w-full select-none">
      <g transform={`translate(${PAD} ${PAD})`}>
        {KEY_LEDS.map((led, i) => (
          <g
            key={led}
            role="button"
            tabIndex={0}
            aria-pressed={selected.has(led)}
            aria-label={`Keyboard zone ${i + 1}`}
            onClick={activate([led])}
            onKeyDown={activate([led])}
            className="cursor-pointer outline-none"
          >
            <rect
              x={i * ZONE_W}
              y={-0.1}
              width={ZONE_W}
              height={layout.height + 0.1}
              fill={colours[led] ?? "#000"}
              fillOpacity={selected.has(led) ? 0.22 : 0.08}
              stroke={selected.has(led) ? "var(--color-rog)" : "transparent"}
              strokeWidth={0.08}
            />
            <text x={i * ZONE_W + ZONE_W / 2} y={layout.height + 0.55} textAnchor="middle" fontSize={0.5} fill="var(--color-mute)" letterSpacing={0.08}>
              ZONE {i + 1}
            </text>
          </g>
        ))}

        {layout.keys.map((k) => {
          const zone = zoneOf(k);
          const colour = colours[zone] ?? "#000000";
          const on = selected.has(zone);
          return (
            <g key={k.id} onClick={activate([zone])} className="cursor-pointer" aria-hidden="true">
              <rect
                className="key-cap"
                x={k.x}
                y={k.y}
                width={k.w}
                height={k.h}
                rx={0.14}
                fill={colour}
                fillOpacity={0.9}
                stroke={on ? "#fff" : "#000"}
                strokeOpacity={on ? 0.9 : 0.55}
                strokeWidth={on ? 0.07 : 0.05}
                style={{ filter: `drop-shadow(0 0 0.25px ${colour})` }}
              />
              <text x={k.x + k.w / 2} y={k.y + k.h / 2 + 0.16} textAnchor="middle" fontSize={k.h < 0.9 ? 0.34 : 0.4} fill={readable(colour)} fillOpacity={0.85}>
                {k.label}
              </text>
            </g>
          );
        })}

        <text x={0} y={layout.height + BAR_GAP - 0.2} fontSize={0.42} fill="var(--color-mute)" letterSpacing={0.08}>
          LIGHTBAR
        </text>
        {BAR_DRAW_ORDER.map((led, i) => {
          const w = layout.width / BAR_LEDS.length;
          const colour = colours[led] ?? "#000";
          const on = selected.has(led);
          return (
            <g
              key={led}
              role="button"
              tabIndex={0}
              aria-pressed={on}
              aria-label={`Lightbar LED ${BAR_LEDS.length - i}`}
              onClick={activate([led])}
              onKeyDown={activate([led])}
              className="cursor-pointer outline-none"
            >
              <rect
                x={i * w + 0.06}
                y={layout.height + BAR_GAP}
                width={w - 0.12}
                height={BAR_H}
                rx={0.1}
                fill={colour}
                stroke={on ? "#fff" : "#000"}
                strokeWidth={on ? 0.08 : 0.04}
                style={{ filter: `drop-shadow(0 0 0.3px ${colour})` }}
              />
            </g>
          );
        })}
      </g>
    </svg>
  );
}
