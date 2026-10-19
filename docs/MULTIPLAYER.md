# Mods in multiplayer

In a multiplayer world the host picks the packs and the host's game runs the world. This page tells you what other players must install to join a world that uses your mod, what they see when they cannot, and how your mod's panels behave with several people in the room.

## Which packs a joiner needs

A joiner needs only the packs that change what their own screen draws: packs that carry `art` or `locale` content. The game calls these client-required packs. Every other pack reaches the joiner through the world itself, so it needs no install:

| Your pack | Joiner must install it |
|---|---|
| Hardware, scenarios or both | No. The world carries the catalog. |
| A script mod | No. The script runs on the host. |
| A script mod with panels | No. The panels travel in the world's data. |
| Anything with `art` or `locale` content | Yes, at the same version as the host. |

Today the game loads one kind of art file, a pack icon under `art/icons/`, and Fluent text under `locale/`. See [Content packs](CONTENT_PACKS.md).

The host's world records each client-required pack with its version and hash. A joiner's game states which of these packs it holds when it connects, and the host's game compares. A copy that differs in a single file has a different hash and is refused.

### A pack cannot carry art and a script

A pack that declares `art` or `locale` content cannot also be a script mod. The game refuses the manifest with the rule `client_pack_has_script`, and `uptime-mod validate` shows it. The game's message says why: a pack that every player downloads cannot carry a script. Split the work into two packs. Make the script pack depend on the art pack if it needs the art.

## When a joiner is refused

A joiner who lacks a client-required pack is refused before it enters the world. The host's game sends one of two messages, both in plain words:

- **Missing**: "The host's world uses packs you don't have", followed by the pack ids and versions.
- **Mismatch**: "Your copy of a pack the host uses doesn't match theirs", followed by the pack id, the host's version and the joiner's.

The refusal also carries the Workshop item id of each pack, when the pack was uploaded with `uptime-mod upload`. The package file records the id. See [Workshop](WORKSHOP.md#updates). A pack that never went to the Workshop gets no id and no button.

### The dependency dialog

The refusal opens the **Packs needed to join** dialog. It has one row for each pack:

- A missing pack reads "name version, not installed". A pack at another version reads "name: the host has X, you have Y".
- A row with a Workshop item id has **Subscribe** (or **Update** for a mismatch) and **Open in Workshop**. A row without one shows the text and no button, since the game has nowhere to fetch it from.
- **Retry** joins again with whatever is installed by then. **Back** returns to the title screen. The dialog never joins on its own.

Subscribing downloads a pack. It does not turn it on. A joiner's game claims every usable client-required pack on the machine, switched on or not, so subscribing is enough.

### The lobby block

A joiner usually learns about missing packs before the host starts the world. The host's game publishes the list of client-required packs in the lobby. A joiner who lacks one sees the same sentence in the room, in place of the waiting caption, and its Ready button is disabled.

The game checks again when a Workshop item the joiner is waiting for finishes installing. For any other change, the room has a **Check again** button. Ready returns once nothing is missing. If the host has already started the world, the joiner enters on its own once the missing packs are installed or Check again clears the block. If the host starts the world while the block stands, the joiner stays in the room.

A host with no client-required packs publishes nothing, and an older host that publishes nothing blocks nobody in the lobby. The runner still checks at the join.

### When the host changes world

If the host loads a different world while guests are connected, the game checks every guest again against the new world. A guest who lacks one of its client-required packs is disconnected with the same missing or mismatch message.

## Script mods run on the host only

A script mod runs once, in the host's game, or in the server process on a dedicated server. Guests do not run it, do not need its files and cannot see its code. They see what it does: the commands it issues change the shared world like any other command, and its panels appear on their screens.

This has two results for you:

- On a dedicated server the operator sets your mod's config, and no connection can change it. See [Config](CONFIG.md#config-for-a-dedicated-server).
- Hot reload is off in a multiplayer world. See [Lua mods](LUA.md).

## Panels in multiplayer

A panel's spec and data go to every player in the world, so a guest sees your panel with no pack installed. Who may press its controls depends on the world:

| World | Who can press |
|---|---|
| Single player | The player. |
| A hosted world | Every joined player, the host and the guests. |
| A dedicated server | Nobody. Its panels are view-only. |

The `UiInput` event your mod receives carries a `player` field. It is the seat of the player who pressed. The host is seat `0`, in a single-player world as well, and each guest has a seat of their own. The game fills `player` in from the connection, so a client cannot press as someone else.

**Check `player` before a control does something only the host should decide.** A button that buys hardware, changes a policy or clears state is host business in most mods, and every guest can press it. Compare against 0:

```lua
for _, ev in ipairs(ctx.events) do
  local input = ev.UiInput
  if input and input.panel == "fleet" and input.widget == "go" then
    if input.player == 0 then
      want_host = true
    end
    -- a guest's press is ignored
  end
end
```

A control that every player may use, such as a toggle that changes a panel's own view, needs no check.

The game refuses a press from a connection that has not finished joining (`not_joined`) and any press on a dedicated server (`dedicated`). Your mod sees neither. The limits that apply to each connection are the ones in [Panels](PANELS.md#player-input): a burst of 8 inputs, 4 a second over time, and 32 waiting for your mod in one quantum across all players. A press from a guest therefore counts against the same 32 as the host's.

On a dedicated server a player's panel shows its controls disabled, with the tooltip "Mod panels on this server are view-only."
