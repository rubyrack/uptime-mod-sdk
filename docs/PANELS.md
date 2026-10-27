# Panels

A panel is a screen a script mod draws inside the game. You declare the panel in a JSON file. Your mod fills it with data on every step. The game draws the widgets and checks every value your mod sends. A mod never draws pixels and never runs code in the game's interface.

Panels belong to script mods. The shared rules for script mods are in [Script mods](SCRIPTS.md).

## Declare panels

List the panel files in the manifest, at most 32:

```json
"panels": ["panels/fleet.json"]
```

Each file is a JSON object with a `panels` list and may be at most 64 KiB. Every object refuses keys it does not list. This file declares one panel with every widget kind:

```json
{
  "panels": [
    {
      "id": "fleet",
      "title": "Fleet policy",
      "point": "dashboard",
      "widgets": [
        { "id": "cash", "label": "Cash", "kind": "stat_tile" },
        {
          "id": "queue", "label": "Pending orders", "kind": "table",
          "columns": [
            { "id": "host", "label": "Host" },
            { "id": "age", "label": "Age", "numeric": true }
          ]
        },
        { "id": "load", "label": "Load", "kind": "line_chart", "tone": "info" },
        { "id": "note", "label": "Status", "kind": "text" },
        { "id": "auto", "label": "Buy automatically", "kind": "toggle" },
        { "id": "limit", "label": "Spend limit", "kind": "slider", "min": 0, "max": 100, "step": 5 },
        {
          "id": "mode", "label": "Mode", "kind": "select",
          "options": [ { "id": "fast", "label": "Fast" }, { "id": "slow", "label": "Slow" } ]
        },
        { "id": "go", "label": "Buy now", "kind": "button" }
      ]
    }
  ]
}
```

A panel has these fields:

| Field | Meaning |
|---|---|
| `id` | The panel's name, unique in your mod. |
| `title` | The heading, at most 64 bytes. |
| `point` | Where the panel appears. See the next section. |
| `entity` | Required for `entity_tab` and refused for other points. |
| `key` | Optional. A default key that opens the panel. `dashboard` panels only. See [Keys](#keys). |
| `widgets` | 1 to 24 widgets. A `hud` panel has 1 to 3. |

A `button` widget may also carry a `key`, which presses it. See [Keys](#keys).

An id matches `[a-z0-9_]{1,32}`. Panel ids are unique per mod. Widget ids are unique per panel. Column ids are unique per table, and option ids are unique per select.

## Extension points

`point` is one of these:

- `dashboard` adds a panel to the dashboard.
- `entity_tab` adds a tab to one kind of entity. Set `entity` to `host`, `rack` or `service`.
- `settings` adds a panel to the settings screen.
- `hud` adds a small card to the player's view, outside the ops console. See [The HUD point](#the-hud-point).

A control (toggle, slider, select or button) sits only on a `dashboard` or `settings` panel. An `entity_tab` panel and a `hud` panel display values and take no input.

## Widget kinds

| Kind | Extra fields | Value your mod writes |
|---|---|---|
| `stat_tile` | none | `Stat`: a `value`, a `detail` and a `tone`. |
| `table` | `columns`: 1 to 8, each with `id`, `label` and optional `numeric` | `Rows`: a list of rows, each with one string per column. |
| `line_chart`, `bar_chart` | optional `tone` | `Series`: a list of finite numbers. |
| `text` | none | `Text`: a string. |
| `toggle` | none | `Control` with `Bool`. |
| `slider` | `min`, `max` and `step`, all required | `Control` with `Number`. |
| `select` | `options`: 1 to 32, each with `id` and `label` | `Control` with `Choice`, an option id. |
| `button` | none | `Control` with `Press`. |

A field that does not belong to the widget's kind is an error. A slider needs finite bounds, `min` below `max` and `step` above 0. The distance from `min` to `max` must be a whole number of steps.

A tone is `neutral`, `info`, `good`, `warn` or `danger` in the file. In data it is `Neutral`, `Info`, `Good`, `Warn` or `Danger`. A tone is never a colour. The game picks the colour.

## Fill a panel with data

Call `ctx:panel(id, update)` in Lua, or `step.panel(id, &update)` in Rust. The update has a `values` table that maps a widget id to a value. Each call replaces the whole panel's data, and the last call for an id in a step wins. A widget you leave out shows no value, and it stays blank after a load or when a player joins. Send every widget you want shown, on every update.

```lua
ctx:panel("fleet", {
  values = {
    cash = { Stat = { value = "$12,400", detail = "after rent", tone = "Good" } },
    queue = { Rows = { { "web-1", "3" }, { "web-2", "7" } } },
    load = { Series = { 0.2, 0.4, 0.3 } },
    note = { Text = "All hosts answer." },
    auto = { Control = { Bool = true } },
    limit = { Control = { Number = 40 } },
    mode = { Control = { Choice = "fast" } },
  },
})
```

In Rust the same update uses the SDK types:

```rust
use std::collections::BTreeMap;
use uptime_mod::{PanelUpdate, Tone, WidgetValue};

let mut values = BTreeMap::new();
values.insert(
    "cash".to_owned(),
    WidgetValue::Stat { value: "$12,400".into(), detail: "after rent".into(), tone: Tone::Good },
);
step.panel("fleet", &PanelUpdate { values, entities: None })?;
```

For an entity tab, add `entities`. It is a list of `{ id, values }`, one row for each entity the tab shows. Entity rows belong to `entity_tab` panels only, an update holds at most 256 of them, and an entity may appear once.

The control values show the control's current state. When you change them, the game redraws the control to match.

## When data arrives

Panel data lands with the step's commands, two quanta after the step that wrote it. A quantum is 15 seconds of sim time, so data from step `N` shows at `N + 2`. Until then the player sees the previous data.

The game checks each update when the step returns:

- The panel must be one your mod declares, and each widget id must belong to that panel.
- Each value must fit its widget. A `Rows` row must have one cell per column. A `Choice` must name an option. A `Number` must be inside the slider's range.
- A step may not return the same panel twice.

A refused update writes one line to the host log and changes nothing. The previous data stands, and a refusal is never a strike. The game logs the first 8 refusals of a step and counts the rest.

The game refuses an update that goes over a cap below. It never trims one.

## Caps

| What | Cap |
|---|---|
| Panel files in a manifest | 32 |
| Size of a panel file | 64 KiB |
| Widgets in a panel | 24 |
| Columns in a table | 8 |
| Rows in a table | 100 |
| Points in a chart series | 240 |
| Options in a select | 32 |
| Entity rows in one update | 256 |
| Bytes of one panel update | 16 KiB |
| Bytes of panel data the world holds for one mod | 64 KiB |
| Bytes of one update to a `hud` panel | 2 KiB |
| Commands, panels and toasts in one step | 256 KiB |

An update that would take your mod past 64 KiB of held panel data is refused, and that panel keeps its previous data. A step that goes over 256 KiB in total is discarded whole and counts as a strike. See [Script mods](SCRIPTS.md). The caps on a `hud` panel are in [The HUD point](#the-hud-point), and the caps on keys in [Keys](#keys).

## Text rules

All panel text is plain text. The game never reads markup, so a string such as `[url=x]y[/url]` shows as typed.

| Text | Cap |
|---|---|
| Panel title, widget label, column label, option label | 64 bytes |
| Stat value, table cell | 128 bytes |
| Stat detail | 64 bytes |
| Text widget | 1,024 bytes |

### Labels that name a locale key

A panel title, widget label, column label or option label in a panel file is plain text, or `@` followed by a locale key. `"label": "@mod-com-example-fleet-rate"` shows the pack's text for that key, in the player's language. Text starting with `@` is a key, so write `@@` for a label that really starts with an at sign: `"@@handle"` shows `@handle`.

The key must be one of the pack's own messages, or a message of a locale pack the mod lists in `dependencies` (see [Content packs](CONTENT_PACKS.md#locale-packs)). Either way the owning pack's `en` files must define it. A key in any other namespace, a game key included, fails with `locale_key_not_namespaced`. A key with no `en` text, or whose dependency is not installed, fails with `locale_key_missing_en`. Both name the file and the field. A key reference counts against the 64 byte cap as written. If the player's packs do not hold the key, the label shows the last word of the key.

A pack that carries `locale` content cannot also be a script mod, so a script mod ships its words in a second, locale pack and depends on it:

```text
com.example.fleet-words/        kind content, content ["locale"]
  locale/en/main.ftl            mod-com-example-fleet--words-rate = Spend rate
com.example.fleet/              kind script
  manifest.json                 "dependencies": { "com.example.fleet-words": ">=1.0.0" }
  ui/panels.json                { "id": "rate", "label": "@mod-com-example-fleet--words-rate", ... }
```

The locale pack is client-required and the script mod is not, so a player joining a multiplayer world needs only the text pack. Data a mod writes at run time (stat values, cells, text) is not looked up.

The rules differ for files and data:

- In a panel file, a title or label that holds a control character or an invisible formatting character (bidirectional marks, overrides and isolates, zero-width spaces, the byte order mark) makes the pack invalid. The zero-width joiner and non-joiner are allowed, since Persian and Indic text need them. The game does not fix it. What you wrote is what shows. A config schema's titles follow the same rule.
- In data, the game drops those same characters and normalises the rest to NFC. A text widget keeps its newlines. Every other string is one line. The text must still fit its cap after cleaning, or the update is refused.

## The HUD point

A `hud` panel is a small card on the player's view. The player sees it while walking the datacenter, with no console open. Use it for one number or one short line that a player should be able to glance at, such as a count of open incidents. A card shows values only. The HUD has no cursor, so it takes no input.

```json
{
  "panels": [
    {
      "id": "hud",
      "title": "Incidents",
      "point": "hud",
      "widgets": [
        { "id": "open", "label": "Open incidents", "kind": "stat_tile" }
      ]
    }
  ]
}
```

You fill it with `ctx:panel` or `step.panel`, like any panel. The update lands two quanta after the step, as [When data arrives](#when-data-arrives) says, and every player in the world sees the same card. Each player can switch a mod's card off with "Show on HUD" on the Mods page, and the HUD shows at most 4 cards across all mods.

The pack check refuses a `hud` panel that breaks one of these rules. Each message starts with the code:

| Code | When |
|---|---|
| `hud_point_twice` | The mod declares a second `hud` panel, in any of its panel files. |
| `hud_widget_count` | The panel has more than 3 widgets. |
| `hud_widget_kind` | A widget is anything but a `stat_tile` or a `text`. |
| `key_not_allowed` | The panel carries a `key`. A card cannot be opened. |

A `hud` panel also refuses `entity`, as every panel that is not an `entity_tab` does.

The caps are tighter than for other panels:

| What | Cap |
|---|---|
| `hud` panels per mod | 1 |
| Widgets in the panel | 3 |
| Bytes of one update | 2 KiB |
| Values in one update | 3 |
| A `text` value | 128 bytes, on one line |

An update over a HUD cap is refused whole, like any other: one line in the host log, and the previous data stands. A line break in a `text` value on the HUD refuses the update. The card's data counts against the same 64 KiB of held panel data as the mod's other panels.

An update with an empty `values` clears the card, in Lua (`values = {}`) and in Rust (an empty map) alike. A mod uses it to take an alert down when there is nothing to show.

## Player input

A player presses a control, and your mod learns about it as an event. The event arrives in your mod's next batch, after the receipts, whatever your `events` list says. Only the mod that owns the panel gets it.

```lua
for _, ev in ipairs(ctx.events) do
  local input = ev.UiInput
  if input and input.panel == "fleet" then
    if input.widget == "auto" then
      auto_buy = input.value.Bool
    elseif input.widget == "go" and input.value == "Press" then
      want_host = true
    end
  end
end
```

A `UiInput` entry has `at_ns`, `player`, `panel`, `widget` and `value`. The value is `"Press"` for a button, `{ Bool = ... }` for a toggle, `{ Number = ... }` for a slider and `{ Choice = "id" }` for a select. Keep the result in saved state. The input is a one-time event, and the game does not repeat it.

A command you issue in response lands two quanta after the step that read the input. A panel update written in the same step lands with that command, two quanta after the step.

The game checks an input before your mod sees it. It refuses an input in these cases, and your mod gets nothing:

| Reason | When |
|---|---|
| `unknown_mod`, `unknown_panel`, `unknown_widget` | The id does not exist in the world. |
| `not_a_control` | The widget only displays. |
| `wrong_type`, `bad_value`, `unknown_choice` | The value does not fit the control. |
| `out_of_range`, `off_step` | A slider value is outside the range or between two steps. |
| `rate` | A connection sent more than 8 inputs at once, or more than 4 a second over time. |
| `queue` | Your mod already has 32 inputs waiting for this quantum. |
| `not_running` | The mod's worker has stopped. |
| `dedicated`, `not_joined` | See the next section. |

## Who can press

Every player who has joined the world can press a control: the host and every guest. The event's `player` is the seat of the player who pressed it, and 0 is the host. The game sets it from the connection, so a client cannot press as someone else. A connection that has not been accepted into the world is refused with `not_joined`.

If a control should only work for the host, check `player` in your mod and ignore the input when it is not 0. The game does not do this for you.

A dedicated server has no host player, so its panels show values and take no input (`dedicated`).

A mod cannot press its own controls. Input comes from player requests only, and no mod command presses a control.

The demo has no mods, and a demo build refuses panel input.

## Keys

A mod can ask for a key. A key does one of two things:

- A `key` on a `dashboard` panel opens that panel in an overlay, from anywhere in the world. The player does not walk to the laptop.
- A `key` on a `button` widget of a `dashboard` panel presses the button, from anywhere. The panel does not need to be open.

Both are for the mod's own panels. A mod key never opens a panel that belongs to the game or to another mod.

```json
{
  "id": "board",
  "title": "Alert board",
  "point": "dashboard",
  "key": "F6",
  "widgets": [
    { "id": "ack", "label": "Acknowledge", "kind": "button", "key": "F7" }
  ]
}
```

A key on any other panel or widget is refused with `key_not_allowed`: a `settings`, `entity_tab` or `hud` panel, or a widget that is not a button.

A press key gives your mod the same `UiInput` event as a click on the button: the `panel`, the `widget`, the value `"Press"` and the `player` who pressed. The same checks apply, including the input limits in [Player input](#player-input). A dedicated server takes no input, so a press key does nothing there.

### What a key looks like

`key` is a default. It is one base key with optional modifiers, written in this order and each at most once:

```text
ctrl+  alt+  shift+  BASE
```

`BASE` is one of `A` to `Z`, `0` to `9` or `F1` to `F12`, in capitals. `F6`, `shift+F7` and `ctrl+alt+J` are keys. `f6`, `Ctrl+F6`, `alt+ctrl+J` and `Space` are not.

### Caps and refusals

A mod declares at most 4 keys in all its panel files. The pack check refuses these, and each message starts with the code:

| Code | When |
|---|---|
| `key_not_allowed` | The key is not on a `dashboard` panel or on a button of one. |
| `key_bad_chord` | The key does not follow the grammar above. |
| `key_reserved` | The base key is one the game binds. |
| `key_twice` | Another key of this mod has the same default. |
| `key_cap` | This would be the mod's fifth key. |

The game binds these base keys, and a mod default on one is refused whatever its modifiers, so `ctrl+F` is refused as well as `F`:

`A`, `B`, `D`, `E`, `F`, `G`, `K`, `M`, `O`, `Q`, `R`, `S`, `U`, `W`, `1`, `2`, `3`, `4`, `F1`, `F2`, `F4`, `F8`, `F10`, `F12`, and the keys Escape, Tab and Shift.

That list can grow with the game. The source of truth is `RESERVED_BASE_KEYS` in `sim_core::mod_ui`, and `uptime-mod validate` reports a refused key with the code `key_reserved`.

The game matches its own keys by the physical key and ignores modifiers, which is why a modifier does not free a reserved key. Among a mod's own keys a modifier does count: `F6` and `ctrl+F6` are two separate keys and a press of one never fires the other.

### What the player controls

Only the default travels with the world. The binding is the player's own, kept in their settings, so two players in one world can use different keys.

- A player can rebind a mod key, or turn it off, in Controls. The pane lists each mod's keys under the mod.
- A default that the game already uses, or that an earlier mod in the world's load order has taken, does not bind. The key starts unbound and the player picks one in Controls. A mod cannot take a key from the game or from another mod.
- A player's own choice wins over another mod's key: binding a chord to a mod key takes it from any other mod key. A key the game itself uses cannot be given to a mod key, whatever the modifiers.

So write your panel and your button so they work without the key, and say nothing in the UI that depends on a particular key. The `alerts` example's card says "open the board" and not "press F6".

### The example

`ModSDK/examples/com.uptime.example-alerts` declares a `hud` card, a dashboard panel that opens on `F6`, and an Acknowledge button on `F7`. It raises a toast too. See [Toasts](TOASTS.md).
