# Sharing a mod on the Workshop

The Steam Workshop is where players find your mod. One pack is one Workshop item. This page covers uploading, who can see the item, updating it, and what a player sees after subscribing. The Workshop is not part of the demo.

## Upload

`uptime-mod upload` packages your pack the way [`package`](CLI.md#package) does, checks the result again, and publishes it.

```
uptime-mod upload <dir> [--visibility private|friends|public]
                  [--change-note <text>] [--preview <image>] [--dry-run]
```

- The upload needs Steam running and signed in to an account that owns Uptime. The `uptime-mod` in the game's `ModSDK/bin` folder can reach Steam. A copy built without Steam stops with a message and still runs `--dry-run`.
- `--dry-run` prints the plan and sends nothing. It also writes nothing into your folder. Run it first. The [quickstart](QUICKSTART.md) does.
- `--change-note` is the text Steam shows for this version. It may hold line breaks and is at most 8000 bytes.
- `--preview` is the item's picture. It must be a file of at most 1 MiB.
- The exit code is 0 for a finished upload or dry run, 1 when the pack or Steam refuses it, and 2 when the command line is wrong. Nothing is sent when the pack has problems.

Steam sets the item's title and description on the first upload only. The title is your pack id and the description is the id and version. Edit them on the item's Workshop page. Later uploads leave them alone.

The item carries tags that come from your manifest: its kinds (for example `content` or `script`), its content categories, and `lua` or `rust` for a script mod. It also carries the key-value tags `mod_id` and `version`, so an item can be found by its pack id.

## Visibility

A new item is private. Only you can see it. Pass `--visibility friends` or `--visibility public` to change that. An upload without `--visibility` keeps the item's current setting, so a public item stays public when you push an update.

## The legal agreement

Steam keeps a new item hidden from everyone but you until you accept the Workshop legal agreement. When Steam says you have not, the upload still finishes and prints a warning with the link. Accept the agreement once for your account, and your items become visible at the setting you chose.

## Updates

The first upload makes the item and writes its id to `workshop.json` in your pack folder:

```json
{
  "item_id": 3456789012,
  "published": true
}
```

Every later upload from that folder updates the same item. The id stays stable as long as `workshop.json` stays. Keep the file with your source and commit it. Do not edit it, and do not copy it to another folder you upload from: two folders with the same id update one item. The command refuses a file that is not a nonzero `item_id`. If you delete the file, the next upload makes a new item and the old one stays on the Workshop.

`workshop.json` never goes into the package. The upload does record the item id in the package's `uptime-package.json`. That is how a player who is refused at a multiplayer join gets a Subscribe button for your pack. See [Multiplayer](MULTIPLAYER.md).

Raise `version` in your manifest for each release. Players and the game tell two copies of a pack apart by its version and its hash.

## What a player sees

A player subscribes on the item's Workshop page. Steam downloads the item. The game then lists it in the Mods panel with a Workshop badge, next to the packs from the local mods folder. A pack from the Workshop goes through the same checks as a local pack, so a pack that `validate` accepts loads, and one it refuses does not.

A new item is listed **off**. Subscribing never turns a pack on. The player turns it on in the Mods panel, which is also where a pack that changes gameplay shows as one.

The Status column of the panel tells the player where the item stands:

| Status | Meaning |
|---|---|
| OK | The pack is installed, current and usable. |
| Needs update | Steam has a newer version it has not fetched yet. The old copy stays usable until Steam swaps it. |
| Downloading | Steam is fetching the item. |
| Not downloaded | The player is subscribed and Steam has not started the download. |
| No pack found | The item is installed, but its folder holds no pack the game can read. |
| Shadowed | The player has a local pack with the same id. The local copy is used while it can load, and the Workshop copy cannot be turned on. When the local copy cannot load and the Workshop copy can, the Workshop copy is used instead, and the local row shows Shadowed with its own problems. |
| Can't load | The pack is well formed but this game version cannot load it. |
| Invalid | The pack has problems. Select the row to read them. |

Selecting a Workshop row shows an **Open in Workshop** button that opens the item's page.

No pack found usually means the item was uploaded from a folder that was not a packaged pack. Upload with `uptime-mod upload`, which packages for you.

## A dedicated server

A dedicated server needs no Workshop code. The operator downloads the item with `steamcmd` and points the runner at the folder it lands in. The `steamcmd` steps below are the operator's procedure with Valve's tool. The game does not run or test them. Only the `sim-runner` flags are the game's. Steam keeps the app's Workshop items in `steamapps/workshop/content/4813880/<item id>/`, and each of those folders is one pack, which is the layout `--mods-dir` reads.

An anonymous download works only if the Uptime app allows anonymous Workshop downloads, which is a Steamworks partner setting the publisher controls. Otherwise log in with an account that owns Uptime: replace `+login anonymous` with `+login <account>`.

```
steamcmd +login anonymous +workshop_download_item 4813880 3456789012 +quit
sim-runner --server --scenario garage-to-glory \
  --mods-dir <steamcmd folder>/steamapps/workshop/content/4813880 \
  --enable-mods com.example.my-mod
```

- `--enable-mods` takes pack ids, not item ids.
- The runner needs both flags together, and a world to build (`--scenario` or `--world`). It stops with a message when a pack is invalid.
- The server does not update items itself. The operator runs `steamcmd` again to fetch a new version, then restarts the server.
- A server that runs script mods can set their options with `--mod-config`. See [Config](CONFIG.md#config-for-a-dedicated-server).
