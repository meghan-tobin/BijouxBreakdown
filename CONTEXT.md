# BijouxBreakdown — Project Context

A local jewelry cost-tracking tool. Built as a single `index.html` served by a tiny Node/Express server. All data lives in `localStorage` under the key `bijoux_v1`.

## Running it

```bash
npm install    # first time only
npm start      # serves at http://localhost:3000
```

## File structure

```
BijouxBreakdown/
  index.html      — the entire app (HTML + CSS + JS, ~1600 lines)
  server.js       — Express static file server, port 3000
  package.json    — single dependency: express
  breakdown/      — original incomplete Rust CLI (not used, kept for reference)
```

## Data model (`localStorage` key: `bijoux_v1`)

```js
S = {
  configs: [{ id, name, colorIdx, chipSalt, supplies: [{ id, name, cost, quantity, unit }], locked }],
  items:   [{ id, name, configId, usages: [{ supplyId, amount, unit }],
              groupId, x, y, rx, ry, width, height }],
  groups:  [{ id, name, colorIdx, x, y, width, height }]
}
```

- `x/y` — absolute position on the board (ungrouped items)
- `rx/ry` — position relative to the group container (grouped items)
- `groupId: null` means the item is loose on the board
- `colorIdx` — index into `BLOCK_COLORS` palette for config panels and groups
- `chipSalt` — random string mixed into supply chip color hashing; changing it reshuffles chip colors

## Units

Supported: `yards`, `feet`, `in`, `cms`, `count`, `n/a`

Length units (`yards/feet/in/cms`) are interconvertible at item-creation time — a supply saved in yards can have usage entered in inches. Conversion goes through inches as a base. `count` and `n/a` are not convertible.

Cost formula: `(supply.cost / supply.quantity) * convertedAmount`

## UI sections

**Header** — title + reset (↺) and help (?) buttons. No browser `alert()` or `prompt()` anywhere — all dialogs are custom in-app modals.

**Configure Supplies** (`+ Configure Supplies` button) — creates a config panel. Each panel gets a randomized color on creation (no two in a row). Controls while editing:
- `✦` — shuffles the panel color and all supply chip colors for that config (name is preserved)
- `Save` — locks the config
- `Delete` — removes it (warns if items use it)
- Small color dot (bottom-right) — cycles through the 7-color palette on click

Locked configs show Edit / Delete. The config name is flushed to state before any re-render (submit supply, delete supply, cycle color, shuffle) so it's never lost mid-edit.

**Board** (below configs) — free-form canvas. Two buttons in the board heading row:
- `+ Add Item` — opens modal; if groups exist, shows an optional "Add to Group" dropdown
- `+ New Group` — opens a custom modal for the group name, creates a resizable group bubble with a randomized color

**Cards** — click to expand (shows cost breakdown + Delete / Add to Group buttons). Drag to move. Resize via bottom-right handle. Items inside a group have their border color matched to the group's color.

**Groups** — drag by the label row. Resize via bottom-right handle. Items inside are absolutely positioned within the group (`rx/ry`). Resizing only reflows items if they'd overflow. Small color dot (bottom-left, away from resize handle) cycles group color; item card borders update to match automatically.

**Dragging grouped items** — uses a ghost clone. Dropping back within 120px snaps back. Dropping >120px outside breaks the item free onto the board. Badge turns orange (will snap) → red (will break) as you drag away.

## Visual style

- Sketchbook aesthetic: grid-line background, Georgia serif, italic title
- SVG `feTurbulence` + `feDisplacementMap` filter (`#sketchy`) applied to all major elements for a hand-drawn wobble
- Deterministic per-element border-radius and rotation via seeded hash (`sketchRand`), so shapes are consistent across re-renders

## Color system

**Block colors** (`BLOCK_COLORS`) — 7 options used for config panels and group containers:
red, teal, blue, orange, green, pink, purple

- New configs/groups get a random color via `randomColorIdx()`, which guarantees no same color twice in a row (tracked via `_lastColorIdx`)
- Color dot on each block cycles through the palette (`cycleConfigColor` / `cycleGroupColor`)
- `shuffleConfigColors(id)` re-randomizes a config's panel color and chip salt

**Supply chip colors** (`CHIP_COLORS`) — 6 colors assigned per-supply via `hashColorIdx(sup.id + cfg.chipSalt, len, prevIdx)`:
- Uses FNV-1a hash for uniform distribution (no color favoritism)
- `exclude` param picks uniformly from `len-1` valid slots then maps past the excluded index — no back-to-back repeats and no color gets a bonus from being the fallback

## Key functions

| Function | What it does |
|---|---|
| `render()` | Clears and redraws entire `#content` div |
| `buildConfigPanel(cfg)` | Builds a config panel DOM node |
| `buildBoard()` | Builds the board section with groups + ungrouped cards |
| `buildItemCard(item)` | Builds a draggable item card |
| `calcCost(item)` | Sums `(cost/qty) * convertedAmount` across usages |
| `startMove(e, type, id, el)` | Begins drag for item or group |
| `startResize(e, type, id, el)` | Begins resize for item or group |
| `reflowGroupMembers(grp)` | Re-flows items inside a group only if they overflow |
| `submitSupply(configId)` | Validates and adds a supply to a config |
| `saveConfig(id)` | Reads name input, locks config |
| `openItemModal()` | Opens Add Item modal (requires ≥1 locked config) |
| `saveItem()` | Saves item to state, respects optional group selection |
| `openGroupModal(itemId)` | Opens assign-to-group modal for an item |
| `shuffleConfigColors(id)` | Reshuffles chip salt + panel color for one config |
| `hashColorIdx(seed, len, exclude)` | FNV-1a hash → uniform color index, skipping `exclude` |
| `randomColorIdx()` | Random block color index, no repeat from previous |
| `confirmReset()` | Wipes localStorage and resets state |
| `sketchRand(seed, min, max)` | Deterministic seeded random for consistent shapes |

## Things that came up during development (good to know)

- Re-renders wipe the DOM, so any unsaved input (like the config name) must be flushed to state *before* `render()` is called — `submitSupply`, `deleteSupply`, `cycleConfigColor`, and `shuffleConfigColors` all do this
- The ghost drag pattern (cloneNode) is used for grouped items so the original stays in place until drop
- `reflowGroupMembers` is intentionally conservative — it only reflows if items would overflow, so expanding a group never moves things around
- No browser `alert()` or `prompt()` anywhere — every dialog is a custom modal using `openModal(id)` / `closeModal(id)`
- If localStorage gets stale schema (items with `x/y` but no `rx/ry`), clearing it with `localStorage.removeItem('bijoux_v1')` in the browser console fixes it
