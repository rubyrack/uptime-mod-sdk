# Mods in other languages

The game runs a script mod as a WebAssembly component. The Rust SDK and the Lua guest are two ways to make one. You can make one in any language that builds a component for the WIT world, such as TinyGo, JavaScript with `jco`, C or C#. This page says what such a component must do. It uses no SDK.

Read [Script mods](SCRIPTS.md) first. It covers the quantum, commands, receipts, faults and the cost to a run. This page covers only what you need when you write the glue yourself.

## Why the world passes bytes

The world passes the snapshot, the events, every command and the config as bytes, and the snapshot, events and commands are msgpack. The config is named msgpack too, and it is empty bytes when the pack has no `config_schema`, so treat empty as the defaults. [Config](CONFIG.md) says how it is built. The saved state is opaque bytes: the format is yours. The world does not pass typed WIT records. That is on purpose. A typed record would break every mod, in every language, each time a field is added or a command changes. Bytes plus a documented encoding let the API grow without a break: a reader skips a key it does not know. The price is that your language gets no generated types. The ledgers and the JSON schema below are your types.

## Target the world

The world is `uptime-mod` in package `uptime:mod@0.1.0`. The file is `ModSDK/wit/uptime-mod.wit`. Point your toolchain's WIT binding generator at it.

The component imports one interface, `host`, with `log(level: u8, msg: string)`. It exports three functions:

| Export | Gets | Returns |
|---|---|---|
| `init` | `config: list<u8>`, `saved-state: option<list<u8>>` | ok, or an error string |
| `step` | `quantum` (`index`, `sim-time-ns`, `seed`), `snapshot: list<u8>`, `events: list<u8>` | a `step-output`, or an error string |
| `migrate` | `from-schema: u32`, `state: list<u8>` | the new state bytes, or an error string |

A `step-output` has four fields:

- `commands`: a list of byte lists. Each one is one command.
- `panels`: a list of a panel id and the bytes of one panel update. Leave it empty if you draw no panels. [Panels](PANELS.md) has the format.
- `toasts`: a list of byte lists. Each one is a toast, a named-msgpack map of `tone` (`"Neutral"`, `"Info"`, `"Good"`, `"Warn"` or `"Danger"`) and `text`. Leave it empty if you raise none. [Toasts](TOASTS.md) has the caps. The field is part of the WIT record, so a component built from an older copy of the WIT file needs rebuilding.
- `state`: the bytes to save, or none.

A component may import only `uptime:mod/host` and the WASI 0.2 interfaces a standard toolchain emits: clocks, random, io, cli (stdin, stdout, stderr, environment, exit) and terminal. The game serves them as stubs. The clocks return the quantum's sim time, random comes from the quantum's seed, stdout and stderr go to the mod's log, and `exit` traps. Any other import, such as the filesystem or sockets, fails to link, and the error names it. [Script mods](SCRIPTS.md) has the rule for determinism.

In the manifest, set `script.runtime` to `component` and `script.entry` to the `.wasm` file, such as `bin/mod.wasm`.

## The encoding

Everything is msgpack with named fields. A struct is a map keyed by field name. Never use the positional encoding, which writes a struct as an array. A new field would shift every later one.

Rules for all three directions:

- A struct is a map of field name to value.
- An enum is externally tagged. A variant with fields is a map with one key, the variant name, whose value is a map of the fields: `{"AcceptProspect": {"prospect_id": 7}}`. A variant with no fields is the bare name as a string.
- An optional field that is absent is either a missing key or a nil. Both appear: some fields inside a section are written as nil. Your decoder must treat both as absent. A top-level section is only ever a missing key, never a nil.
- Integers may be written at any width. Read every width: the encoder picks the smallest that fits.
- Unknown keys must be skipped. The game adds keys within a minor.

### Snapshot

The snapshot is one map. It starts with the envelope, `mod_api`, `quantum` and `sim_time_ns`, then holds the sections you declared in `reads`. A section you did not declare has no key. A declared section also has no key when the world has nothing for it, such as `active_attack` when no attack is running. A declared list section that exists with no rows is an empty array. A top-level section is never nil. Your decoder must not require a declared key.

The fields of every section are in `snapshot.v1alpha1.fields`. The same data as a JSON Schema is in `snapshot.v1alpha1.schema.json`. Generate your language's types from the schema if its tools can do that. Both files are in `ModSDK/schemas/`. The game's build generates them, so they match the game. This page does not repeat them.

The game sends a new snapshot on the first step and then every fourth quantum. On the quanta between, the host hands your step an empty byte list. Keep the last non-empty snapshot and use it. The envelope's `quantum` tells you how old it is.

### Events

`events` is one map, `quantum`, `dropped` and `events`. Each entry of `events` is a lifecycle marker, a receipt, an engine event or a panel input. `events.v1alpha1.fields` has the batch and the category that carries each engine event. Engine events arrive only for the categories you listed in the manifest's `events`, and receipts only when you list `receipts`. A panel input always reaches the mod that owns the panel, whatever you listed.

### Commands

Each element of `commands` is one command: the externally tagged map of one variant. `commands.v1alpha1.fields` lists every command and field. A field that has a default may be omitted. `AcceptProspect` needs only `prospect_id`.

The game decodes each command by itself. A command it cannot decode is dropped, and you get an `invalid_command` receipt with the command's position. The rest of the step still lands. A command outside the categories your manifest lists gets a `permission_denied` receipt.

## Limits

The caps on a step's output, the size of one command, the size of saved state and the size of one panel update are the constants in `ModSDK/wit/limits.rs`. The count of commands a step may return, and every fuel and memory limit, are in the table in [Script mods](SCRIPTS.md). This page does not copy them.

Time is fuel. Fuel counts wasm operations plus a fixed charge for each call into the host, so a limit breach happens on the same step every run. Your component decodes the snapshot itself, and that costs fuel. Declare the least you need in `reads`. [Script mods](SCRIPTS.md) has the full limits table, the decode allowance, strikes and rebuilds.

A language runtime that ships a garbage collector or a large standard library spends fuel on its own start up. Measure an empty step before you add logic, and keep the whole component within the memory limit in the [Script mods](SCRIPTS.md) table.

## Declare reads

Reads are declared in the manifest, not in the component:

```json
"reads": ["topbar", "prospects", "capacity"]
```

The game projects only the sections the loaded mods declare and gives you only yours. The names you may use are in [Script mods](SCRIPTS.md). The game checks each name when it loads the pack. Add a name when your code reads a section, and remove it when it stops.

## Save state

Return the bytes you want to keep in `state` from `step`. The format is yours: the game never looks inside. The rules:

- Return none to keep the last state. Return some to replace it.
- A state over the state cap in `limits.rs` is refused with a warning, and the game keeps the previous one.
- After a load, a strike that rebuilds the instance, or a reload, the game builds your component again and calls `init` with the config and the last accepted state. Anything you did not save is gone.
- Set `state_schema` in the manifest to a whole number and raise it when the shape of your state changes. The game calls `migrate(from-schema, state)` on a throwaway instance to turn old state into the new shape. That instance first runs `init(config, none)`, so `init` must succeed with no saved state, or the migration fails. `migrate` cannot read the snapshot or issue commands. If you do not handle a schema, return an error: the game turns the mod off for that save and shows a warning.
- Keep state a function of your config, your saved state and what each step is given. Then a run replays the same after a load.

## A worked example

`ModSDK/examples/raw-wat/guest.wat` is a complete component with no SDK. It is a few dozen lines of WebAssembly text. Each step it:

1. Finds `quantum` in the snapshot and reads the integer after it.
2. Issues `AcceptProspect` with that number as the prospect id.
3. Saves one byte, the count of steps it has run, and restores it in `init`.

It handles the empty snapshot: it issues nothing and still counts the step. It scans for the key bytes where a real mod would decode the map. That is the only shortcut in it. It reads no optional field, so it never meets a nil. Yours will.

Make a component from it with `wasm-tools`:

```
wasm-tools component embed ModSDK/wit guest.wat --world uptime-mod -o embedded.wasm
wasm-tools component new embedded.wasm -o bin/mod.wasm
```

The game's tests build this file the same way, load it and run it. They check that the command comes out at every integer width (fixint, u8, u16, u32 and u64) and that the state survives a reload.

## Check your component

`uptime-mod validate` checks the pack and links the component against the game's imports. `uptime-mod run` steps it against a scenario and prints each command that landed. See [The uptime-mod command](CLI.md).
