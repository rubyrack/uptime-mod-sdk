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
  "api_minor": 0,
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

The game accepts an `api_minor` between two limits. The low limit is the latest minor that declared a break on a part of the API your pack uses. The high limit is the minor this build serves. A pack outside that range is incompatible and the game says so by name. A pack built against an older minor still loads while no break has landed since.

The current minor is `0`, and no minor has shipped yet. While `v1alpha1` is a preview, a minor may contain a break. The ledgers in [Reference](REFERENCE.md) record every change.

## Hardware packs

Declare `"content": ["hardware"]`. The game loads two files and no others from `hardware/`:

- `hardware/hosts.json` holds `{ "hosts": [ ... ] }`.
- `hardware/switches.json` holds `{ "switches": [ ... ] }`.

The game refuses any other file under `hardware/` with the rule `content_category_unsupported`.

### The base-model rule

A row names a built-in SKU in `model` and sets scalar stats only. It cannot set shape. Rack units, ports, NIC and PSU counts, bays, architecture, accelerator, category and switch role all come from the base model. Every row rejects unknown keys. The game ships no way to import new geometry, so a pack SKU always draws as its base model.

Copy base SKU names exactly from the built-in catalog, for example `Balanced x86 1U`, `Compute 1U` and `16-port Access GbE`. The GPU host is `GPU 2× 2U` and the name contains the character `×`.

### Host rows

| Field | Notes |
|---|---|
| `id` | Required. The item name in the pack. |
| `model` | Required. The base host SKU the row starts from. |
| `overrides` | Optional. `base:<name>` to replace a built-in SKU. |
| `vcpu`, `mem_mb`, `ssd_gb`, `ssd_iops`, `power_w` | Optional numbers. |
| `capex`, `hourly_cost` | Optional money, see below. |
| `mtbf_hours`, `generation_year` | Optional numbers. |
| `accel_count`, `gpu_mem_mb_per_gpu` | Optional. Above zero only when the base model has an accelerator. |

### Switch rows

| Field | Notes |
|---|---|
| `id` | Required. The item name in the pack. |
| `model` | Required. The base switch SKU the row starts from. |
| `overrides` | Optional. `base:<name>` to replace a built-in SKU. |
| `capex`, `hourly_cost` | Optional money, see below. |
| `power_w`, `fabric_capacity_gbps`, `pps_ceiling`, `mtbf_hours` | Optional numbers. |

Each number has a floor and a ceiling. The exact bounds are in `content.v1alpha1.schema.json`, described in [Reference](REFERENCE.md). A value outside them fails with the rule `min` or `max` and names the field.

Money fields are whole numbers in minor units. One dollar is 100,000,000 units. The example hardware pack sets `capex` to `220000000000`, which is 2,200 dollars.

### Ids and overrides

An item `id` matches `[a-z0-9][a-z0-9_-]{0,63}`. A new row becomes the SKU named `<pack id>:<id>`. Scenarios and `sku_gates` use that full name.

A row with `"overrides": "base:<name>"` replaces that built-in SKU's stats in place and keeps its name. Three rules apply:

- The text after `base:` must name a built-in SKU of the same family.
- The row's `model` must equal that same name.
- One pack may override a target once.

Only built-in SKUs can be overridden. If two selected packs override the same built-in SKU, the one that loads last wins. Load order is the dependency order, and ties go to the smaller pack id.

## Scenario packs

Declare `"content": ["scenarios"]`. A scenario is a RON file at `scenarios/<item>.ron`. The file name stem must match the item id pattern above. The game refuses files in subfolders of `scenarios/`, and any file that is not `.ron` or not UTF-8 text.

The scenario's own `id` must equal `<pack id>:<file stem>`. The rule `scenario_id_not_namespaced` fires otherwise.

A scenario `config` block may set only these keys: `tuning`, `sku_gates`, `pricing`, `prospects`, `staff`, `bootstrap`, `component_lifecycle`, `relationship`, `consequences`, `service_investment`, `service_plane`, `rewards`, `quests`, `coop`, `bailout` and `depreciation`. The same holds for the `config` inside `hard_mode`. A pack scenario may not set `catalogs`, `steam` or `sites`. New SKUs come from hardware packs, and the achievement catalog and the sites belong to the base game. Every SKU a scenario names must exist in the packs you selected or in the base game. Every `archetype` a seeded customer or prospect carries must be an archetype id or name from the base game, the scenario's own archetypes or a selected customer pack, or it fails with `unknown_archetype`.

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

The manifest accepts `art`. This build loads one kind of art file: a pack icon, a PNG at `art/icons/<item>.png` where `<item>` is the id of one of the pack's own hardware items. An icon is at most 64 KiB and at most 256 by 256 pixels. The shop shows it on that item's row. A pack with `art` content is client-required: a player who joins a multiplayer world that uses it must have it installed. See [Multiplayer](MULTIPLAYER.md).

Every other file under `art/` fails with the rule `content_category_unsupported`.

## Dependencies

`dependencies` maps a pack id to a semver range:

```json
"dependencies": {
  "com.uptime.example-hardware": ">=1.0.0 <2.0.0"
}
```

The game refuses a pack that depends on itself. A dependency must be installed, must match the range and must be able to load. The player must also select it. The game loads dependencies first. A dependency cycle makes every pack on the cycle incompatible. If a dependency cannot load, the packs that need it cannot load either, and the message carries the root cause.

## Archive rules

These rules apply to every file in a pack. The scan checks the pack folder against them before it reads any content.

- `manifest.json` must be at the root.
- Every file the manifest points at must exist: `script.entry`, each `panels` entry and `config_schema`.
- A path uses only `A-Z`, `a-z`, `0-9`, `.`, `_`, `-` and `/`.
- A path is at most 240 bytes. It may not be empty, absolute or contain a backslash or a percent sign. It may not have a segment that is empty, is `.`, ends in a dot, or is made only of dots.
- The part of a segment before its first dot may not be a Windows device name: `con`, `prn`, `aux`, `nul`, `com1` to `com9` or `lpt1` to `lpt9`. So the game refuses `con.txt` too.
- A pack holds at most 4096 entries.
- One file may not exceed 32 MiB, and all files together may not exceed 64 MiB. The game counts the bytes it reads, not the size a header claims.
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
| `archetype_name_taken`, `override_field_forbidden` | A customer rule failed. See [Customer packs](#customer-packs). |
| `event_text_not_owned`, `event_text_missing` | An event's text is not a key of the pack's own `en` text. See [Event packs](#event-packs). |
| `service_name_taken`, `service_spread_unsupported`, `unknown_service` | A service rule failed. See [Service packs](#service-packs). |
| `content_outside_declared_category`, `content_category_unsupported` | A file is in the wrong place, or in a category this build does not load. |
| `locale_syntax`, `locale_key_not_namespaced`, `locale_key_missing_en` | A locale rule failed. See [Locale packs](#locale-packs). |

An incompatible pack is well formed but cannot load here. The game marks it when its id is installed twice, its `game_version` range excludes this build, its `api_minor` is out of range, a dependency is missing or at the wrong version, or it sits on a cycle. The rule code is `incompatible`. The related selection errors are `mod_unavailable`, when a selected id is not installed or is invalid, and `runtime_unavailable`, when this build has no runtime for a script mod.

## The example packs

`ModSDK/examples/com.uptime.example-hardware` carries two new host SKUs, one new switch SKU and one override. `ModSDK/examples/com.uptime.example-scenario` carries one scenario and depends on the hardware pack. `ModSDK/examples/com.uptime.example-locale` carries English and German text. `ModSDK/examples/com.uptime.example-customers` carries two new archetypes and one override. `ModSDK/examples/com.uptime.example-service` carries a locked `Cold Storage` offering over object storage and an archetype that buys it. `ModSDK/examples/com.uptime.example-events` carries two macro events and their English text. Copy any of them as a starting point.
