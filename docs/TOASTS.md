# Toasts

A toast is a short notice that appears on screen for a few seconds and goes away. A script mod raises one from its step. The game shows it to every player, the host and the guests alike. Use a toast for a thing the player should notice now, such as an attack starting. Keep a record of anything the player may want later in a panel, because a toast leaves no trace.

A toast is a screen-corner notice in the game's own style. A mod picks the tone and the words and nothing else. It cannot set a colour, a duration, an icon or a button.

Toasts belong to script mods. The shared rules for script mods are in [Script mods](SCRIPTS.md). A toast needs no entry in the manifest and no panel file.

## Raise a toast

In Lua, call `ctx:toast(tone, text)`:

```lua
for _, ev in ipairs(ctx.events) do
  local e = ev.Engine and ev.Engine.event
  if type(e) == "table" and e.DdosStarted then
    ctx:toast("Danger", "DDoS attack: " .. e.DdosStarted.gbps .. " Gbps")
  end
end
```

In Rust, call `step.toast(tone, text)`. It returns an error when the toast breaks a cap, so you learn at the call:

```rust
use uptime_mod::Tone;

if let Err(why) = step.toast(Tone::Warn, "Cash is below the reserve") {
    step.log(3, &why);
}
```

Handle the error as shown. Returning it from `step` with `?` would drop the step's commands and state.

A component in any other language puts each toast in the `toasts` field of its `step-output`. See [Other languages](OTHER_LANGUAGES.md).

## Tone

The tone says how serious the notice is. It is one of these, spelled as shown:

| Tone | Use it for |
|---|---|
| `Neutral` | A plain fact. |
| `Info` | Something worth knowing. |
| `Good` | Something that went well. |
| `Warn` | Something that needs attention soon. |
| `Danger` | Something that is going wrong now. |

A tone is never a colour. The game picks the look. An unknown tone is a Lua error, and in Rust it does not compile, because `Tone` is an enum.

## Text

The text is plain text of 1 to 160 bytes, on one line. The game never reads markup, so `[b]x[/b]` shows as typed. The cleaning is the one panel labels get in data: the game drops every control character, a newline included, and the invisible formatting characters, turns the rest into NFC and refuses a text that is too long after that. A text that is empty after cleaning is refused. The text is not looked up as a locale key, so it shows in the language you wrote.

The game shows the mod's id with the toast, so a player can tell who raised it.

A mod has one toast on screen at a time. A newer toast from the same mod replaces the one still showing, so when a step raises two, the player sees the second.

## Caps

| What | Cap |
|---|---|
| Toasts in one step | 2 |
| Bytes of one toast's text, after cleaning | 160 |
| Toasts the world holds | The newest 16, across all mods |
| Commands, panels and toasts in one step | 256 KiB |

The Lua guest and the Rust SDK refuse a third toast, or a text that is empty or over 160 bytes, at the call. In Lua that is an error you can catch with `pcall`. A step that catches it still lands.

The game checks every toast again when the step returns, because a component in another language has no SDK to refuse for it:

- A step that returns more than 2 toasts has all of its toasts refused. The game never keeps the first two. It writes one line to the host log, `toast refused: toasts_over_cap`, and the rest of the step lands.
- A toast whose text is empty or too long after cleaning is refused on its own, with one log line, and its neighbour in the same step still lands.
- A refused toast is never a strike.
- The toasts count toward the 256 KiB a step may return. A step over that is discarded whole and counts as a strike, as it does for commands and panels. See [Script mods](SCRIPTS.md#commands-and-receipts).

One quantum is 15 seconds of sim time, and a step can raise at most 2 toasts, so a mod cannot raise more than one toast every 7.5 sim seconds. The world also holds only 16 toasts at a time. Raise a toast for a change, not on every step, because a player can mute a mod that is noisy.

## When a toast arrives

A toast lands with the step's commands, two quanta after the step that raised it. A quantum is 15 seconds of sim time, so a toast from step `N` shows at `N + 2`, about 30 seconds of sim time later. See [Panels](PANELS.md#when-data-arrives).

Toasts are part of the world. A save keeps the toasts that have landed. A toast raised by a step that was still in flight when the game saved lands after the load. The game never shows a player the same toast twice, and a player who loads or joins a world does not see the toasts that landed before. They see only the ones that land afterwards.

A mod that stops, or that the game disables, loses the toasts it had in the world. A save that loads without the mod drops them too.

## What a player controls

A toast is transient. It never goes into the comms panel's history, and a mod cannot read one back, not even its own. A player can mute a mod's toasts, one mod at a time, from the Mods page. A muted mod keeps running and keeps raising toasts, and the player sees none of them.

Because of this, never put the only copy of something important in a toast. A player may have muted you, may have looked away or may have joined later.

## Toasts in multiplayer

Every joined player gets the same toast, and a mod cannot aim one at a single player. A dedicated server shows toasts to its players too. A mute is a choice each player makes on their own screen. See [Multiplayer](MULTIPLAYER.md#hud-toasts-and-keys-in-multiplayer).

## The example

`ModSDK/examples/com.uptime.example-alerts` raises one `Danger` toast when a DDoS attack starts. It also shows a HUD card and two keys. See [Panels](PANELS.md#the-hud-point).
