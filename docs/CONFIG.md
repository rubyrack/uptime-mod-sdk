# Config

A script mod can offer settings. You describe them in a config schema file, and the game checks the player's values against it, fills in defaults and hands your mod the result in `init`. A mod with no schema takes no config.

## Declare a schema

Name the file in the manifest:

```json
"config_schema": "config.json"
```

The file is a small subset of JSON Schema, at most 64 KiB:

```json
{
  "type": "object",
  "title": "Autoscaler",
  "additionalProperties": false,
  "properties": {
    "reserve_percent": {
      "type": "integer",
      "title": "Cash reserve",
      "minimum": 0,
      "maximum": 50,
      "multipleOf": 5,
      "default": 10
    },
    "greedy": { "type": "boolean", "title": "Buy early", "default": false },
    "growth_rate": {
      "type": "number",
      "title": "Growth rate",
      "minimum": 0.5,
      "maximum": 4,
      "multipleOf": 0.5,
      "default": 1
    },
    "policy": {
      "type": "string",
      "title": "Policy",
      "enum": ["steady", "fast"],
      "default": "steady"
    }
  }
}
```

The root is an object. `additionalProperties` is required and must be `false`, because a config takes only the keys it declares. `title` and `description` are optional on the root and on each property.

A property has a `type` and a `default`. Every property needs a `default`. The type decides the other keywords:

| Type | Required | Optional | The game draws it as |
|---|---|---|---|
| `boolean` | `default` | none | A toggle. |
| `integer` | `minimum`, `maximum`, `default` | `multipleOf`, 1 when absent | A slider. |
| `number` | `minimum`, `maximum`, `default` | `multipleOf` | A slider. |
| `string` | `enum`, `default` | none | A select. |

These rules apply:

- A property key matches `[a-z0-9_]{1,32}`, and the schema has at most 64 properties.
- Integer bounds and steps are whole numbers no larger than 2^53. The default is inside the bounds and a multiple of the step.
- A string lists its values in `enum`, 1 to 32 values. Each value is an id (`[a-z0-9_]{1,32}`), with no repeats, and the default is one of them. There is no free-text field.
- A `title` is at most 64 bytes with no control, bidirectional or zero-width characters. Without a `title`, the key is the label. A `title` may instead be `@` and a locale key, such as `"@mod-com-example-autoscaler-reserve"`, which shows the pack's text in the player's language; `@@` starts a title that really begins with an at sign. The key rules are the ones for panel labels in [Panels](PANELS.md#labels-that-name-a-locale-key): the key is one of the pack's own messages or a message of a locale pack it lists in `dependencies`, and that pack's `en` files define it, or the pack fails with `locale_key_not_namespaced` or `locale_key_missing_en`. A script mod keeps its words in a second, locale pack and depends on it; the two-pack example is in that section.
- Any other keyword is an error that names it. So is a keyword that does not fit the type, such as `minimum` on a string.

The ledger `content.v1alpha1.fields` lists the accepted keywords and ranges, and `content.v1alpha1.schema.json` holds the same rules for your editor. See [Reference](REFERENCE.md).

## What the player's values must satisfy

The game builds a settings form from the schema in the mod's entry on the Mods screen. It has a toggle for each boolean, a slider for each integer or number, and a select for each string. A select shows the `enum` values as its labels.

Each change is saved at once to `mods/config/<mod id>.json` in the game's user data folder, as a JSON object that maps a key to a value. The saved values reach the game with the next mod selection, so they apply the next time a run starts or loads. If a mod update removes a key from the schema, the game drops that key from the saved values and still loads the run.

The game checks the values before the mod runs:

- Text over 64 KiB, a value that is not a JSON object and a key the schema does not declare are refused.
- A missing key takes its default.
- A value must have the right type, sit inside its bounds and be a multiple of its step.
- A string value must be one of the `enum` values.

A refused config is an error that names the mod, the schema file and the key. A session runs on one config. A load picks up the current values, and if they differ from the ones the save ran with, the load shows a `config_changed` warning.

## What your mod receives

The game encodes the checked values as named msgpack with the keys in sorted order. Each integer property is an integer, and each number property is always a float. The same values give the same bytes, whatever order the player's JSON listed them in.

In Lua the table is `ctx.config`:

```lua
function M.init(ctx)
  local reserve = ctx.config.reserve_percent
  local greedy = ctx.config.greedy
end
```

In Rust, `Init::config` holds the bytes. Decode them with `rmp_serde::from_slice` into a struct whose fields match your keys. The example autoscaler treats an empty config as the defaults.

A mod with no `config_schema` gets an empty config, and the game refuses any config sent for it.

## Config for a dedicated server

A player sets config through the game's settings form. A dedicated server has no form, so the operator names a folder with `--mod-config`:

```
sim-runner --server --scenario garage-to-glory \
  --mods-dir /srv/uptime/mods --enable-mods com.example.my-mod \
  --mod-config /srv/uptime/mod-config
```

The folder holds one file for each configured mod, named `<mod id>.json`, such as `com.example.my-mod.json`, with the object described above:

```json
{ "reserve_percent": 20, "policy": "fast" }
```

The runner reads these files at boot:

- `--mod-config` needs `--mods-dir` and `--enable-mods`.
- A mod with no file runs on its defaults.
- A file that fails the checks above stops the boot and the message names the file.
- A `.json` file that names a mod the operator did not enable is ignored, and the runner logs one line.
- No connection can change the config while the server runs.
