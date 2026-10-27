# Rust mods

A Rust mod is a WebAssembly component. You write it against the `uptime-mod` SDK, build it for `wasm32-wasip2` and ship the result as `bin/mod.wasm`. It reads the same snapshot, issues the same commands and gets the same events as a Lua mod, under the same limits. The shared rules are in [Script mods](SCRIPTS.md). This page covers what is specific to Rust.

## What you need

- A Rust toolchain with the `wasm32-wasip2` target: `rustup target add wasm32-wasip2`.
- The SDK crate, `uptime-mod`. It ships in `ModSDK/rust/uptime-mod` and is published on crates.io under the same name. `uptime-mod new --template rust` writes a path dependency on the `ModSDK` copy into your `Cargo.toml`; pass `--sdk <ModSDK folder>` if the command cannot find the folder. To use crates.io instead, write `uptime-mod = "0.2"` with the version from [Versions](#versions).
- The WIT file `ModSDK/wit/uptime-mod.wit`. The crate carries its own copy in `wit/` and reads it for you.

## The WIT world

A mod component exports one world, `uptime-mod`, from package `uptime:mod@0.1.0`. It imports one interface, `host`, with a single function: `log(level: u8, msg: string)`. Levels run from 0 (trace) to 4 (error).

It exports three functions:

| Export | Does |
|---|---|
| `init(config, saved-state)` | Builds the mod. `config` is bytes. `saved-state` is the last state, when there is one. Returns ok or an error message. |
| `step(quantum, snapshot, events)` | Runs one quantum. `quantum` is `{index, sim-time-ns, seed}`. `snapshot` and `events` are named msgpack. Returns commands, panels, toasts and an optional new state. |
| `migrate(from-schema, state)` | Rewrites state saved under an older `state_schema`. |

The SDK hides all of this. You implement a trait and call one macro.

## A first mod

```rust
struct Hello;

impl uptime_mod::Mod for Hello {
    fn init(_: uptime_mod::Init<'_>) -> Result<Self, String> {
        Ok(Self)
    }

    fn step(&mut self, step: &mut uptime_mod::Step<'_>) -> Result<(), String> {
        for p in step.snapshot.prospects.iter().flatten() {
            step.issue(&uptime_mod::ModCommand::AcceptProspect {
                prospect_id: p.id,
                isolate_dedicate: None,
                isolate_mode: None,
                isolate_segment_id: None,
            })?;
        }
        Ok(())
    }
}

uptime_mod::export_mod!(Hello);
```

Call `export_mod!` once per crate. The crate type is `cdylib`.

## The SDK

`Mod` has three methods:

- `init(Init) -> Result<Self, String>` builds the mod. `Init` holds `config` (bytes) and `saved_state` (optional bytes). An `Err` disables the mod with that message.
- `step(&mut self, &mut Step) -> Result<(), String>` runs once per quantum.
- `migrate(from_schema, state) -> Result<Vec<u8>, String>` has a default that refuses. Override it when you change your state shape.

`Step` holds:

| Member | What |
|---|---|
| `quantum` | `{index, sim_time_ns, seed}`. |
| `snapshot` | The latest snapshot, typed as `ModSnapshot`. |
| `events` | The event batch as raw named msgpack bytes. Typed events are not generated yet. The shape is in `events.v1alpha1.fields`. |
| `rng` | A stream seeded from `quantum.seed`. |
| `issue(&ModCommand)` | Queues a command. It lands two quanta later. |
| `panel(id, &PanelUpdate)` | Sets the values of a panel the pack declares. See [Panels](PANELS.md). |
| `toast(Tone, &str) -> Result<(), String>` | Raises a toast every player sees. The text is 1 to 160 bytes and a step raises at most 2. The call returns an error for a toast that breaks a cap, and the step may carry on. See [Toasts](TOASTS.md). |
| `save_state(Vec<u8>)` | Returns your state. The last call in a step wins. You choose the encoding. |
| `log(level, msg)` | Writes a log line. |
| `queued()` | The number of commands queued so far in this step. |

The host sends the snapshot bytes only when the snapshot refreshed. Otherwise it sends an empty list, and the SDK hands you the copy it already decoded. A mod never sees the difference.

Each section in `ModSnapshot` is an `Option`. A section you did not list in `reads` is `None`. A listed section that is empty is `Some` of an empty list. `uptime_mod::snapshot::SECTIONS` holds every name `reads` may use, and `ModSnapshot::sections()` the ones a snapshot carries.

`uptime_mod::units(&snapshot)` gives one row per host with its unit counts and service classes. It is empty when the mod did not read `hosts`.

`ModCommand` is an enum with one variant per command. Optional fields are `Option` and you must write each one, as in the example above.

Every tag enum in the SDK's snapshot types has an `Unknown` arm, and every struct ignores fields it does not know. A mod built against an older minor still reads a snapshot from a later one, until a minor declares a break. If you write your own decoder, for the events or for a part of the snapshot, give every enum a way to skip a value it does not know, or one new value will fail the whole batch. `#[serde(other)]` covers only a bare name such as `"Press"`, not a new kind with a payload such as `{"Slider": {...}}`. The example wraps each event and each UI value in an untagged enum whose last arm is `serde::de::IgnoredAny`, and skips that arm.

The SDK's `testing` feature builds snapshots from a partial JSON value for unit tests on your own machine. It is for the host target only and is never part of a mod build.

Set the manifest's `api_minor` to `uptime_mod::snapshot::API_MINOR`.

## Versions

The crate's version follows the mod API it reads. Each crate release reads exactly one generation and minor.

| Mod API | Crate version |
|---|---|
| `v1alpha1`, minor M | `0.(M+1).x`: minor 0 is `0.1.x`, minor 1 is `0.2.x` |
| a later alpha or beta generation | the next unused `0.y.x` |
| `vN` (a stable generation), minor M | `N.M.x` |

The last number, `x`, moves for an SDK fix that leaves the mod API alone.

Under an alpha generation every minor may break, and cargo treats each `0.y` as incompatible with the one before, so `uptime-mod = "0.2"` never moves you to a new minor by itself. To move to a new minor, raise the crate version and `api_minor` together. From `v1` on, a minor only adds, so `uptime-mod = "N.M"` picks up the later minors of `N`. When it does, set `api_minor` to the new `uptime_mod::snapshot::API_MINOR`.

The SDK in `ModSDK/rust/uptime-mod` and the crate on crates.io for the same game release are the same code.

## The manifest

```json
{
  "manifest_version": 1,
  "id": "com.example.my-mod",
  "version": "1.0.0",
  "api_version": "v1alpha1",
  "api_minor": 1,
  "game_version": ">=0.1.0, <0.3.0",
  "kind": ["script"],
  "script": { "runtime": "component", "entry": "bin/mod.wasm" },
  "affects_gameplay": true,
  "commands": ["finance", "customers"],
  "events": ["receipts", "customers"],
  "reads": ["topbar", "prospects", "capacity"]
}
```

The pack holds `manifest.json` and `bin/mod.wasm`. The game refuses any other file. Your source stays outside the pack. The entry must end in `.wasm`.

## Building to bin/mod.wasm

Build the crate in release mode for the wasip2 target, then copy the result into the pack:

```
cargo build --release --target wasm32-wasip2
cp target/wasm32-wasip2/release/my_mod.wasm my-pack/bin/mod.wasm
```

The file name under `release/` is your crate name with dashes turned into underscores.

This release profile in your `Cargo.toml` is a good start for a mod. It builds small and without panic machinery:

```toml
[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

The SDK installs a panic hook that writes the panic message to your log at level 4. With `panic = "abort"` the panic then traps, and a trap is a strike.

### What the component may import

A component built with `std` for `wasm32-wasip2` imports a set of WASI interfaces. The host serves all of them as inert stubs, and your mod calls none of them in practice. The stubs are:

- `wasi:io` error, poll and streams.
- `wasi:clocks` wall-clock and monotonic-clock. Both return the quantum's sim time.
- `wasi:random` random and insecure-seed. Both draw from the quantum's seeded stream.
- `wasi:cli` environment (empty), exit (traps), stdin (always closed), stdout and stderr (log lines at levels 2 and 3), and the terminal interfaces (none).

Any other import fails to link, for example the file system or sockets. The error names the import, and the world refuses to start. The host also refuses a component that defines its own resource types.

## Limits

The shared limits table is in [Script mods](SCRIPTS.md). The ones that matter most to a Rust author:

- A step has 20,000,000 fuel. A step that runs out is a strike.
- `init` has 200,000,000 fuel. The `init` that rebuilds your instance after a strike has 20,000,000, charged to the step that needed it.
- The whole mod gets 64 MiB of memory, for the linear memory and the GC heap together. Going over disables the mod at once.
- A step may issue 32 commands, and one encoded command may be 16 KiB.
- Saved state may be 1 MiB. The game refuses a larger state and keeps the old one.

Returning `Err` from `step` drops that step's commands and state. The game does not count it as a strike.

## Determinism rules

A run must replay exactly. The host feeds your mod only things a replay also has. Keep to these rules.

- Draw random numbers from `step.rng`. Do not read any other entropy.
- Do not use the wall clock. The clock stubs return sim time anyway, but your code should use `quantum.sim_time_ns`.
- Do not let hash order decide the order of commands. Use `BTreeMap` and `BTreeSet`, or sort before you issue. The example autoscaler keeps its pending orders in a `Vec` sorted by prospect id.
- Keep what must survive a fault in saved state. After an out-of-fuel or trap strike, or a load, your instance is built again and `init` gets the last saved state. Anything held only in memory is gone. The example autoscaler keeps its pending orders in saved state for this reason.
- Encode saved state in a way that does not depend on map order. The example uses `rmp_serde::to_vec_named` on a plain struct.
- Do not spawn threads. The host has none to give you.

The host runs the guest with fuel metering, canonical NaN values, no threads and no relaxed SIMD.

## The example

`ModSDK/examples/example-autoscaler` is the source of `ModSDK/examples/com.uptime.example-autoscaler/bin/mod.wasm`. It accepts the prospects the fleet can serve, at most 8 per quantum, and buys a host when free vCPU or memory runs low. It treats an empty config as the defaults and reads any other config as named msgpack. All of its memory is saved state: the orders in flight, the quantum of its last purchase, the pause switch and the last ten decisions. A mod rebuilt from that state plays on as if it never stopped. It has a config schema, two panels (the policy and the decisions, with a pause toggle) and `state_schema` 1, with a `migrate` that reads the schema 0 state.

It declares `reads` of `topbar`, `prospects` and `capacity`, and `commands` of `finance` and `customers`.

The Lua example, `ModSDK/examples/com.uptime.example-autoscaler-lua`, is the same policy. See [Lua mods](LUA.md).
