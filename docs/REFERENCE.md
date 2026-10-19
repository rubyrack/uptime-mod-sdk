# Reference

The mod API is described by a set of generated files. This page says what each one is and when to read it. The game's build generates all of them, and none is edited by hand. They are in `ModSDK/schemas/`.

## Ledgers

A ledger records every field of one part of the mod API, with a type and whether it is required. Each line has a name, a type, `required` or `optional`, and `@` with the minor the field first appeared in. A minor that broke a part of the API adds a `breaking` line for it. The first lines of each file say which way the data flows and which generation and minor it describes.

| File | What it is | Who writes, who reads |
|---|---|---|
| `manifest.v1alpha1.fields` | Every `manifest.json` field, with its type, its accepted size and the words an enumerated field takes. | A mod writes, the host reads. |
| `commands.v1alpha1.fields` | Every command a mod may issue, with every field. | A mod writes, the host reads. |
| `events.v1alpha1.fields` | The event batch envelope and the category that carries each engine event. | The host writes, a mod reads. |
| `snapshot.v1alpha1.fields` | Every field of every snapshot section. | The host writes, a mod reads. |
| `content.v1alpha1.fields` | The hardware rows and the config schema file, with the range each number accepts. | A pack writes, the host reads. |
| `scenario.v1alpha1.fields` | Every field a scenario pack's RON file may set. | A pack writes, the host reads. |

The direction decides what counts as a break. When the host reads, removing a field or adding a required one breaks every mod that was valid, and a new optional field does not. When a mod reads, a new field does not break a mod that ignores unknown fields.

A command field that names a workload or a part takes an id from the snapshot. The migrate commands' `unit_id` comes from the `workloads` section, and the part commands' `component_id` from the `parts` section. A mod reads either only when its manifest's `reads` names it. See [Script mods](SCRIPTS.md).

## Schemas

| File | What it is |
|---|---|
| `snapshot.v1alpha1.schema.json` | A JSON Schema for the whole mod snapshot. The Lua type stubs and the Rust snapshot types are generated from it. |
| `content.v1alpha1.schema.json` | A JSON Schema for the hardware pack files. Point your editor at it to get completion and the bounds on every number. |

## Leaves

`scenario.v1alpha1.leaves` lists every value a scenario RON file may set, traced from the scenario types. The scenario ledger is built from it. Read it when you need the exact name and type of a nested scenario field.

## How versions work

Every part of the mod API shares one generation, `v1alpha1`, and one minor. The header of each ledger says `generation v1alpha1, minor 0, shipped false`. That means the minor is still open and changes fold into it.

When a minor ships, each ledger gets a frozen copy and a digest. After that, any change to any file moves the minor. The game then checks each mod's `api_minor`:

- The game refuses by name a mod newer than the game.
- The game refuses by name a mod older than the latest break on a part of the API it uses.
- A mod built against an older minor, with no break since, still loads.

This SDK reads minor 0. Set `api_minor` in your manifest to the minor of the SDK you built against. In the Rust SDK that is `uptime_mod::snapshot::API_MINOR`. See [Content packs](CONTENT_PACKS.md) for the load rule.

While the generation is `v1alpha1`, a minor may contain a break. The loader then applies the rules in the list above.

## Other pages

- [Content packs](CONTENT_PACKS.md) for manifests, hardware, scenarios and archive rules.
- [Script mods](SCRIPTS.md) for what Lua and Rust mods share.
- [Lua mods](LUA.md) and [Rust mods](RUST.md) for each language.
