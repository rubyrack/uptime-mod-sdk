# Script mods

A script mod is code that plays the game for the player. It runs once per quantum, reads a slice of the world and issues commands. This page covers what Lua mods and Rust mods share: the manifest fields, the quantum, declared reads, commands and receipts, limits, faults and the cost to a run.

A script mod can also draw [panels, a HUD card and keys](PANELS.md) and raise [toasts](TOASTS.md).

Language details are in [Lua mods](LUA.md) and [Rust mods](RUST.md). To make a mod in any other language, see [Other languages](OTHER_LANGUAGES.md). The manifest rules that all packs share are in [Content packs](CONTENT_PACKS.md).

## The manifest

A script mod sets `kind` to `["script"]` and adds a `script` block. A mod may be both a script mod and a content pack: `kind` can list `content` and `script` together.

```json
{
  "manifest_version": 1,
  "id": "com.example.my-mod",
  "version": "1.0.0",
  "api_version": "v1alpha1",
  "api_minor": 1,
  "game_version": ">=0.1.0, <0.3.0",
  "kind": ["script"],
  "script": { "runtime": "lua", "entry": "logic/main.lua" },
  "affects_gameplay": true,
  "commands": ["finance", "customers"],
  "events": ["receipts", "customers"],
  "reads": ["topbar", "prospects", "capacity"]
}
```

- `script.runtime` is `lua` or `component`. A `lua` entry ends in `.lua`. A `component` entry ends in `.wasm`.
- `commands` lists the command categories the mod may issue: `infrastructure`, `finance`, `customers`, `incidents`. The gate refuses a command outside them.
- `events` lists the event categories the mod is told about. An unlisted category is never encoded for the mod.
- `reads` lists the snapshot sections the mod sees. See the next section.
- `state_schema` is a number that versions the mod's saved state. The game migrates a saved state of another schema before the mod sees it. A mod that changes its state shape sets a new number and handles the old one in `migrate`.
- A script mod with no `commands` does not affect gameplay. `affects_gameplay` must then be `false`, unless the same mod also carries hardware, scenarios or customers.

Set `api_minor` to the minor of the SDK you built against. The Rust SDK exports it as `uptime_mod::snapshot::API_MINOR`, currently `1`. The Lua types come from the same ledgers. A mod built against an older minor keeps loading unless a later minor broke a section its `reads` names; [Reference](REFERENCE.md) lists the breaks.

## The quantum

A mod runs on a grid of sim time. One quantum is 15 seconds of sim time. Boundary `k` sits at `k` times 15 seconds. At each boundary the game runs one step of every script mod. Pausing the game stops the grid, because the grid is sim time.

At boundary `k` the game does these things in order:

1. It lands the commands the mods returned from step `k - 2`.
2. It closes the event batch for `k`: the engine events since the last boundary, the receipts from step 1 and, on the first boundary of a world, a lifecycle marker.
3. On every fourth boundary, and on the first boundary of a world, it projects a fresh snapshot of the sections the mods declared.
4. It hands step `k` to the mod worker.

The mod worker runs on its own thread and never touches the engine. The game waits for step `k - 2` to come back before landing it, so a command lands exactly two quanta after its step, whatever the step cost. Two quanta is 30 seconds of sim time.

Player commands apply as they arrive, so they are already in when a boundary runs.

## Declared reads

A mod sees only the sections its `reads` list names. Any other section is absent: `nil` in Lua and `None` in Rust. A section that exists and is empty is an empty list. A mod with no `reads` gets only the envelope.

The envelope always carries three fields: `mod_api` (the generation and the minor), `quantum` (the quantum the snapshot was projected at) and `sim_time_ns`.

The sections a mod may name are these. A name outside the list makes the pack invalid, with the rule `manifest` at `/reads/<index>`.

`status`, `topbar`, `cage_offer`, `regions`, `az_trunks`, `azs`, `racks`, `hosts`, `switches`, `appliances`, `cables`, `edds`, `gateways`, `patch_panels`, `active_attack`, `flood_protection`, `cable_trays`, `lags`, `switch_pairs`, `host_pools`, `segments`, `subnets`, `reservations`, `device_transit`, `deliveries`, `sale_history`, `active_incidents`, `active_macro_events`, `spares`, `pending_growth_requests`, `pending_customer_requests`, `service_gates`, `service_planes`, `function_pool`, `difficulty`, `min_difficulty`, `hostility`, `relocation`, `migration`, `degraded_control_planes`, `stranded_vms`, `customers`, `prospects`, `customers_summary`, `staff`, `hire_pool`, `staff_auto_evacuate`, `cost_breakdown`, `cost_rates`, `financing_breakdown`, `revenue_breakdown`, `capacity`, `service_settings`, `business_policy`, `port_flows`, `workloads` and `parts`.

Each section has the shape of the matching view in `snapshot.v1alpha1.fields`. A section that is not on the list, such as the cart, haulers, loose items or teaching UI, is never given to a mod.

Two sections carry the ids that some commands need and no other section lists:

- `workloads` has one row for each workload that can be moved to another host, with `unit_id`, `kind_tag`, `id`, `customer_id` and `host_id`. `MigrateWorkload` and `MigrateWorkloadToAz` take `kind_tag`, `id` and `unit_id` back unchanged, with `host_id` as `source_host_id`. This works for every service, not only VMs.
- `parts` has one row for each device with a part that is not healthy or that has an order, a spare or a swap in progress. The row lists all of that device's parts. `OrderPart`, `CancelPartOrder` and `RepairPart` take its `owner_kind` and `owner_id` and a part's `component_id`. A device whose parts are all healthy, with nothing ordered, is not listed, because those commands refuse a healthy part.

The game builds neither section unless a loaded mod declares it.

Rules for `reads`:

- No name twice, and at most 64 names.
- Declare the least you need. The game projects only the union of what the loaded mods declare, and hands each mod only its own sections.
- The snapshot is copied into the mod and decoded there, and both come out of the step's fuel. A large section such as `hosts` costs real fuel on a big fleet.
- The snapshot refreshes every fourth quantum, which is 60 seconds of sim time. In between, a step sees the same snapshot as before. The envelope's `quantum` tells you which quantum it came from.

## Events

Each step gets a batch of events: `quantum`, a `dropped` count and an `events` list. `dropped` counts events the engine's ring had already trimmed. A mod that sees a gap can resynchronise from the snapshot.

An entry of the list is one of these:

- A lifecycle marker, `save_loaded` or `run_started`.
- A receipt for a command that landed. See below.
- An engine event with the time it happened (`at_ns`), its category and the event itself.

The engine event categories are `incidents`, `customers`, `finance` and `builds`. Which category carries which event is in `events.v1alpha1.fields`.

## Commands and receipts

A command is an externally tagged value, such as `AcceptProspect` with a `prospect_id`. The set of commands a mod may issue is the mod subset of the player's commands. `commands.v1alpha1.fields` lists every command and field. The game refuses a command a mod may not use.

Rules for commands:

- A step may return up to 32 commands. The game drops the rest. The mod gets one `rate_limited` receipt whose `index` is 32, the first dropped position. The receipt carries no count, so work it out as the number you queued minus 32.
- One encoded command may be at most 16 KiB. A step's commands, panels and toasts together may be at most 256 KiB. The game discards a step over either cap whole and counts a strike.
- The game drops a command it cannot decode. The mod gets an `invalid_command` receipt that carries only the command's position. The decoder's message reaches neither the mod nor the log.
- Commands land in a fixed order: mod by mod, and each mod's commands in the order the step returned them.
- A landed command goes through the same handler as a player command, with the mod as the actor. The gate refuses a command whose category the manifest did not list, or one the game does not publish to mods. The receipt then carries the code `permission_denied`.

A receipt tells the mod what happened. It arrives in the batch for the boundary where the command landed, which is two quanta after the step. A receipt has these fields:

| Field | Meaning |
|---|---|
| `quantum` | The quantum whose step issued the command. |
| `index` | The position of the command in that step's output. |
| `ok` | Whether the command succeeded. |
| `code` | A stable machine code. It is empty on success and never empty on a failure. |
| `msg_key` | A localization key for the failure, or empty. |
| `created` | Every entity the command created, in creation order. |

On a failure the code is the engine's code, or else the message key, or else `refused`.

A command takes effect at the landing, but a snapshot shows it only once a snapshot is projected at or after that boundary. Do not repeat an order while the first one is in flight. The example autoscaler tracks each order by the quantum it issued and keeps that list in saved state.

## State and migration

A step may save state. The host keeps the last state a step returned, and the save file keeps it too. State is bytes in the Rust SDK and a Lua table in the Lua guest. A state may be at most 1 MiB. The game refuses a larger one with a warning and keeps the previous state.

Saved state is the only thing that survives a load or a fault. After an out-of-fuel or trap strike, a reload or a load, the game builds the mod again and runs `init` with the config and the last saved state. A local, a global or a field that you did not save is gone. A mod whose behaviour depends only on its config, its saved state and what each step hands it plays the same after a load as it would have without one.

Two cases skip that rebuild. A mod the save recorded as off stays off, and the load shows a `mod_off_in_save` warning, unless the mod's version is newer than the one the save ran. A newer version clears the off flag and the strike counters and resumes. A saved state over 1 MiB is dropped at load, before any migration, and the mod is rebuilt with no state.

The first step after a load gets the whole snapshot again, and the extra fuel that decoding it takes. The next section covers that fuel.

### The state schema

`state_schema` in the manifest is a whole number that names the shape of your saved state. It defaults to 0. Raise it when you change that shape. The save records the schema next to the state.

When a save loads and its schema differs from the manifest's, the game migrates the state before the mod sees it:

1. It builds a throwaway instance of your mod and runs `init` on it with your config and no state.
2. It calls `migrate(from_schema, state)` on that instance. `from_schema` is the schema in the save, and `state` is the saved state.
3. What `migrate` returns becomes the state, and the save records the manifest's schema.

In Lua, `migrate` is the `M.migrate` function of the entry table. In Rust, override `Mod::migrate`. Without one, in either language, a raised `state_schema` fails the migration and the mod stays off for that save.

`migrate` receives two values and has no `ctx`. It cannot read the snapshot, issue a command or save state. It runs on its own fuel, 100,000,000, and its result must fit the 1 MiB state cap.

A migration fails when `migrate` returns an error, runs out of fuel, traps or returns too much. It also fails when the save's schema is newer than the manifest's, because the game cannot turn a newer shape back. A failed migration keeps the old state, schema and version as saved, turns the mod off for that save and shows a warning. The rest of the world loads.

### Decode fuel

A step that receives a new snapshot gets extra fuel to decode it, 180 for each byte of the snapshot, up to the allowance of a snapshot as large as a mod's memory, 64 MiB. That extra fuel is on top of the 20,000,000 a step has, and it is why a snapshot refresh step can run longer than other steps. A snapshot cannot be larger than the memory limit, so the allowance stops there. Declare the least you need in `reads` so the allowance covers the decode with room to spare.

## Limits

These limits hold for every script mod. Fuel is the deciding limit for time. It counts wasm operations plus a fixed charge for each call into the host, so the same mod hits the same limit on every machine.

| Limit | Value |
|---|---|
| Fuel per `step` | 20,000,000 |
| Fuel per `init` | 200,000,000 |
| Fuel for the `init` that rebuilds an instance after a strike | 20,000,000 for a component, 200,000,000 for Lua |
| Fuel per call into the host | 2,000, plus 4 per byte moved |
| Memory per mod | 64 MiB for the linear memory and the GC heap together |
| Commands per quantum | 32 |
| Bytes in one command | 16 KiB |
| Bytes in commands, panels and toasts per step | 256 KiB |
| Toasts per step | 2 |
| Bytes of one toast's text | 160 |
| Bytes of saved state | 1 MiB |
| Live host handles | 64 |
| Bytes one random call may ask for | 64 KiB |
| Log lines per call | 64 |
| Bytes kept of one log message | 512 |
| Lines in a mod's log ring | 256 |

A wall-clock backstop also guards each call. It is set far past the fuel limit and never under one second. It exists for a failure of the metering and fuel decides first, even on a loaded machine. If it ever fires, the mod is disabled and the run is not replayable.

## Faults, strikes and rebuilds

Each step ends in one of these ways:

| Outcome | What happens |
|---|---|
| Ok | The commands and state are applied. |
| The mod returns an error | In Lua, an `error()` in `step`. In Rust, an `Err` from `step`. The game drops the step's commands and state. It is not a strike, and the mod runs again next quantum. |
| Out of fuel | A strike. |
| A trap | A strike. |
| An output over a cap | A strike. The instance stays. |
| A toast or a panel update over its own cap | Not a strike. The game refuses that toast or update and logs one line. The rest of the step lands. |
| Memory limit | The mod is disabled at once. |
| Wall-clock backstop | The mod is disabled at once. |
| Stack overflow | The mod is disabled at once. |
| A panic in the host | The mod is disabled at once. |

After an out-of-fuel or trap strike the game drops the instance. The next step builds a fresh instance and runs `init` again with the mod's config and its last accepted state. That `init` runs on its own fuel, which the step is charged for, and it uses up one rebuild.

After an over-cap output strike the instance stays. The game discards that step's output, builds nothing and uses no rebuild.

The counters:

- Three strikes disable the mod. Three clean steps in a row clear the strikes.
- A mod gets three rebuilds per session. A strike that needs a fourth disables the mod.
- If `init` fails when the world loads, or when the instance is rebuilt, the mod is disabled.

A disabled mod runs nothing for the rest of that world's session. It stays loaded and off. The game logs why, with a line for each strike. It does not stop the world.

A world refuses to start when the script host cannot load a component, or when a Lua script does not compile.

## Determinism

A run must replay: the same inputs must give the same commands. The host removes the usual sources of drift.

- The clocks a mod can read return the quantum's sim time. Nothing reads the wall clock.
- Random numbers come from a stream seeded by the quantum's seed. The seed is a fixed hash of the world's mod seed, the quantum and the mod's id.
- The engine runs with fuel metering and canonical NaN values, and with no threads and no relaxed SIMD.
- Fuel counts operations, so a limit breach is the same breach on every run.

What stays with you: do not let an unordered walk decide the order of commands. Sort first. The language pages give the details.

## Testing a mod

The runner has a batch mode for mods. It builds a scenario world with your pack selected, steps it for a number of quanta and prints one JSON line for each command that landed. The line has `issued`, `landed`, `mod`, `ok` and `code`. The run is the same every time for the same seed, and the seed is `1` unless you pass one. A golden file holds the expected output. The runner exits `1` and names the first differing line when the output does not match. A bad command line exits `2`.

```
sim-runner --mod-run PACKDIR (--scenario ID | --world SAVE) --quanta N [--seed N] [--golden FILE] [--watch]
```

Your mod's log lines go to stderr as `[mod-log] <id> k=<quantum> <level>: <text>`, so they never change the JSON lines a golden file compares. `--watch` reloads the pack when its files change, as developer mode does in the game. `uptime-mod run` wraps this; see [the CLI](CLI.md#run).

## Developer mode

Developer mode reloads your mod into a running single-player game when you change its files, without a restart. Turn it on with the Developer mode toggle on the title screen's Mods panel, then start or load a run. The laptop's Mods page then shows the mod log, the last reload and what a change needs. It is off in multiplayer and in the demo.

The game looks at your pack folder every half second and sorts each change:

| You changed | What happens |
|---|---|
| A Lua script | The scripts reload between two steps. |
| A component, the manifest's script `entry` | The new component is compiled and reloaded between two steps. |
| A panel file | The panels are checked again and swapped in at the next quantum. |
| A file under `art/` | The game redraws the art. |
| A scenario | Start a new run to play it. |
| Anything else: the manifest, hardware rows, the config schema | Save, then load the save. |

A reload builds a fresh instance and runs `init` with the mod's config and its last saved state, as a rebuild does. It is not a strike and uses no rebuild. The two steps the old code had in flight still land; the new code's commands land from the quantum after. A reload that fails keeps the old code running and writes `reload failed` to the log.

A mod the game disabled (strikes, rebuilds, a failed `init`, a fault) runs again after a reload that comes up, with its counters cleared. A mod that a save keeps off stays off.

A save made after a reload records that developer mode changed the run. Loading it later says the pack changed since the save, as any edited pack does.

## What a modded run costs the player

A run that has had a pack that affects gameplay active earns no achievements, posts no leaderboard scores and feeds no Steam stats. The flag stays set for the life of the save, including after a recovery that strips the mod. A pack that does not affect gameplay never sets it. Packs that only carry `locale` or `art`, and script mods that declare no `commands`, are in that group.

Tell players before they install the mod. The flag cannot be cleared once it is set.
