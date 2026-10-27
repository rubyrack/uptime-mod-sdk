# Modding Uptime

This folder holds the guides for people who make mods. A mod is a folder with a `manifest.json`. It is either a content pack, which adds hardware, scenarios, customers or services, or a script mod, which runs code against a running world. A mod can be both.

## Start here

| Guide | Read it to |
|---|---|
| [Quickstart](QUICKSTART.md) | Make, check, run and package a first mod, and preview its upload. |
| [The uptime-mod command](CLI.md) | Use `new`, `validate`, `package`, `run` and `upload`, and read their exit codes. |
| [Content packs](CONTENT_PACKS.md) | Write a manifest, add hardware or scenarios, and learn what makes the game refuse a pack. |
| [Equipment models](MODELS.md) | Give a new host or switch its own 3D model: the three files, the conventions, every limit and the `.pips.json` fields. |
| [Script mods](SCRIPTS.md) | Learn what Lua and Rust mods share: declared reads, the 15 second quantum, commands and receipts, limits, faults, and what a modded run costs the player. |
| [Lua mods](LUA.md) | Write a mod in Lua: the entry table, `require`, the sandbox, determinism and hot reload. |
| [Rust mods](RUST.md) | Write a mod in Rust: the SDK, building to `bin/mod.wasm`, and determinism. |
| [Other languages](OTHER_LANGUAGES.md) | Make a component in TinyGo, JavaScript or any language with no SDK: the WIT world, the msgpack encoding, the limits and the state contract. |
| [Panels](PANELS.md) | Draw screens in the game: panel files, widgets, data timing, player input, text rules, the HUD card and keys. |
| [Toasts](TOASTS.md) | Raise a short notice every player sees: the call, the tones and the caps. |
| [Config](CONFIG.md) | Offer settings with a config schema, and set them on a dedicated server. |
| [Workshop](WORKSHOP.md) | Upload a mod, choose who sees it, push updates, and learn what players and dedicated server operators do with it. |
| [Multiplayer](MULTIPLAYER.md) | Learn which packs other players must install to join a world with your mod, and how panels behave with several players. |
| [Reference](REFERENCE.md) | Find the generated schemas and ledgers, and learn how API versions work. |

## Examples

The game's `ModSDK/examples` folder carries ten working mods (in the repository they are in `mods/examples/`):

- `com.uptime.example-hardware` adds host and switch SKUs and overrides one built-in SKU.
- `com.uptime.example-model` adds a switch SKU with its own 3D model. The model files are game art, not MIT, see `TERMS.md`.
- `com.uptime.example-scenario` adds a scenario and depends on the hardware pack.
- `com.uptime.example-customers` adds two customer archetypes and changes one built-in archetype.
- `com.uptime.example-service` adds a `Cold Storage` offering over object storage and a customer archetype that buys it.
- `com.uptime.example-events` adds two macro events and their English text.
- `com.uptime.example-locale` carries text in English and German.
- `com.uptime.example-autoscaler` is a Rust script mod.
- `com.uptime.example-autoscaler-lua` is the same policy as a Lua script mod.
- `com.uptime.example-alerts` is a Lua script mod with a HUD card, a toast and two keys. It issues no command.

## Which API version this reads

These guides describe API generation `v1alpha1`, minor `1`. Set `api_minor` in your manifest to the minor of the SDK you built against. [Reference](REFERENCE.md) explains the rule.
