# Reference

The mod API is described by a set of generated files. This page says what each one is and when to read it. The game's build generates all of them, and none is edited by hand. They are in `ModSDK/schemas/`.

## Ledgers

A ledger records every field of one part of the mod API, with a type and whether it is required. Each line has a name, a type, `required` or `optional`, and `@` with the minor the field first appeared in. A minor that broke a part of the API adds a `breaking` line for it. The first lines of each file say which way the data flows and which generation and minor it describes.

| File | What it is | Who writes, who reads |
|---|---|---|
| `manifest.v1alpha1.fields` | Every `manifest.json` field, with its type, its accepted size and the words an enumerated field takes. | A mod writes, the host reads. |
| `commands.v1alpha1.fields` | Every command a mod may issue, with every field, and the `ModToast` a step may return. | A mod writes, the host reads. |
| `events.v1alpha1.fields` | The event batch envelope and the category that carries each engine event. | The host writes, a mod reads. |
| `snapshot.v1alpha1.fields` | Every field of every snapshot section. | The host writes, a mod reads. |
| `content.v1alpha1.fields` | The hardware rows, the panel file (its points, including `hud`, and the `key` of a panel or button) and the config schema file, with the range each number accepts. | A pack writes, the host reads. |
| `scenario.v1alpha1.fields` | Every field a scenario pack's RON file may set. | A pack writes, the host reads. |

The direction decides what counts as a break. When the host reads, removing a field or adding a required one breaks every mod that was valid, and a new optional field does not. When a mod reads, a new field does not break a mod that ignores unknown fields.

A command field that names a workload or a part takes an id from the snapshot. The migrate commands' `unit_id` comes from the `workloads` section, and the part commands' `component_id` from the `parts` section. A mod reads either only when its manifest's `reads` names it. See [Script mods](SCRIPTS.md).

## Schemas

| File | What it is |
|---|---|
| `snapshot.v1alpha1.schema.json` | A JSON Schema for the whole mod snapshot. The Lua type stubs and the Rust snapshot types are generated from it. |
| `content.v1alpha1.schema.json` | A JSON Schema for the hardware pack files. Point your editor at it to get completion and the bounds on every number. |

## Leaves

The ledgers do not cover a pack model's `.pips.json` sidecar. Its fields, limits and rule codes are in [Equipment models](MODELS.md#the-pipsjson-sidecar). `uptime-mod validate` and `uptime-mod model` check it with the game's own code.

`scenario.v1alpha1.leaves` lists every value a scenario RON file may set, traced from the scenario types. The scenario ledger is built from it. Read it when you need the exact name and type of a nested scenario field.

## How versions work

Every part of the mod API shares one generation, `v1alpha1`, and one minor. The header of each ledger says `generation v1alpha1, minor <!-- mod:minor -->10<!-- /mod:minor -->`. Minors <!-- mod:shipped -->0 to 10<!-- /mod:shipped --> have shipped: their frozen copies are <!-- mod:frozen -->`<surface>.v1alpha1.0.fields` to `<surface>.v1alpha1.10.fields`<!-- /mod:frozen -->.

When a minor ships, each ledger gets a frozen copy and a digest. After that, any change to any file moves the minor. The game then checks each mod's `api_minor`:

- The game refuses by name a mod newer than the game.
- The game refuses by name a mod older than the latest break on data it uses. A break in one snapshot section reaches only the script mods whose `reads` name that section. A break on one scenario field reaches only the packs whose scenario files set it, and a removed scenario enum variant reaches only the packs whose scenario files name it. Any other break reaches every mod written against that part of the API.
- A mod built against an older minor, with no break on data it uses, still loads.

The breaks that shipped, minor by minor:

<!-- mod:history -->
- Minor 1 breaks two things. In `active_incidents`, `articles` became `article_ids`. In scenario files, `pricing.load_balancer.per_million_requests_microcents` and `tuning.eviction_refund_cap_months` are gone. A mod built against minor 0 that reads `active_incidents`, or sets either field, must be rebuilt against minor 1. Every other minor-0 mod still loads.
- Minor 2 breaks nothing.
- Minor 3 breaks port numbers, which are now 16-bit. Every `port` and `index` on the snapshot's port lists and cables changed, as did every `CableEndpoint` port in commands. The seven per-kind `revenue_breakdown` fields (`vms_dollars`, `managed_db_dollars`, `k8s_dollars`, `cdn_dollars`, `object_store_dollars`, `load_balancer_dollars`, `functions_dollars`) are removed; read `revenue_breakdown.services`, one `class_tag` and `dollars` entry per service. `DiagnoseCause` gains the required `entity_kind` and `sub_id`. `BuyHostNicModule` is removed; a host network card is bought with `BuyModule`, whose `install` names the cage to seat it in. Because commands changed, the game refuses every script mod built against minor 2 or earlier, whatever it reads. Rebuild a Rust mod with uptime-mod 0.4; a Lua mod sets `api_minor` to 3 in its manifest and replaces any `BuyHostNicModule` call. Minor 3 also removes the lose conditions `DataLossEvent` and `AuditFailure` from `rules.loses`, because nothing in the game could trigger them. A scenario pack built against minor 2 or earlier that names either one is refused by name and must be rebuilt against minor 3 without it. Other scenario packs are not affected.
- Minors 4, 5, 6 and 7 break no declared field. They add fields and the `device_naming` section. In minor 4, a customer's `p99_latency_ms`, `p99_recent_ms` and `p50_latency_ms` are `None` while the figure is stale.
- Minor 8 breaks two things in scenario files and the naming fields of the snapshot. In scenario files, the scripted events' `at_secs` and `ScheduleCableKnockLoose.delay_secs` are renamed `at_real_secs` and `delay_real_secs`, and count real seconds at 1x instead of game seconds. A pack built against minor 7 or earlier that sets the old names is refused by name. In the snapshot, hosts, switches, gateways, appliances and edds lose `display_name` and `display_name_plain` (hosts also lose `label`), and `stranded_vms[].host_display_name` becomes `host_name`. `device_naming` loses `suffixed`, `kind_words`, `rack_rows`, `racks[].n` and the `admits` word, host ids and numbers. Read `name` and `name_plain` on every named row, and `stored_name` for a rename prefill. A script mod built against minor 7 or earlier that reads one of these sections is refused by name.
- Minor 9 breaks no declared field. `deliveries[].location` can now read `Pushable`; a mod that matches the location must accept it. It adds the crash cart events (`PushableKitFitted`, `PushableSold`, `UpsBridgeCarrying`, `UpsBridgeEmpty`), the `Pushable` created-ref in a command receipt, the `CrashCartRacksReach` quest trigger, and `ruled_out` and `ruled_out_by` on the causes of `active_incidents`.
<!-- /mod:history -->

The current minor is not in this list until it ships. A break it declares appears in the ledgers as a `breaking` line.

Slots count from 0 in every command field (`install.slot`, `InstallModule.slot`) and in the snapshot. The `slot` argument in a card command's reply or refusal text counts from 1, because it is the number the console shows the player; read the slot from the snapshot, not from reply text.

This SDK reads minor <!-- mod:minor -->10<!-- /mod:minor -->. Set `api_minor` in your manifest to the minor of the SDK you built against. In the Rust SDK that is `uptime_mod::snapshot::API_MINOR`. See [Content packs](CONTENT_PACKS.md) for the load rule.

While the generation is `v1alpha1`, a minor may contain a break. The loader then applies the rules in the list above.

## Other pages

- [Content packs](CONTENT_PACKS.md) for manifests, hardware, scenarios and archive rules.
- [Equipment models](MODELS.md) for pack 3D models and the `.pips.json` sidecar.
- [Script mods](SCRIPTS.md) for what Lua and Rust mods share.
- [Panels](PANELS.md) for panel files, the HUD point and keys, and [Toasts](TOASTS.md) for the notices a step may raise.
- [Lua mods](LUA.md) and [Rust mods](RUST.md) for each language.
