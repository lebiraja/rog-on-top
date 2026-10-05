# API (unix socket, JSON lines)
Socket: `$XDG_RUNTIME_DIR/rogd.sock` (0600). One request line → one response line.

| Request | Response |
|---|---|
| `{"cmd":"set_effect","effect":{"effect":"rainbow","period_ms":6000,"spread":1.0}}` | `{"status":"ok"}` |
| `{"cmd":"set_brightness","level":0-100}` | ok, or `{"status":"error","message":...}` |
| `{"cmd":"state"}` | `{"status":"state","effect":{...},"brightness":N}` |
| `{"cmd":"claude","event":"start|working|ask|done|end","session":"id","pid":N?,"transcript":"path"?}` | ok |

Effects: `static{colour}`, `breathe{colour,period_ms}`, `pulse{colour,period_ms}`, `rainbow{period_ms,spread?}`, `comet{colour,period_ms,tail?,floor?,tail_colour?}`. Colours are `#rrggbb`.

CLI: `rogctl effect <static|breathe|rainbow|comet> ...`, `rogctl brightness N`, `rogctl state`, `rogctl hook <event>`.

# HTTP API (127.0.0.1:7777, `/api/v1`)
Host must be `127.0.0.1:7777` or `localhost:7777`; a present `Origin` must be `http://127.0.0.1:7777` or `http://localhost:7777`, else 403. Bodies are JSON.

| Route | Body → Response |
|---|---|
| `GET /state` | → `{"status":"state","effect":{...},"brightness":N}` |
| `PUT /effect` | effect object (as above) → `{"status":"ok"}` |
| `PUT /brightness` | `{"level":0-100}` → ok / 400 error |
| `GET /frame` | → `{"device":"...","leds":[{"name","group","colour"}]}` current target colours incl. Claude overlay (UI polls at 10 Hz) |

Everything else serves the embedded UI (`web/dist`, SPA fallback). Build the UI before the release binary: `cd web && npm run build`.
