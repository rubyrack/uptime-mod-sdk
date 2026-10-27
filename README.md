# Uptime mod SDK

This folder holds everything you need to make mods for Uptime. Start with the [quickstart](docs/QUICKSTART.md), which takes you from nothing to a packaged mod. To load a pack in the game, see [where the mods folder is](docs/CONTENT_PACKS.md#where-the-mods-folder-is).

| Path | Contents |
|---|---|
| `bin/` | The `uptime-mod` command, which makes, checks, runs, packages and uploads a mod. It ships in the game's `ModSDK` folder. The public mirror has the source and docs only. See [the uptime-mod command](docs/CLI.md). |
| `docs/` | The guides. [Start at the index](docs/README.md). To give new hardware its own 3D model, read [Equipment models](docs/MODELS.md). |
| `examples/` | Working mods: content packs, one with its own 3D model (`com.uptime.example-model`), two Lua script mods (one of them shows the HUD card, toasts and keys), and a Rust script mod with its source. |
| `templates/` | Reference copies of what `uptime-mod new` writes. Editing them changes nothing, because the command carries its own templates. |
| `rust/uptime-mod` | The Rust SDK crate for script mods. See [Rust mods](docs/RUST.md). |
| `lua/types`, `lua/.luarc.json` | LuaLS type stubs and the editor config for Lua mods. See [Lua mods](docs/LUA.md). |
| `wit/uptime-mod.wit` | The `uptime-mod` world that a mod component exports. [Other languages](docs/OTHER_LANGUAGES.md) explains how to target it. |
| `schemas/` | The generated schemas and field ledgers for the mod API. See [Reference](docs/REFERENCE.md). |
| `NOTICES`, `THIRD_PARTY.md` | Third-party licence texts. |
