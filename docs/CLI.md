# The uptime-mod command

`uptime-mod` makes, checks, packages, runs and uploads a mod. It ships in the game's `ModSDK/bin` folder. It checks a pack with the same code the game uses. When `validate` passes a pack with no warnings, the game loads that folder as it is. When it passes with warnings, the game loads the `package` output and refuses the folder itself.

```
uptime-mod new --template content|lua|rust --id <mod id> [--title <title>]
               [--sdk <ModSDK folder>] [<dir>]
uptime-mod validate <dir> [--json]
uptime-mod package <dir> [--out <dir>]
uptime-mod run <dir> (--scenario <id> | --world <save>) --quanta <n>
               [--seed <n>] [--golden <file>] [--watch] [--runner <path>]
uptime-mod upload <dir> [--visibility private|friends|public]
                  [--change-note <text>] [--preview <image>] [--dry-run]
uptime-mod model <source.glb> --proxy <proxy.glb> --pack <dir> --item <item id>
```

`uptime-mod --help` prints this usage with a line on each command, and `uptime-mod <command> --help` prints the same text.

## Exit codes

| Code | Meaning |
|---|---|
| 0 | The command finished. |
| 1 | The pack has problems, or a step failed. |
| 2 | The command line is wrong, or `run` found no runner. |

`run` passes on the exit code of the game's runner. See [run](#run).

## new

`new` makes a pack folder from a template.

```
uptime-mod new --template lua --id com.example.my-mod
```

- `--template` is `content` (a hardware pack), `lua` or `rust` (a script mod). Each template is a working mod that `validate` accepts, except `rust`, which needs a build first. The command reports `bin/mod.wasm` as missing and tells you how to build it.
- `--id` is a reverse-DNS id such as `com.example.my-mod`.
- `--title` sets the display name. Without it, the title comes from the last part of the id, so `com.example.cool-gear` becomes `Cool Gear`. A title cannot hold quotes, backslashes or line breaks.
- `--sdk` is the `ModSDK` folder. Only the `rust` template uses it: the new `Cargo.toml` gets the absolute path of `rust/uptime-mod` in that folder. Without `--sdk`, the command uses `UPTIME_SDK`, then the `ModSDK` folder this `uptime-mod` runs from. If it finds none, it exits 1 and asks for `--sdk`.
- `<dir>` is the folder to make. It defaults to the id. The command refuses a folder that exists and is not empty.

The command replaces `{{id}}` and `{{title}}` in the template files, and `{{sdk_path}}` in the `rust` template, and changes nothing else.

## validate

`validate` checks a pack the way the game does and lists every problem.

```
uptime-mod validate com.example.my-mod
```

It runs the game's pack scan, which covers the manifest, the schemas, the archive rules and the API minor. For a script mod it then compiles the script the way the game does before a world runs. A component must link against the game's imports, and a Lua pack must compile with no syntax error.

Each problem is one line with the file, the field, the rule in brackets and a message. A clean pack prints `ok:` with the id, the version and the pack hash.

`--json` prints one JSON report with `ok`, `id`, `version`, `hash`, a `problems` list and a `warnings` list. Each problem and each warning has `file`, `field`, `rule` and `message`.

`validate` checks the pack without the files that belong to building a mod: dotfiles, `Thumbs.db`, `desktop.ini`, and the top-level `src`, `target`, `tests`, `dist`, `Cargo.toml`, `Cargo.lock` and `workshop.json`. `package` leaves the same files out, and writes a hard-linked file as a plain copy.

The game reads a pack folder as it is. It skips only the files an operating system makes on its own (`.DS_Store`, `Thumbs.db`, `desktop.ini`, `.localized` and `._*`), so it refuses a folder that holds any of the other build files or a hard link. When the pack itself is clean, `validate` also scans the folder as it is and prints each thing the game would refuse there as a `warning:` line. Those warnings do not fail the command, but they mean the folder must not be copied or linked into the mods folder. Put the `package` output there instead.

The exit code is 0 for a clean pack and 1 for any problem.

## package

`package` validates the pack, then writes the files the game accepts to `<out>/<id>-<version>/`. The default `<out>` is `dist`.

```
uptime-mod package com.example.my-mod --out dist
```

- The folder holds the accepted files and `uptime-package.json`, which records the id, the version and a SHA-256 hash of the files.
- The output depends only on the pack's files. It is sorted and carries no timestamps, so two runs give identical bytes.
- The command rescans its output and checks that it gives the same hash.
- It replaces an earlier package of the same name. It refuses a folder that exists and holds something else.
- It refuses an `--out` whose package folder would be the pack folder itself, would hold it, or would sit among the pack's own files. A `dist` folder inside the pack is fine.

A pack with any problem is not packaged, and the exit code is 1.

## run

`run` plays your pack in a scenario and prints what your mod did. It validates the pack, stages a copy under the pack id and starts the game's own `sim-runner` in batch mode on that copy.

```
uptime-mod run com.example.my-mod --scenario garage-to-glory --quanta 200
```

The runner prints one JSON line for each command that landed, with `issued`, `landed`, `mod`, `ok` and `code`. A run is the same every time for the same seed, and the seed is 1 unless you pass `--seed`.

Your mod's own log lines (`ctx:log` in Lua, `host.log` in a component) go to stderr, one per line, after the quantum that wrote them:

```
[mod-log] com.example.my-mod k=12 info: capacity low: buying a host
```

The level is `debug`, `info`, `warn` or `error`. Control characters are dropped and a line is cut at 512 bytes. The JSON lines on stdout stay as they were, so a `--golden` file is not affected.

| Flag | Meaning |
|---|---|
| `--scenario <id>` | A built-in scenario to build the world from. |
| `--world <save>` | A save file to load instead. |
| `--quanta <n>` | How many 15 second quanta to step. |
| `--seed <n>` | The scenario seed. `--world` ignores it. |
| `--golden <file>` | A file of expected output. The runner exits 1 and names the first differing line when the output does not match. |
| `--watch` | Copy changes from your pack folder into the running copy, so the runner reloads them between steps. Lua scripts, a component and panel files reload. A change to anything else prints a line saying what it needs: a new run for a scenario, a restart for the manifest or hardware. |
| `--runner <path>` | The `sim-runner` to use. |

The runner's exit code becomes the command's: 0 for a finished run, 1 for a golden mismatch and 2 for anything else it refuses. A pack with problems exits 1 before the runner starts, and the output says `nothing was run`.

`uptime-mod` looks for the runner in this order:

1. `--runner <path>`.
2. The `UPTIME_RUNNER` environment variable.
3. The game folder that holds `ModSDK`: `bin/<triple>/sim-runner`, where `<triple>` is the target for your machine, such as `aarch64-apple-darwin` or `x86_64-pc-windows-gnu`.
4. macOS only: the same game folder, `Uptime.app/Contents/MacOS/bin/<triple>/sim-runner`, where the app bundle keeps its runner.

When no runner is found, the command exits 2 and lists every place it looked.

A stop signal (Ctrl-C) goes to the runner, and `run` removes its staged copy when the runner exits. A second stop ends a runner that ignored the first.

## upload

`upload` packages the pack the way `package` does, checks the result again and publishes it as a Steam Workshop item. `--dry-run` prints what it would send and sends nothing.

```
uptime-mod upload com.example.my-mod --dry-run
```

The item id is kept in `workshop.json` in your pack folder, so each later upload updates the same item. A new item is private. Exit codes are 0 for a finished upload or dry run, 1 when the pack or Steam refuses it and 2 for a wrong command line. [Workshop](WORKSHOP.md) has the flags, visibility, updates and the legal agreement.

## model

`model` builds the three files of a new host or switch model from the GLB you export from your modelling tool, writes them into the pack, then checks them the way the game does.

```
uptime-mod model work/box.glb --proxy work/box_proxy.glb --pack com.example.my-mod --item box
```

- `<source.glb>` is your full model: the body, the ports, the lights and the marker empties, as one `.glb` with its data and textures inside. Keep it outside the pack folder. The command refuses a source inside `--pack`, since the game refuses any file a pack does not load.
- `--proxy` is the far model you made by hand. The command copies it unchanged.
- `--pack` is the pack folder. The files go in its `art/models/` folder, and the command replaces any earlier files of the same item.
- `--item` is the id of a new host or switch in the pack's `hardware` rows. An override of a built-in SKU cannot ship a model.

It writes:

| File | What it holds |
|---|---|
| `art/models/<item>_render.glb` | The near model. Every drawn node is placed in model space and merged into one primitive per material, under one mesh node. The command leaves out LED lenses, empties and every extension but `KHR_texture_transform`. |
| `art/models/<item>.pips.json` | Where the port lights, ports, fans, power inlets, module bays and nameplate sit, read from your node names. |
| `art/models/<item>_proxy.glb` | The copy of `--proxy`. |

### Naming the nodes

The sidecar comes from the node names alone, so name them the way the base models are named. The command ignores a trailing suffix such as `.001`.

| Node name | Read as |
|---|---|
| `Port_1`, `Port_2`, ... | A cabled data port, numbered from 1. Its position is the cable anchor. |
| `Port_<name>`, such as `Port_BMC` | A named port. `Port_Console`, `Port_USB_Mgmt`, `Port_Mgmt` and `Port_iDRAC` are management pins and carry no lights. |
| A mesh named `LED_Link...` or `LED_Activity...` | The link or activity light of the port it sits under (a `Port_<N>` or `PortSlot_<N>` parent), or else of the nearest port. |
| A mesh with `LED` and `PSU` in its name or material | A PSU light. The command numbers PSU lights from the rightmost, as it numbers the inlets. |
| A mesh with `LED` and `power` or `_PWR` in its name, or `_PWR` in its material | The power light. |
| A mesh `StatusLED_SYS`, `StatusLED_ALM` or `StatusLED_FAN` | The system, alarm or fan status light. Any other mesh with `LED` in its name is a status light with no role. |
| A node ending `_IEC` or `AcInlet`, or `_RearIEC_<n>` | A power inlet, one per PSU. |
| `Bay_1`, `Bay_2`, ... | A module bay frame. |
| `FanRotor...` | A spinning fan blade. |
| A mesh `Nameplate_Plate`, or else a node `_NameDisplay` | The name placard. With neither, the sidecar has no nameplate. |

Every light is a mesh with `LED` in its name. Its box sets the size of the light the game draws. The command also leaves out of the render model any primitive whose material has `LED` in its name.

### The checks

After writing, the command runs the game's pack scan and prints what it refuses in this item's model, grouped by rule. For example:

```
refused [model_over_limit]:
  art/models/box_render.glb /materials the list has more than 8 entries
box: 1 model problem(s)
```

A clean model prints `ok: the game's checks pass box's model`. The counts must match the base model the row names: the same number of cabled ports, one inlet per PSU, and no bays or the base model's bays. The model must fit the base model's rack slot, with at most 8 materials. A default export usually needs work before it passes:

- Join or reuse materials until there are 8 or fewer. The command merges by material, so the material count is the primitive count.
- The game draws the lights from the sidecar, so no material needs to glow. The game refuses a material that emits light.
- A texture is a PNG inside the GLB, at most 2048 by 2048.

The command counts problems elsewhere in the pack but does not list them. Run `validate` for those. The exit code is 0 when the model passes, 1 when the game refuses it or a step failed, and 2 for a wrong command line.
