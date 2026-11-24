# Content packs

A content pack adds hardware, scenarios, customers, services, events or text to the game. It holds data files and no code. This page covers the manifest, the hardware, scenario, customer, service, event and locale files, the archive rules and what makes the game refuse a pack.

Script mods use the same manifest. The script fields are in [Script mods](SCRIPTS.md).

## What a pack is

A pack is a folder. The game scans a mods folder, and each subfolder is one pack. `manifest.json` sits at the root of the pack folder.

```
com.example.my-hardware/
  manifest.json
  hardware/hosts.json
  hardware/switches.json
```

The game reads a pack where it lies. It does not unpack or copy anything. It never follows a link inside a pack.

## Where the mods folder is

The game keeps its mods folder in its user data folder. Find it here:

- macOS: `~/Library/Application Support/Uptime/mods/`
- Windows: `%APPDATA%\Uptime\mods\`
- Linux: `~/.local/share/Uptime/mods/`, or `$XDG_DATA_HOME/Uptime/mods/` when `XDG_DATA_HOME` is set.

Each subfolder is one pack. If the folder does not exist, create it. Restart the game, or reopen Mods, to see a new pack.

## The manifest

`manifest.json` is strict. A field the game does not know is an error, and so is an unknown value in a list. The file may not exceed 64 KiB.

```json
{
  "manifest_version": 1,
  "id": "com.example.my-hardware",
  "version": "1.0.0",
  "api_version": "v1alpha1",
  "api_minor": 10,
  "game_version": ">=0.1.0, <0.3.0",
  "kind": ["content"],
  "content": ["hardware"],
  "affects_gameplay": true
}
```

| Field | Required | What it holds |
|---|---|---|
| `manifest_version` | yes | The manifest format. It must be `1`. The game checks it before any other field. |
| `id` | yes | The pack's name. Two or more dot-separated segments, each `[a-z][a-z0-9_-]*`, at most 128 bytes in all. |
| `version` | yes | A semver version of the pack. |
| `api_version` | yes | The mod API generation. The only word accepted is `v1alpha1`. |
| `api_minor` | no | The mod API minor the pack was built against. It defaults to `0`. |
| `game_version` | yes | A semver range of game versions the pack works with. |
| `kind` | yes | A non-empty list of `content` and `script`. |
| `content` | with `kind` content | The categories the pack carries: `hardware`, `scenarios`, `customers`, `services`, `events`, `locale`, `art`. It must be non-empty when `kind` has `content`, and it must be absent otherwise. |
| `script` | with `kind` script | A block with `runtime` and `entry`. It is covered in [Script mods](SCRIPTS.md). |
| `affects_gameplay` | yes | Whether the pack changes play. The game checks the value, see below. |
| `commands` | no | Command categories a script mod may issue: `infrastructure`, `finance`, `customers`, `incidents`. |
| `events` | no | Event categories a script mod is told about: `lifecycle`, `receipts`, `incidents`, `customers`, `finance`, `builds`. |
| `reads` | no | Snapshot sections a script mod reads. |
| `panels` | no | Panel files, at most 32. See [Panels](PANELS.md). |
| `config_schema` | no | A file that describes the mod's config. See [Config](CONFIG.md). |
| `state_schema` | no | A number that versions the mod's saved state. It defaults to `0`. |
| `dependencies` | no | Other packs this one needs, at most 64. See below. |

Lists may not repeat a word. A path in the manifest must pass the path rules below, and the file must exist in the pack.

### The affects_gameplay check

The game works out the right value for `affects_gameplay` and refuses the manifest when yours differs. The value is `true` when the pack lists any `commands` category or carries `hardware`, `scenarios`, `customers`, `services` or `events`. A pack with only `locale` or `art` content, and no commands, is `false`.

Packs that affect play mark the run. See [Script mods](SCRIPTS.md) for what that costs a player.

### api_version and api_minor

`api_version` names a generation. `api_minor` names the minor within it. Every part of the mod API shares one minor: the snapshot, the events, the commands, the hardware, customer and service files, the scenario files and the manifest.

The game accepts an `api_minor` between two limits. The low limit is the latest minor that declared a break on data your pack uses: a scenario field its scenario files set, a removed scenario enum variant its scenario files name, or any break on the hardware, customer and service files or the manifest. The high limit is the minor this build serves. A pack outside that range is incompatible and the game says so by name. A pack built against an older minor still loads while no break on its data has landed since.

A hardware row that sets a field added at minor <!-- mod:minor -->10<!-- /mod:minor -->, such as `name`, `rack_units` or `ports`, needs an `api_minor` of at least <!-- mod:minor -->10<!-- /mod:minor -->. The game does not check this when it loads your pack, but a game that serves an older minor refuses the field as unknown. `uptime-mod validate` reports it as `api_minor_too_low`, see [Hardware errors](#hardware-errors).

The current minor is `<!-- mod:minor -->10<!-- /mod:minor -->`; minors <!-- mod:shipped -->0 to 10<!-- /mod:shipped --> have shipped. A scenario pack built against an older minor is refused by name when a later minor removed or renamed a scenario field or enum variant its files use. [Reference](REFERENCE.md#how-versions-work) lists each minor's breaks. While `v1alpha1` is a preview, a minor may contain a break. The ledgers in [Reference](REFERENCE.md) record every change.

## Hardware packs

Declare `"content": ["hardware"]`. The game loads two files and no others from `hardware/`:

- `hardware/hosts.json` holds `{ "hosts": [ ... ] }`, at most 128 rows.
- `hardware/switches.json` holds `{ "switches": [ ... ] }`, at most 128 rows.

The game refuses any other file under `hardware/` with the rule `content_category_unsupported`.

### The base-model rule

A row names a built-in SKU in `model`. That base supplies every field the row leaves out, so a row that sets only stats (the way packs worked before shapes) loads unchanged. A new row can also declare its own shape and kind: rack units, depth, power supplies, ports, module slots, architecture, accelerator and category on a host, and rack units, depth, power supplies, port groups, class, MLAG and role on a switch. Only some of these change what a model must draw, see [What a row's shape needs](#what-a-rows-shape-needs). Every row rejects unknown keys.

Two things are different about a new look and a new shape, and the difference decides what you ship:

- **A new look on a base.** The row keeps the base's shape and only changes stats, or draws its own model inside the base's slot with the base's port, power and bay counts. See [Equipment models](MODELS.md#a-new-look-or-a-new-shape).
- **A new hardware shape.** The row declares a shape that differs from its base. The base model cannot draw it, so the row must ship its own model, or share another row's model with `art`. The pack is refused with `shape_needs_model` otherwise.

An override row never sets shape. It replaces a built-in SKU's stats in place, keeps its shape and always draws as that SKU.

Copy base SKU names exactly from the built-in catalog, for example `Balanced x86 1U`, `Compute 1U` and `16-port Access GbE`. The GPU host is `GPU 2× 2U` and the name contains the character `×`. Run [`uptime-mod bases`](CLI.md#bases) to list every base with its rack units, depth in millimetres, envelope, ports, power supplies and scalar defaults. Its `--json` output holds a `row` object per base, spelled as a row's fields, to paste into a row and keep the base's shape.

### Host rows

| Field | Notes |
|---|---|
| `id` | Required. The item name in the pack. |
| `model` | Required. The base host SKU the row starts from. |
| `overrides` | Optional. `base:<name>` to replace a built-in SKU. |
| `vcpu`, `mem_mb`, `ssd_gb`, `ssd_iops`, `power_w` | Optional numbers. |
| `capex`, `hourly_cost` | Optional money, see below. |
| `mtbf_hours`, `generation_year` | Optional numbers. |
| `accel_count`, `gpu_mem_mb_per_gpu` | Optional. Above zero only when the row's resolved accelerator is not none. |
| `name`, `description` | Optional. See [Names and descriptions](#names-and-descriptions). An absent name shows the SKU name, `<pack id>:<id>`. |
| `art` | Optional, new rows only. Another new host of this pack whose model draws this one. See [Sharing a model](#sharing-a-model-with-art). |
| `rack_units` | Optional, new rows only. 1 to 10. Absent is the base's. |
| `depth_mm` | Optional, new rows only. 100 to 1070. The chassis depth, and the depth the rack fit checks. Absent is the base's. |
| `psu_count` | Optional, new rows only. 1 to 8. One rear power inlet each. Absent is the base's. |
| `ports` | Optional, new rows only. The numbered data ports, as 1 to 8 banks. See [Port banks](#port-banks). Absent is the base's banks. |
| `module_slots` | Optional, new rows only. 0 to 4 cages that take a bought NIC module. `0` means a fixed factory port set with no NIC to buy. Absent is the base's. |
| `arch` | Optional, new rows only. `x86_64`, `arm64`, `riscv` or `mainframe`. Absent is the base's. |
| `category` | Optional, new rows only. `balanced`, `memory_opt`, `compute_dense`, `storage`, `gpu` or `network_appliance`. Absent is the base's. |
| `accelerator` | Optional, new rows only. `none`, `gpu`, `tpu` or `fpga`. Absent is the base's. `none` clears `accel_count` and `gpu_mem_mb_per_gpu`. |

The management or BMC port is not a data port. It stays separate from the numbered ports and a row does not list it.

### Switch rows

| Field | Notes |
|---|---|
| `id` | Required. The item name in the pack. |
| `model` | Required. The base switch SKU the row starts from. |
| `overrides` | Optional. `base:<name>` to replace a built-in SKU. |
| `capex`, `hourly_cost` | Optional money, see below. |
| `power_w`, `fabric_capacity_gbps`, `pps_ceiling`, `mtbf_hours` | Optional numbers. |
| `name`, `description` | Optional. See [Names and descriptions](#names-and-descriptions). An absent name shows the SKU name, `<pack id>:<id>`. |
| `art` | Optional, new rows only. Another new switch of this pack whose model draws this one. See [Sharing a model](#sharing-a-model-with-art). |
| `rack_units` | Optional, new rows only. 1 to 10. Absent is the base's. |
| `depth_mm` | Optional, new rows only. 100 to 1070. The chassis depth. Absent is the base's. |
| `psu_count` | Optional, new rows only. 1 to 8. One rear power inlet each. Absent is the base's. |
| `port_groups` | Optional, new rows only. The numbered front ports as 1 to 8 banks. See [Port banks](#port-banks). Absent is the base's groups. |
| `class` | Optional, new rows only. `l2` parks loops, `l3` also routes. Absent is the base's. |
| `mlag` | Optional, new rows only. Whether two of these pair into one MLAG system. A pair needs a managed switch with no stack ports and at least two data ports, and adds the rear `Port_MlagKeepalive` jack, which the row's sidecar must hold in `named_ports`. Absent is the base's. |
| `role` | Optional, new rows only. `access`, `tor`, `leaf`, `aggregation` or `spine`. A label for grouping and the shop. Absent is the base's. |

Each number has a floor and a ceiling. The exact bounds are in `content.v1alpha1.schema.json`, described in [Reference](REFERENCE.md). A value outside them fails with the rule `min` or `max` and names the field.

Money fields are whole numbers in minor units. One dollar is 100,000,000 units. The example hardware pack sets `capex` to `220000000000`, which is 2,200 dollars.

### Port banks

A host's `ports` and a switch's `port_groups` list banks. A bank is a run of identical data ports:

```json
{ "media": "sfp28", "speed_gbps": 25, "count": 4 }
```

| Field | Notes |
|---|---|
| `media` | The cage: `rj45`, `sfp_plus`, `sfp28`, `sfp56`, `sfp112`, `qsfp_plus`, `qsfp28`, `qsfp56` or `qsfp_dd`. |
| `speed_gbps` | The line rate of every port in the bank, 1 to 400. At most what the cage carries: 10 for `rj45`, and the cage's own rate for the others. Over it fails with `port_speed_unsupported`. |
| `count` | The ports in the bank, 1 to 255. |

Ports are numbered from 1, bank by bank: with the banks `2x rj45` then `4x sfp28`, ports 1 and 2 are the copper ports and 3 to 6 the cages. A model's `.pips.json` numbers its `ports` the same way, see [Equipment models](MODELS.md#the-pipsjson-sidecar). A host may list up to 64 data ports in all, and a switch up to 128.

A switch row that declares `port_groups` and no `fabric_capacity_gbps` gets a non-blocking fabric over its ports and a wire-speed packet ceiling. Set either field to choose your own. A modular chassis grows its ports from the cards seated in it, so `port_groups` on one fails with `modular_switch_ports`.

A host's declared ports replace the base's numbered ports, and the NIC count and speed the game reads follow them. A bought NIC module adds its ports above the numbered ones, so `module_slots: 0` gives a host that only ever has the ports you listed.

### Names and descriptions

`name` and `description` set the text the game shows for the item: the shop card, the host console, the switch panel and the tooltip. A value is one of:

- Plain text. A `name` is at most 48 bytes and a `description` at most 200. A value over its cap fails with `max`, a value with a control or invisible formatting character fails with `wrong_type`, and an empty value fails with `min`.
- `@` and one of your own `en` locale keys, such as `@mod-com-example-my--hardware-agg24_desc`. The text then shows in the player's language like any other locale text. The key must be one of the pack's own: its prefix plus a name, see [The key prefix](#the-key-prefix), defined in one of your `locale/en/` files. A key that is not your pack's fails with `locale_key_not_namespaced`, and one with no `en` text with `locale_key_missing_en`. A key of another pack, a dependency's included, is not accepted. A pack that uses a key declares `"locale"` in `content`.
- `@@` and plain text, for plain text that starts with `@`.

The text a key resolves to obeys the same limits as plain text: at most 48 bytes for a `name`, 200 for a `description`, one line, and no control or invisible formatting character. The game loads a pack whose key text breaks them, so `uptime-mod validate` is what reports it, with the rules `max`, `wrong_type` and `min`. The check follows terms, messages and their attributes, and takes a select expression's variant that a term call's literal argument picks, else the default one; a `$variable`, a function call or a missing reference is known only in play, so text that holds one skips the `min` check and is judged on the rest.

Commands and saves keep naming the item `<pack id>:<id>`; the name is only what the player reads. An item with no `name` shows its SKU name, `<pack id>:<id>`, and not the bare `id`. An override may set `name` and `description`, and then relabels the built-in SKU it replaces.

### Sharing a model with art

Two rows that look the same need one model, not two. A row's `art` names another new row of the same pack and family (a host names a host, a switch names a switch) whose three model files draw both:

- The named row ships its own model. Naming a row that ships none fails with `art_has_no_model`.
- The two rows resolve to the same shape: rack units, depth, port cages and order, power supplies, module bays, front drive bays, the MLAG jack and the stack ports. A row takes its drive bays from its base, so a host's base must have as many drive bays as the base of the row `art` names. Stats, names, prices, port speed and class may differ. A different shape fails with `art_shape_mismatch`.
- A row that names `art` ships no model of its own. A row with its own files that also names `art` fails with `art_with_own_model`.
- `art` names no other new row of the family in this pack: `art_unknown_item`. An override row may not set `art`.

A row takes every field it leaves out from its `model`, the built-in base, and never from the row `art` names. `art` shares the drawing only: a row with `art` and no `power_w` gets the base's power, not the shared row's.

The textures stay inside the model's GLB, so sharing copies nothing.

### What a row's shape needs

A new row whose resolved shape differs from its base's needs a model, from its own files or through `art`. Without one the pack fails with `shape_needs_model`, which names the three files to add. The shape is what the model draws: rack units, depth, port cages and order, power supplies, module bays, front drive bays (a host draws one drive per bay), the MLAG jack and the stack ports. Port speed, `arch`, `category`, `accelerator`, `class`, `role`, names, prices and every other stat are not shape: a row that changes only those keeps its base's shape and needs no model. A depth you copy from `uptime-mod bases` is the base's `depth_mm`, in whole millimetres, so declare that number to keep the base's shape.

The model is checked against the row's resolved shape and not the base's. Its geometry must fit the slot the row's `rack_units` and `depth_mm` make, its `ports` must number exactly the row's data ports, its `iec` must hold one inlet per `psu_count`, and its `bays` must be empty or hold one frame per module slot. See [Equipment models](MODELS.md).

### Worked example

This pack adds a 3U host, a cheaper variant of it that shares its model, and a 1 Gbps L3 switch. `manifest.json`:

<!-- worked-example: manifest.json -->
```json
{
  "manifest_version": 1,
  "id": "com.example.my-hardware",
  "version": "1.0.0",
  "api_version": "v1alpha1",
  "api_minor": 10,
  "game_version": ">=0.1.0, <0.3.0",
  "kind": ["content"],
  "content": ["hardware", "art", "locale"],
  "affects_gameplay": true
}
```

`hardware/hosts.json`:

<!-- worked-example: hardware/hosts.json -->
```json
{
  "hosts": [
    {
      "id": "rack3-edge",
      "model": "Balanced x86 2U",
      "name": "Edge Rack 3U",
      "description": "Three rack units with fixed 2x1G and 4x25G data ports.",
      "rack_units": 3,
      "depth_mm": 800,
      "psu_count": 2,
      "ports": [
        { "media": "rj45", "speed_gbps": 1, "count": 2 },
        { "media": "sfp28", "speed_gbps": 25, "count": 4 }
      ],
      "module_slots": 0,
      "vcpu": 64,
      "mem_mb": 262144,
      "power_w": 650,
      "capex": 900000000000,
      "hourly_cost": 40000000
    },
    {
      "id": "rack3-edge-lite",
      "model": "Balanced x86 2U",
      "art": "rack3-edge",
      "name": "Edge Rack 3U Lite",
      "rack_units": 3,
      "depth_mm": 800,
      "psu_count": 2,
      "ports": [
        { "media": "rj45", "speed_gbps": 1, "count": 2 },
        { "media": "sfp28", "speed_gbps": 25, "count": 4 }
      ],
      "module_slots": 0,
      "vcpu": 32,
      "mem_mb": 131072,
      "capex": 500000000000
    }
  ]
}
```

`hardware/switches.json`:

<!-- worked-example: hardware/switches.json -->
```json
{
  "switches": [
    {
      "id": "agg24",
      "model": "24-port Access GbE",
      "name": "Aggregation 24",
      "description": "@mod-com-example-my--hardware-agg24_desc",
      "rack_units": 1,
      "depth_mm": 400,
      "psu_count": 2,
      "port_groups": [{ "media": "rj45", "speed_gbps": 1, "count": 24 }],
      "class": "l3",
      "mlag": true,
      "role": "aggregation",
      "capex": 90000000000
    }
  ]
}
```

`locale/en/main.ftl`:

<!-- worked-example: locale/en/main.ftl -->
```
mod-com-example-my--hardware-agg24_desc = A 24-port gigabit L3 switch with two power supplies. Two of them pair into one MLAG system.
```

What it declares:

- `rack3-edge` is a 3U host with six numbered data ports: ports 1 and 2 are 1G copper and ports 3 to 6 are 25G SFP28 cages. `module_slots: 0` makes them its factory ports, with no NIC to buy. Its management port is separate and is not in `ports`. Every field left out, such as `ssd_gb` and `arch`, is `Balanced x86 2U`'s.
- `rack3-edge-lite` declares the same shape and names `art`, so it ships no model of its own and the game draws it with `rack3-edge`'s. It names `art` for the drawing only, so every stat it leaves out, such as `power_w` and `hourly_cost`, is `Balanced x86 2U`'s and not `rack3-edge`'s. Its name and price are its own.
- `agg24` is a 1U switch with two power supplies, 24 gigabit ports, L3 routing and MLAG, and a description that names its own locale text. Two of them pair, which adds the rear `Port_MlagKeepalive` jack. Its fabric is non-blocking over the 24 ports.

The pack ships `art/models/rack3-edge_render.glb`, `rack3-edge_proxy.glb` and `rack3-edge.pips.json` for the host, and the three files named `agg24` for the switch, made as [Equipment models](MODELS.md) describes. The host's sidecar numbers `ports` 1 to 6 and holds two `iec` entries and no `bays`. The switch's numbers `ports` 1 to 24, holds two `iec` entries, and holds `Port_MlagKeepalive` in `named_ports`. `uptime-mod validate` accepts this pack once those files are in place, and it is the quickest way to check yours.

### Hardware errors

These rules apply to hardware rows, on top of the number bounds above:

| Rule | Cause and fix |
|---|---|
| `shape_needs_model` | The row's resolved shape differs from its base and it has no model. Add the three model files named after the row, or set `art` to another row's, or remove the shape fields. |
| `art_unknown_item` | `art` names no other new row of this pack and family. Fix the id, and check the row is a host for a host. |
| `art_has_no_model` | The row `art` names ships no model. Add that row's three files. |
| `art_with_own_model` | The row ships its own files and also sets `art`. Delete one of them. |
| `art_shape_mismatch` | The two rows resolve to different shapes. Make rack units, depth, port cages, power supplies, module slots, drive bays (through the base), MLAG and stack ports match, or give the row its own model. Port speed and class may differ. |
| `port_speed_unsupported` | A bank's `speed_gbps` is above what its cage carries. Lower the speed or pick a faster cage. |
| `modular_switch_ports` | `port_groups` on a modular chassis. Remove it, a modular chassis takes its ports from cards. |
| `mlag_unsupported` | A switch that is MLAG, by `mlag: true` or by its base, cannot run a pair: unmanaged, stackable, or with fewer than two data ports. It is also reported at `/port_groups` when a row leaves an MLAG base with fewer than two ports. Pick a managed base, or give it two data ports. |
| `pips_named_port_missing` | A row that declares `mlag` ships a sidecar whose `named_ports` leaves out a rear port the switch has, such as `Port_MlagKeepalive`. Add it; `uptime-mod bases` lists the names. |
| `gpu_memory_missing` | A GPU host with no `gpu_mem_mb_per_gpu`, from the row or its base. Set it. |
| `override_field_forbidden` | A shape field, `art`, or a port bank on an override row. An override keeps its base's shape. Make it a new row. |

`uptime-mod validate` and `uptime-mod model` add checks the game does not run. `model` runs the model checks on its own item even when the rest of the pack has other problems. The game still loads a pack that breaks them, because a refused pack would sell the hardware players own from it, so fix them before you publish:

| Rule | Cause and fix |
|---|---|
| `pips_named_port_missing` | On a row that takes MLAG or stack ports from its base, the sidecar's `named_ports` leaves one out. Add it, so the game can draw the jack and its cable. |
| `port_faces_inward` | A port's +Y axis does not point out of the panel it sits on. Rotate the node in your source and export again, see [Port orientation](MODELS.md#port-orientation). |
| `gpu_without_accelerators` | A GPU host whose `accel_count` is 0. Set `accel_count` above zero. |
| `gpu_host_undersized` | A GPU host whose vCPU or memory is below what its GPU slices draw. Raise `vcpu` or `mem_mb`. |
| `api_minor_too_low` | A row sets a hardware field that minor <!-- mod:minor -->10<!-- /mod:minor --> added (`name`, `description`, `art`, `rack_units`, `depth_mm`, `psu_count`, `ports`, `port_groups`, `module_slots`, `arch`, `category`, `accelerator`, `class`, `mlag` or `role`) and the manifest declares a lower minor. Raise `api_minor` to at least that minor. An older game refuses these fields as `unknown_field`. |
| `max`, `wrong_type`, `min` | On a `name` or `description` that is an `@` key: the key's `en` text is over 48 or 200 bytes, runs over more than one line or holds a control character, or is empty. Shorten the text. |

### Ids and overrides

An item `id` matches `[a-z0-9][a-z0-9_-]{0,63}`. A new row becomes the SKU named `<pack id>:<id>`. Scenarios and `sku_gates` use that full name.

A row with `"overrides": "base:<name>"` replaces that built-in SKU's stats in place and keeps its SKU name. It may also set `name` and `description`, which replace the text the game shows for the built-in SKU. Three rules apply:

- The text after `base:` must name a built-in SKU of the same family.
- The row's `model` must equal that same name.
- One pack may override a target once.

Only built-in SKUs can be overridden. If two selected packs override the same built-in SKU, the one that loads last wins. Load order is the dependency order, and ties go to the smaller pack id.

## Scenario packs

Declare `"content": ["scenarios"]`. A scenario is a RON file at `scenarios/<item>.ron`. The file name stem must match the item id pattern above. The game refuses files in subfolders of `scenarios/`, and any file that is not `.ron` or not UTF-8 text.

The scenario's own `id` must equal `<pack id>:<file stem>`. The rule `scenario_id_not_namespaced` fires otherwise.

A scenario `config` block may set only these keys: `tuning`, `sku_gates`, `pricing`, `prospects`, `staff`, `bootstrap`, `component_lifecycle`, `relationship`, `consequences`, `service_investment`, `service_plane`, `rewards`, `quests`, `coop`, `bailout` and `depreciation`. The same holds for the `config` inside `hard_mode`. A pack scenario may not set `catalogs`, `steam` or `sites`. New SKUs come from hardware packs, and the achievement catalog and the sites belong to the base game. Every SKU a scenario names must exist in the packs you selected or in the base game. Every `archetype` a seeded customer or prospect carries must be an archetype id or name from the base game, the scenario's own archetypes or a selected customer pack, or it fails with `unknown_archetype`.

Two top-level fields decide who can play the scenario. `coop` says whether players can join it in multiplayer and on a dedicated server, and defaults to `true`; write `coop: false` for a single-player scenario. `versus` defaults to `false`. Both are separate from the `config.coop` key above, which only tunes difficulty scaling for co-op runs. A dedicated server refuses a pack scenario that says `coop: false`, so ship the scenario without that line unless it is meant for one player. Never ask operators to edit an installed pack: the edit breaks its Workshop package hash.

The field-by-field shape of a scenario file is in `scenario.v1alpha1.fields`.

## Customer packs

Declare `"content": ["customers"]`. A customer pack adds customer types, which the game calls archetypes, and can change the built-in ones. The game loads one file and no others from `customers/`:

- `customers/archetypes.json` holds `{ "archetypes": [ ... ] }`, at most 64 rows.

The game refuses any other file under `customers/` with the rule `content_category_unsupported`.

### The model rule

A row names a built-in archetype in `model` and sets demand on top of it: the tier, what the customer asks for, how it grows and when it leaves. Everything else comes from the model: how the customer talks, how patient it is during an incident, its sales cycle and its compliance needs. Every row rejects unknown keys.

The built-in archetype ids are lowercase words such as `hobbyist`, `freelance_dev`, `local_business`, `etsy_seller` and `agency`.

### Archetype rows

| Field | Notes |
|---|---|
| `id` | Required. The item name in the pack. |
| `model` | Required. The built-in archetype the row starts from. |
| `overrides` | Optional. `base:<id>` to change a built-in archetype. |
| `name` | New rows only, where it is required. The name players see, at most 48 bytes of plain text. |
| `tier` | Optional. `bronze`, `silver`, `gold` or `platinum`. A fixed tier replaces the model's tier and any range of tiers. |
| `workload_from` | Optional, new rows only. A built-in archetype whose main service this one asks for. |
| `extra_workloads_from` | Optional, new rows only. Up to 4 built-in archetypes whose main services this one adds. It replaces the model's extra services; `[]` asks for the main service alone. |
| `growth_chance_per_check`, `viral_spike_chance_per_check` | Optional, 0 to 1. How often a happy customer asks for more. |
| `plateau_at_allocation` | Optional, 1 to 1000. The size at which the customer stops asking for more. |
| `payload_ceiling_mult`, `payload_growth_per_30_days`, `site_share` | Optional. How far and how fast the customer's traffic grows. |
| `signup_bonus_min`, `signup_bonus_max` | Optional. The signup bonus range in whole dollars, up to 1,000,000. The minimum may not be above the maximum. |
| `competitor_poach_chance` | Optional, 0 to 1. The chance per month a competitor takes the customer. |
| `outage_tolerance_minutes` | Optional. Outage minutes before the customer starts to leave. |
| `referral_chance_when_happy` | Optional, 0 to 1. The chance per month a happy customer sends you a new prospect. |
| `exposure_weight`, `attack_event_chance_per_month` | Optional. How much attacker attention the customer draws. |
| `requires_multi_az` | Optional. The contract needs two availability zones. |
| `min_rep` | Optional, new rows only, 0 to 1. The reputation at which this archetype's prospects start to arrive. |
| `service` | Optional, new rows only. A service offering, as `<pack id>:<item>`, that the archetype's workload of that offering's kind is sold as. See [Service packs](#service-packs). |

The exact bounds are in `content.v1alpha1.schema.json`. A value outside them fails with the rule `min` or `max` and names the field.

A row asks for services by naming built-in archetypes, not services. Every service a built-in archetype asks for is open to yours, including VMs, managed databases and CDNs. A name in `workload_from` or `extra_workloads_from` must be a built-in archetype, or it fails with `unknown_model`.

### Ids, names and overrides

A new row becomes the archetype `<pack id>:<id>`. Prospects of it arrive like any other: the game adds it to the prospect roster with its `min_rep`, unless the scenario's own roster names it already. A pack scenario can name `<pack id>:<id>` in its `prospects` roster to place it on its own ladder. Customer names follow the model's naming pattern.

Each archetype's `name` must be its own. A new row fails with `archetype_name_taken` when its name is a built-in archetype's name, another row's name in your pack or in any selected pack, a built-in archetype id such as `agency`, or shaped like `<pack id>:<id>`. The game shows the name as plain text.

A row with `"overrides": "base:<id>"` changes that built-in archetype in place and keeps its id. The text after `base:` must name a built-in archetype, the row's `model` must equal it, and one pack may override a target once. An override keeps its archetype's name, services and place on the roster, so `name`, `workload_from`, `extra_workloads_from`, `min_rep` and `service` fail on an override with `override_field_forbidden`. If two selected packs override one archetype, the one that loads last wins.

A new row starts from its model as the selected overrides leave it.

### Removing a customer pack

A save keeps the customers its packs brought. When you load it without the pack, each customer of one of its archetypes becomes a customer of the built-in archetype the row was built from. It keeps its name, its contract and every workload it runs. Only its persona changes. The game drops waiting prospects of the pack's archetypes, and undoes any change the pack made to a built-in archetype.

## Event packs

Declare `"content": ["events", "locale"]`. An event pack adds macro events: the big outside shocks, such as a chip shortage or a power crisis, that a scenario draws a few of for each run. The game loads one file and no others from `events/`:

- `events/templates.json` holds `{ "events": [ ... ] }`, at most 32 rows.

The game refuses any other file under `events/` with the rule `content_category_unsupported`.

### The model rule

A row names a built-in macro event in `model`, and the model decides what the event does: which prices and costs go up, how lead times change, and how customers react. The row sets the news text, when the event may start, how long it lasts and how often the draw picks it. Every row rejects unknown keys. An event pack adds no new kind of trouble, only new stories for the kinds the game already has.

The models are `nand_shortage`, `gpu_allocation_crisis`, `regional_power_crisis`, `data_residency_law`, `submarine_cable_cut`, `major_cve`, `rival_price_war`, `acquisition_offer`, `zero_day_worm` and `ai_runaway_job`.

### Event rows

| Field | Notes |
|---|---|
| `id` | Required. The item name in the pack. |
| `model` | Required. The built-in macro event the row starts from. |
| `start_text` | Required. A key of your pack's locale text, posted to `#news` when the event starts. |
| `end_text` | Required. A key of your pack's locale text, posted to `#news` when the event ends. |
| `duration_hours_min`, `duration_hours_max` | Optional, 1 to 2160. How long the event lasts. With neither set the scenario's range applies; with one set the event lasts exactly that long. The minimum may not be above the maximum. |
| `weight` | Optional, 1 to 100. How likely the draw is to pick this event. Every built-in event weighs 10, which is also the default. |
| `when` | Optional. World facts the event waits for, all of them, on top of the day the draw gives it. |

`when` takes these fields, each optional:

| Field | Notes |
|---|---|
| `after_day` | 0 to 3650. The event starts on this day of the run at the earliest. |
| `min_customers` | Up to 100,000. The event waits until you have this many customers. |
| `min_cash_dollars` | 0 to 1,000,000,000,000. The event waits until you hold this much cash. |
| `min_reputation` | 0 to 1. The event waits until your reputation reaches this. |
| `min_progression_step` | Up to 64. The event waits until the campaign reaches this step. |

The exact bounds are in `content.v1alpha1.schema.json`. A value outside them fails with the rule `min` or `max` and names the field.

### Event text

The text is never written in the row. `start_text` and `end_text` name messages in your pack's locale files, so the text shows in the player's language and voice like any other locale pack text. See [Locale packs](#locale-packs).

- A key that is not your pack's prefix plus a name with no dash, such as a sentence or another pack's key, fails with `event_text_not_owned`.
- A key that is not in one of your `locale/en/` files fails with `event_text_missing`.

```
mod-com-example-weather-heat_start = A heatwave is pushing the grid hard. Power costs more until it breaks.
    .eng = Heatwave: grid at peak load, utility power prices up.
mod-com-example-weather-heat_end = The heatwave is over. Power prices are back to normal.
```

Because an event pack carries locale text, it is client-required: a player who joins a multiplayer world that uses it must have it installed.

### When an event runs

A new row becomes the event `<pack id>:<id>`. Events come from a scenario's macro event pool. Your events join the draw of every scenario whose pool is turned on and lists your model, or lists no kinds at all. A scenario with no pool, such as the sandbox, never draws them, and a scenario that leaves your model out of its pool does not draw that row.

The draw picks events by weight without picking one kind twice, so your row and its model never run in the same draw. A run's draw is fixed by its scenario and seed: the same packs and seed always give the same events. With no event packs selected, every scenario draws exactly as before.

### Removing an event pack

When you load a save without the pack, the events from it that had not started yet are dropped. An event from it that is running keeps going with its model's effects and ends with the game's own text.

## Service packs

Declare `"content": ["services"]`. A service pack adds service offerings and can change the built-in service kinds. The game loads one file and no others from `services/`:

- `services/services.json` holds `{ "services": [ ... ] }`, at most 32 rows.

The game refuses any other file under `services/` with the rule `content_category_unsupported`.

### The kind rule

A row names a built-in service kind in `model` by its tag: `vm`, `fn`, `obj`, `db`, `cdn`, `k8s` or `lb`. An offering is a product line of that kind. Its units are the kind's units: they are placed, moved, served and repaired exactly as the kind's are. A row sets plain values only, the shop name, the unlock gate, the spread grain and the price. It cannot change what a unit is or how it behaves. Every row rejects unknown keys.

### Service rows

| Field | Notes |
|---|---|
| `id` | Required. The item name in the pack. |
| `model` | Required. The built-in kind the offering runs as. |
| `overrides` | Optional. `base:<tag>` to change a built-in kind. |
| `name` | The name players see in the shop. Required on a new row, refused on an override. At most 48 bytes, plain text. |
| `unlock_cost` | Optional. The one-time cost to sell the offering while it is locked, in money minor units, see [Host rows](#host-rows). |
| `enabled_by_default` | Optional. Whether it sells from the start. A new offering without it does. |
| `spread_grain` | Optional. The blast radius the kind's units are spread across: `host`, `switch`, `az`, `region`, `power` or `router`. Only `obj`, `db`, `cdn`, `k8s` and `lb` spread their units, so the field fails on `vm` and `fn` with `service_spread_unsupported`. A tenant's own redundancy contract can still ask for a coarser grain. An offering sets the grain only, never a count of domains: the game shares a kind's units evenly across the domains at that grain. |
| `price_multiplier` | Optional, 0.1 to 10. Scales every billed line of the kind. |

The exact bounds are in `content.v1alpha1.schema.json`.

### Who buys an offering

A new row becomes the offering `<pack id>:<id>`. Only a customer of a pack archetype whose `service` names it buys it, and only for its workload of the offering's kind. Built-in customers never do. The archetype may come from the same pack or from another selected pack, and `service` must name an offering of a selected pack, or the selection fails with `unknown_service`.

An offering that starts locked shows in the shop's Services page under its kind with an Invest button. Its kind must be sold first. A prospect who buys a locked offering cannot sign until you unlock it.

Each offering's `name` must be its own. A name that is a built-in kind's name, or another offering's in your pack or in any selected pack, fails with `service_name_taken`.

### Overrides

A row with `"overrides": "base:<tag>"` changes that built-in kind for every tenant that runs it, and keeps its tag. The text after `base:` must name a built-in kind, the row's `model` must equal it, and one pack may override a kind once. If two selected packs override one kind, the one that loads last wins. An override's `unlock_cost` and `enabled_by_default` replace the scenario's gate for the kind.

### Removing a service pack

A save keeps the tenants that bought an offering. When you load it without the pack, each of them runs on as the built-in kind, with the kind's own grain and price. Every unit stays where it is. An unlock you paid for is kept, and applies again if the pack comes back.

## Locale packs

Declare `"content": ["locale"]`. A locale pack carries text for your mod in one or more languages, as [Fluent](https://projectfluent.org/) files:

```
com.example.words/
  manifest.json
  locale/en/main.ftl
  locale/de/main.ftl
```

The game loads `locale/<lang>/<name>.ftl` and nothing else under `locale/`. `<lang>` is a language tag: two or three lowercase letters, then any number of `-` subtags of two to eight letters or digits, such as `en`, `de` or `pt-BR`. `<name>` matches the item id pattern above. Any other file under `locale/` fails with the rule `content_category_unsupported`.

### The key prefix

Every message in your files is your pack's prefix followed by a name. The prefix is `mod-`, then your pack id with each `.` written as `-` and each `-` written as `--`, then `-`. The name after it has no dash: use letters, digits and `_`. The pack `com.example.night-shift` owns `mod-com-example-night--shift-`, so its messages look like this:

```
-pack-name = Night Shift

mod-com-example-night--shift-title = { -pack-name }
mod-com-example-night--shift-greeting = Welcome to the night shift.
    .eng = On call. Pager is live.
mod-com-example-night--shift-open_tickets = { $count ->
    [one] One ticket waits for you.
   *[other] { $count } tickets wait for you.
}
```

Every key belongs to exactly one pack id: everything before its last dash is the prefix, and the prefix turns back into one id. So no pack can define another pack's text. No game key starts with `mod-`, so a pack adds text and never changes the game's own. A message that is not your prefix plus a dash-free name fails with the rule `locale_key_not_namespaced`.

Terms, the ids that start with `-`, are private to your pack and can use any name. A message can use your own terms and messages, not the game's.

The `.eng` attribute is the text shown when the player has engineer terms on. Without it, both settings show the message's value.

### Languages

The player's language picks your text. When your pack has no text for a message in that language, the game shows your `en` text. So every message in another language must also be in `locale/en/`, or it fails with the rule `locale_key_missing_en`.

A pack cannot add a language to the game. Text in a language the game does not ship is never shown, since the player cannot pick that language.

### Locale rules

- A file is at most 256 KiB and a pack's locale files are at most 1 MiB together. Over either fails with `max`.
- A file must be UTF-8 text, or it fails with `wrong_type`.
- A file must parse as Fluent. A parse error fails with `locale_syntax` and the message gives the line.
- Text a message shows holds no control character other than a newline and no invisible formatting character (bidirectional marks, overrides and isolates, zero-width spaces, tag characters, the byte order mark, the Arabic letter mark, soft hyphens). The zero-width joiner and non-joiner are allowed. The same holds for a string literal, escapes included. Breaking it fails with `locale_syntax`.
- A message nests placeables and function calls at most 8 deep, and one entry holds at most 256 `{` and `(` together. Past either fails with `locale_syntax`.
- A placeable cannot sit directly inside another placeable, as in `{ { $n } }`, and a select cannot choose on a literal, as in `{ "x" -> ... }`. Both fail with `locale_syntax`. Write `{ $n }`, and select on a variable or a function.
- A number in your text is at most 32 characters with at most 18 digits after the point, or it fails with `locale_syntax`.
- A message or term defined twice in one language, in one file or two, fails with `duplicate_id`.

`uptime-mod validate` runs these same checks.

A locale pack does not change play, so `affects_gameplay` is `false`. It is client-required: a player who joins a multiplayer world that uses it must have it installed. See [Multiplayer](MULTIPLAYER.md). The game loads the text of every locale pack installed and valid, enabled or not, so a joiner sees the host's text. The text comes from the same read of your files that the join's pack hash covers.

Panel and config titles and labels are plain text, or `@` and one of your keys. See [Panels](PANELS.md#labels-that-name-a-locale-key).

The game reads the text again when the set of installed packs changes, such as after a Workshop install or removal, so a new pack's words show without a restart.

## Art

Declare `"content": ["art"]`. Art belongs to the pack's own new hardware items, so the pack also carries those rows. The game loads two kinds of art file, icons and models. Every other file under `art/` fails with the rule `content_category_unsupported`.

A pack with `art` content is client-required: a player who joins a multiplayer world that uses it must have it installed. See [Multiplayer](MULTIPLAYER.md). It cannot also carry a script.

### Icons

An icon is a PNG at `art/icons/<item>.png`, where `<item>` is the id of one of the pack's own hardware items. It is at most 64 KiB and at most 256 by 256 pixels. The shop shows it on that item's row.

### Models

A new host or switch can ship its own 3D model, drawn in the rack in place of its base model. A model is three files in `art/models/` named after the item: `<item>_render.glb`, `<item>_proxy.glb` and `<item>.pips.json`. A model that breaks any rule makes the whole pack invalid. [Equipment models](MODELS.md) covers how to make the files, every limit and the `.pips.json` fields.

## Dependencies

`dependencies` maps a pack id to a semver range:

```json
"dependencies": {
  "com.uptime.example-hardware": ">=1.0.0 <2.0.0"
}
```

The game refuses a pack that depends on itself. A dependency must be installed, must match the range and must be able to load. The player must also select it. The game loads dependencies first. A dependency cycle makes every pack on the cycle incompatible. If a dependency cannot load, the packs that need it cannot load either, and the message carries the root cause.

## Updating a pack that a saved world uses

A save records each pack it was played with: the pack id, its `version`, and one content hash. The hash is one SHA-256 over the path and bytes of every file in the pack, in path order, except `uptime-package.json` and `workshop.json`. A change to any file changes it, `manifest.json` included. The save also records whether the pack changes gameplay.

### Which version loads

When a save loads, the installed copy of each pack must satisfy a caret range built from the version the save recorded, so a save at `0.5.0` needs `^0.5.0`. This is the same caret rule as a Cargo or npm range:

| Saved version | Installed version | Loads |
|---|---|---|
| `0.5.0` | `0.5.3` | Yes |
| `0.5.0` | `0.6.0` | No |
| `1.2.0` | `1.9.0` | Yes |
| `1.2.0` | `2.0.0` | No |
| `0.0.3` | `0.0.4` | No |

A pack that does not satisfy the range counts as not installed. What follows depends on the pack:

- A pack that changes gameplay, such as hardware, scenarios, customers, events or services, is then missing, and the load is refused. The game names the missing pack ids, and the player installs a copy in range.
- A pack that does not change gameplay, such as a locale or art-only pack, is dropped, and the world loads without it and with a warning.

So a version bump that you want existing worlds to keep playing stays inside the caret range: `0.5.x` after `0.5.0`, and `1.x` after `1.0.0`. A bump past it, such as `0.5.0` to `0.6.0`, strands every world saved with the old version until the player installs a copy in range. The version a world loads with is then the installed one, and the world records that version for its next save.

### The same version with changed files

An installed copy in range whose hash differs from the one the save recorded still loads. The game plays the installed files, shows the player a warning that the pack has been updated since the save was made, and records the new hash in the world. A save from before hashes were kept has no hash to compare and loads with no warning.

### What happens to hardware the player owns

When the pack's hardware rows change between the save and the load, the game brings the player's owned items in line with the installed rows as the world loads:

| What changed in the row | Owned items |
|---|---|
| A host's numbers: vCPU, memory, disk, IOPS, power or cost per hour | An owned host takes the new row, as long as the row is a drop-in and the host's own workloads still fit the new numbers. The price the player paid is kept as paid. |
| A switch's numbers: power, cost per hour, fabric capacity, packet rate ceiling, mean time between failures or role | An owned switch takes the new row when the row is a drop-in for it. The price the player paid is kept as paid. |
| A host's shape: rack height, chassis depth, architecture, accelerator, module slots, drive bays, power supply count or port banks | The owned host keeps the shape it was bought with. The game draws it with the row's own model while rack height, chassis depth, port media and their order, power supplies, drive bays and module slots are unchanged, so a change to architecture, accelerator or port speed alone keeps it. When any of those physical things changed, it draws with the look of the built-in model it was drawn by when that model still has the owned shape, and as a plain chassis otherwise. New purchases use the new row. |
| A switch's shape: rack height or shelf footprint, port cages and their order, MLAG, module slots, stack ports, power supply count or chassis depth | The owned switch keeps the shape it was bought with, drawn the same way as a host's. New purchases use the new row. |
| A switch's port speed, switch class or fan count, its shape unchanged | A class upgrade, to a class that does everything the old one does, with nothing else changed is a drop-in: the owned switch takes the new row. A port speed change, a class downgrade or a fan change leaves the owned copy as it was bought. New purchases use the new row. |
| A different pack takes over a host's SKU name, and the row fits in place of the owned host (same rack height, chassis depth, module slots, power supplies and port cages; port speed may differ) | A drop-in row, which also matches port speeds, drive bays, architecture, accelerator and accelerator count, is taken by the owned host when the host's workloads fit it. A row that fits in place but is not a drop-in, or that the host's workloads do not fit, leaves the owned copy as it was bought. A row that does not fit in place removes the host and pays the player its resale value. |
| A different pack takes over a switch's SKU name, and the row fits in place of the owned switch (same rack height, port cages, MLAG, module slots, stack ports and power supplies; port speed and class may differ) | A drop-in row, which also matches port speeds, fan count, chassis depth and shelf footprint and has a class that does everything the old one does, is taken by the owned switch. A row that fits in place but is not a drop-in, such as a port speed change or a class downgrade, leaves the owned copy as it was bought. |
| A different pack takes over a switch's SKU name, and the row does not fit in place | The owned switch is removed and the player is paid its resale value. |
| The row is gone, while the pack is still installed | The owned item is removed and the player is paid its resale value. A basket line or a box on its way for it is dropped too. |

A host whose workloads would not fit the new numbers also keeps its old copy. Keep a SKU's shape fixed across versions. Remove a SKU only when you mean the player to sell it back.

### Packs that carry models

A hardware pack with models declares `"content": ["hardware", "art"]`. That makes it both a gameplay pack and a client-required pack. As a gameplay pack, a saved world refuses to load without it. As a client-required pack, a player joining a multiplayer world that uses it needs the same files, see [Multiplayer](MULTIPLAYER.md#which-packs-a-joiner-needs).

## Archive rules

These rules apply to every file in a pack. The scan checks the pack folder against them before it reads any content.

- `manifest.json` must be at the root.
- Every file the manifest points at must exist: `script.entry`, each `panels` entry and `config_schema`.
- A path uses only `A-Z`, `a-z`, `0-9`, `.`, `_`, `-` and `/`.
- A path is at most 240 bytes. It may not be empty, absolute or contain a backslash or a percent sign. It may not have a segment that is empty, is `.`, ends in a dot, or is made only of dots.
- The part of a segment before its first dot may not be a Windows device name: `con`, `prn`, `aux`, `nul`, `com1` to `com9` or `lpt1` to `lpt9`. So the game refuses `con.txt` too.
- A pack holds at most 8192 entries.
- One file may not exceed 32 MiB, and all files together may not exceed 256 MiB. The game counts the bytes it reads, not the size a header claims.
- The game refuses symlinks and special files. On unix it refuses a file with more than one name as well.
- Two paths that differ only in letter case collide, and the game refuses them.
- A file cannot also be a folder.
- The game skips `.DS_Store`, `Thumbs.db`, `desktop.ini`, `.localized` and files that start with `._`.

Every other file must sit under the folder of a category the manifest declares, or be a file the manifest references. A stray file fails with the rule `content_outside_declared_category`. A Lua script pack also allows its `.lua` files, see [Lua mods](LUA.md).

## What refuses a pack

The scan reports each problem with the pack id, the file, a JSON pointer to the field and a stable rule code. A pack lands in one of three states: valid, invalid or incompatible.

An invalid pack has bad files. Nothing from it loads. These rule codes mark it:

| Rule | Cause |
|---|---|
| `manifest` | `manifest.json` is missing a field, has an unknown field or breaks a manifest rule. |
| `archive` | A path, size, count, link or collision rule failed. |
| `unknown_field`, `missing_field`, `wrong_type` | A row has a key the type does not declare, lacks a required key, or has a value of the wrong type. |
| `min`, `max` | A number is below its floor or above its ceiling. |
| `id_pattern` | An item id breaks the id pattern. |
| `duplicate_id` | Two items, or two overrides of one target, in one pack. |
| `unknown_model`, `model_family_mismatch` | `model` names no built-in SKU or archetype, or a SKU of the other family. |
| `override_not_base`, `override_target_missing`, `override_model_mismatch` | An `overrides` rule failed. |
| `unknown_sku` | A scenario names a SKU that exists nowhere. |
| `unknown_archetype` | A scenario seeds a customer or prospect under an archetype that exists nowhere. |
| `scenario_id_not_namespaced`, `scenario_config_key_forbidden`, `scenario_catalog_override_forbidden` | A scenario rule failed. |
| `archetype_name_taken`, `override_field_forbidden` | A customer rule failed, or a shape field was set on a hardware override. See [Customer packs](#customer-packs) and [Hardware errors](#hardware-errors). |
| `shape_needs_model`, `art_unknown_item`, `art_has_no_model`, `art_with_own_model`, `art_shape_mismatch` | A hardware row's shape has no model to draw it, or its `art` is wrong. See [Hardware errors](#hardware-errors). |
| `port_speed_unsupported`, `modular_switch_ports`, `mlag_unsupported`, `gpu_memory_missing` | A hardware row declares a port, switch or GPU setup the engine cannot run. See [Hardware errors](#hardware-errors). |
| `event_text_not_owned`, `event_text_missing` | An event's text is not a key of the pack's own `en` text. See [Event packs](#event-packs). |
| `service_name_taken`, `service_spread_unsupported`, `unknown_service` | A service rule failed. See [Service packs](#service-packs). |
| `content_outside_declared_category`, `content_category_unsupported` | A file is in the wrong place, or in a category this build does not load. |
| `locale_syntax`, `locale_key_not_namespaced`, `locale_key_missing_en` | A locale rule failed. See [Locale packs](#locale-packs). |
| `model_orphan_item`, `model_missing_file`, `model_malformed`, `model_out_of_bounds`, `model_over_limit`, `model_external_uri`, `model_unsupported_feature`, `model_material_forbidden`, `model_bad_image`, `model_outside_envelope` | A model's GLB broke a rule. See [Limits and rules](MODELS.md#limits-and-rules). |
| `pips_malformed`, `pips_over_limit`, `pips_port_mismatch`, `pips_iec_mismatch`, `pips_bay_mismatch`, `pips_named_port_missing`, `pips_bad_xform` | A model's `.pips.json` is malformed, too large, or does not match the row's shape. See [The .pips.json sidecar](MODELS.md#the-pipsjson-sidecar). |
| `client_pack_has_script` | A pack with `art` or `locale` content also carries a script. |

An incompatible pack is well formed but cannot load here. The game marks it when its id is installed twice, its `game_version` range excludes this build, its `api_minor` is out of range, a dependency is missing or at the wrong version, or it sits on a cycle. The rule code is `incompatible`. The related selection errors are `mod_unavailable`, when a selected id is not installed or is invalid, and `runtime_unavailable`, when this build has no runtime for a script mod.

## The example packs

`ModSDK/examples/com.uptime.example-hardware` carries two new host SKUs, one new switch SKU and one override. `ModSDK/examples/com.uptime.example-scenario` carries one scenario and depends on the hardware pack. `ModSDK/examples/com.uptime.example-locale` carries English and German text. `ModSDK/examples/com.uptime.example-customers` carries two new archetypes and one override. `ModSDK/examples/com.uptime.example-service` carries a locked `Cold Storage` offering over object storage and an archetype that buys it. `ModSDK/examples/com.uptime.example-events` carries two macro events and their English text. `ModSDK/examples/com.uptime.example-model` carries one switch SKU with its own model, the three files `uptime-mod model` writes. See [Equipment models](MODELS.md). Copy any of them as a starting point. For a host or switch with its own shape, start from the [worked example](#worked-example). The model files in that pack are game art and are not MIT: see `TERMS.md` in the SDK folder.
