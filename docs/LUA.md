# Lua mods

A Lua mod is a script pack: a manifest and some `.lua` files. It reads the same snapshot, issues the same commands and gets the same events as a Rust mod, under the same limits. The shared rules are in [Script mods](SCRIPTS.md). This page covers what is specific to Lua.

Lua 5.4 runs inside a sandbox with no clock, no files and no network. One guest component runs every Lua mod. The game loads your scripts into it when the world starts.

## Pack layout

```
com.example.my-mod/
  manifest.json
  logic/main.lua
  logic/policy.lua
```

The manifest names the runtime and the entry file:

```json
{
  "manifest_version": 1,
  "id": "com.example.my-mod",
  "version": "1.0.0",
  "api_version": "v1alpha1",
  "api_minor": 0,
  "game_version": ">=0.1.0, <0.3.0",
  "kind": ["script"],
  "script": { "runtime": "lua", "entry": "logic/main.lua" },
  "affects_gameplay": true,
  "commands": ["customers"],
  "events": ["receipts"],
  "reads": ["prospects"]
}
```

There is no build step. The pack ships its source.

The game bundles every `.lua` file under the entry's directory, at any depth. For the entry `logic/main.lua` that is the folder `logic`. For an entry at the pack root it is every `.lua` file in the pack. The bundle may hold at most 64 files and 256 KiB of source in all. A pack that goes over either limit is invalid, with the rule `archive`.

No other file is allowed. A data file, a readme or a `.lua` file outside the entry's folder fails with the rule `content_outside_declared_category`. The same archive rules as every pack apply, see [Content packs](CONTENT_PACKS.md).

Before the world starts, the game compiles every file without running it. A syntax error refuses the world and names `file:line`.

## The entry chunk

The entry file returns a table. `step` is required. `init` and `migrate` are optional.

```lua
local M = {}

function M.init(ctx)
  -- ctx.config: the mod's config table ({} when there is none)
  -- ctx.state:  the last saved state, or nil on a first run
end

function M.step(ctx)
  for _, p in ipairs(ctx.snapshot.prospects or {}) do
    ctx:issue({ AcceptProspect = { prospect_id = p.id } })
  end
end

function M.migrate(from_schema, state)
  return state
end

return M
```

The game reads `step` from the table with a raw lookup, so a metatable cannot change it later. If the entry chunk does not return a table with a `step` function, `init` fails and the mod is off.

`step` runs once per quantum. `ctx` holds:

| Name | What |
|---|---|
| `ctx.quantum` | `{index, sim_time_ns, seed}` |
| `ctx.snapshot` | The latest snapshot. It is the same table across the quanta between refreshes. |
| `ctx.events` | The events for this quantum, in batch order. |
| `ctx:issue(cmd)` | Queues a command and returns its index, counted from 0. The command lands two quanta later. |
| `ctx:save_state(tbl)` | Saves the mod's state. The last call in a step wins. |
| `ctx:panel(id, update)` | Sets the values of a panel the pack declares. See [Panels](PANELS.md). |
| `ctx:log(level, msg)` | Writes a log line. `level` is 0 to 4. |
| `ctx:queued()` | The number of commands queued so far in this step. |

Sections you did not list in `reads` are `nil`. Always guard them: `ctx.snapshot.prospects or {}`.

A command is an externally tagged table: `{ AcceptProspect = { prospect_id = 7 } }`. Optional fields may be left out. A command with no data is a plain string. The command list and the fields are in `commands.v1alpha1.fields`.

Each entry of `ctx.events` is also tagged. An engine event looks like `{ Engine = { at_ns, category, event } }`, and a receipt looks like `{ Receipt = { quantum, index, ok, code, ... } }`. A lifecycle marker is `{ Lifecycle = { kind = "run_started" } }`. Compare the string at `event` for an event that has no data:

```lua
for _, ev in ipairs(ctx.events) do
  if ev.Engine and ev.Engine.event == "DdosEnded" then
    ctx:log(uptime.LOG.info, "the attack ended")
  end
  if ev.Receipt and not ev.Receipt.ok then
    ctx:log(uptime.LOG.warn, "refused: " .. ev.Receipt.code)
  end
end
```

The global `uptime` holds three things:

- `uptime.LOG` has `trace` 0, `debug` 1, `info` 2, `warn` 3 and `error` 4.
- `uptime.GUEST` is a string that names the guest build.
- `uptime.array(t)` marks a table so it saves as an array.

Without the mark, a table saves as an array only when its keys are exactly 1 to n, and an empty table saves as an empty array. Use the mark for a table with holes that must stay a list.

A marked table saves only the entries 1 to `#t`. The encoder drops its string keys, and any integer keys past `#t`, without a warning.

## Several files and require

`require` loads a file from your bundle. The name is relative to the entry's folder, and a dot stands for a folder separator. With the entry `logic/main.lua`:

- `require("policy")` loads `logic/policy.lua`.
- `require("util.sort")` loads `logic/util/sort.lua`.

A file loads once. The next `require` returns the same value. A file that returns nothing counts as `true`. `require` refuses a name that is empty, contains `..` or starts with `/`. A name that matches no bundled file raises an error that names the path it tried. Nothing outside the bundle can be loaded.

The example pack does this: `main.lua` calls `require("policy")` in its top lines and builds the policy in `init`.

## The sandbox

Available: the base functions, `table`, `string`, `math`, `utf8` and `coroutine`.

Removed or limited:

- `io`, `os`, `debug` and `package` do not exist.
- `dofile` and `loadfile` do not exist.
- `string.dump` and `math.randomseed` do not exist.
- `load` reads text only. It never loads bytecode, whatever mode you pass.
- `require` loads bundled files only.
- `collectgarbage` accepts `collect`, `count`, `step` and `isrunning`.
- `print` writes a log line at level 2, and `warn` writes one at level 3. Neither touches standard output.

A script cannot reach the clock, the file system or the network, so it cannot read anything a replay does not also have.

## Data going in and out

The game passes data to the guest as named msgpack. The guest turns it into Lua tables:

- Maps become tables and lists become sequences from 1.
- Integers become Lua integers. Floats become Lua floats.
- A map key that is nil or NaN is an error.

Commands and saved state go back through the same encoder, with these rules:

- Keys must be strings or integers. A function, a userdata or a thread is an error.
- A table that contains itself, or a table with a weak-mode metatable, is an error.
- A float with a whole value saves as an integer. `5.0` comes back as `5`.
- A string that is not valid UTF-8 saves as binary.
- The encoder writes map keys in sorted order, so the saved bytes do not depend on the order you built the table in.
- Within one step, the commands and the state together may be at most 4 MiB. Past that the step fails with an error. The host's lower caps apply as well.

## Determinism rules

A run must replay exactly: the same inputs must give the same commands. The sandbox removes the usual sources of drift. Three things stay with you.

- **Sort when order reaches a command.** Do not rely on `pairs()` order. The order can change when the Lua state is rebuilt, which happens after a load, after an out-of-fuel or trap strike and on a reload, because the tables are filled in a different order. If a loop decides which command goes first, loop over a sorted list of keys. `table.sort` is not stable, so give it a comparison that never ties.
- **Use `math.random` for randomness.** The guest reseeds it from `ctx.quantum.seed` at the start of every step, so it draws the same numbers on every replay. Do not seed it yourself. During `init` and while the entry chunk loads, it draws from a fixed seed.
- **Keep state in `save_state`.** After an out-of-fuel or trap strike, a reload or a load, the Lua state is built from scratch and `init` runs again with the last saved state. Anything kept only in a local or a global is gone.

The example autoscaler has one `pairs()` loop, which merges the config over the defaults. It issues no command, so its order cannot change the output. Every loop that issues a command or drops an order walks a list with `ipairs`.

Saved state holds tables, arrays, strings, integers, floats and booleans.

## Errors

An error in `step` ends that step. The game logs the message with a traceback. The game drops the step's commands and state, and the mod runs again next quantum. It is not a strike.

Running out of fuel is a strike. Three strikes disable the mod. The full table is in [Script mods](SCRIPTS.md).

An error in `init`, or in the entry chunk, disables the mod. The game logs the message.

## Limits

The shared limits table is in [Script mods](SCRIPTS.md). The ones that matter most to a Lua author:

- A step has 20,000,000 fuel. A script that loops forever runs out and the step is a strike.
- Loading the scripts and running `init` has 200,000,000 fuel. The `init` that rebuilds the instance after a strike gets the same amount, because it parses every script again.
- The whole mod gets 64 MiB. Lua's own heap is a fixed region inside that. A script that grows without bound gets Lua's `not enough memory` error, which `pcall` can catch.
- A step may issue 32 commands. The game drops the rest and sends a `rate_limited` receipt.
- Saved state may be 1 MiB. The game refuses a larger state and keeps the old one.

## Editor support

`ModSDK/lua/types/` holds LuaLS definition files, and `ModSDK/lua/.luarc.json` points the language server at them. The config sets Lua 5.4, turns off `io`, `os`, `debug` and `package`, and loads the library folder `types`. Copy both next to your pack, or point `workspace.library` at the folder.

| File | What |
|---|---|
| `uptime.d.lua` | Written by hand. The `ctx` classes, the `uptime` global and the shape of the entry table. |
| `uptime-snapshot.d.lua` | Generated. One class per snapshot type, from `snapshot.v1alpha1.schema.json`. |
| `uptime-commands.d.lua` | Generated. One class per command and the `uptime.Command` union, from `commands.v1alpha1.fields`. |
| `uptime-events.d.lua` | Generated. The event envelope and one class per delivered engine event, from `events.v1alpha1.fields`. |

The generated files follow the mod API ledgers. Do not edit them.

## Dev loop: hot reload

A debug build of `sim-runner` can reload your scripts while the world runs. Start it with `--watch-mods`, plus `--mods-dir` and `--enable-mods` or a pack selected in the game:

```
sim-runner --stdio --watch-mods
```

The runner compares the scripts on disk with the running ones twice a second. When they differ it builds a new Lua state and swaps it in between steps. The log shows `[mods] watching` when the watch starts and `reloaded` on a reload.

What a reload does:

- The scripts run from a new Lua state. `init` runs again with the last state you saved. Locals and globals start over.
- If the new scripts do not load, the old ones keep running and the log shows `reload failed:` with the file and line. Fix the file and save again.
- A reload is not a strike and does not count toward the rebuild limit. A mod the game has already disabled stays disabled until the world is loaded again.
- Only the `.lua` files are watched. A change to `manifest.json`, such as `commands`, `reads` or `events`, needs a restart.

Hot reload works only in a debug build of the runner, and only in a single-player world. A release build ignores `--watch-mods`. In a multiplayer world, and in a `--server` process, the runner logs one line to say it is off.

## The example

`ModSDK/examples/com.uptime.example-autoscaler-lua` is a full Lua mod: a manifest, a `main.lua` that builds the policy in `init` and calls it every step, and a `policy.lua` that holds the logic. It accepts the prospects the fleet can serve and buys a host when free capacity runs low. It declares `reads` of `topbar`, `prospects` and `capacity`, the three sections its policy uses. It also has a config schema, two panels (the policy and the decisions, with a pause toggle) and `state_schema` 1.

The Rust autoscaler in `ModSDK/examples/example-autoscaler` follows the same policy line for line, with the same defaults and the same saved state. The game's own tests run both and require equal commands and state. See [Rust mods](RUST.md).
