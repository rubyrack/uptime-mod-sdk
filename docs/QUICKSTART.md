# Quickstart

This page takes you from nothing to a packaged mod that is ready to upload. You make a Lua script mod, check it, run it in a scenario, package it and preview its Workshop upload. Each command block marked `<!-- run -->` runs in a test, so the steps here work as written.

You need the game's `ModSDK` folder. Its `bin` folder holds `uptime-mod`. Run the commands below from a folder you can write to. [The uptime-mod command](CLI.md) explains every flag.

## 1. Make a mod

<!-- run -->
```sh
uptime-mod new --template lua --id com.example.first-mod first-mod
```

This makes the folder `first-mod` with a manifest and two Lua files. It is a working autoscaler that accepts prospects and buys a host when capacity runs low.

## 2. Validate it

<!-- run -->
```sh
uptime-mod validate first-mod
```

The command checks the pack the way the game does. A clean pack prints `ok:` and a hash. Any problem prints the file, the field and the rule, and the exit code is 1.

## 3. Run it

<!-- run -->
```sh
uptime-mod run first-mod --scenario garage-to-glory --quanta 200
```

The command plays your mod in a scenario for 200 quanta, which is 50 minutes of sim time. It prints one JSON line for each command your mod landed. If it cannot find the game's runner, pass `--runner` or set `UPTIME_RUNNER`.

## 4. Package it

<!-- run -->
```sh
uptime-mod package first-mod --out dist
```

The command writes `dist/com.example.first-mod-1.0.0/`, the folder you share. It also prints the pack hash.

To try the pack in the game, copy `dist/<id>-<version>/` into your mods folder. For this mod, that is `dist/com.example.first-mod-1.0.0/`. [Where the mods folder is](CONTENT_PACKS.md#where-the-mods-folder-is) gives the path on each system.

## 5. Preview the upload

<!-- run -->
```sh
uptime-mod upload first-mod --dry-run
```

The command packages the mod again, then prints what it would send to the Steam Workshop: the pack id and version, the hash, the title, the tags and the visibility. A first upload is private. It ends with `dry run: nothing was sent to the Workshop`, and it writes nothing into your folder. Drop `--dry-run` to upload for real, with Steam running and signed in to an account that owns Uptime. [Workshop](WORKSHOP.md) covers visibility, updates and what players see.

## What next

- [Lua mods](LUA.md) and [Rust mods](RUST.md) cover each language.
- [Script mods](SCRIPTS.md) covers what every script mod shares.
- [Panels](PANELS.md) and [Config](CONFIG.md) add screens and settings.
- [Workshop](WORKSHOP.md) covers sharing a mod, and [Multiplayer](MULTIPLAYER.md) covers what other players need to join a world that uses it.
