# Equipment models

A content pack can give a new host or switch its own 3D model. The rack draws that model in place of the base model the row starts from. This page covers the files, how to model and name the parts, the limits, the `.pips.json` sidecar field by field, and what happens when a model fails.

A model is part of the `art` category, see [Art](CONTENT_PACKS.md#art). The example pack `com.uptime.example-model` carries one switch SKU with a model. Its manifest, hardware row and the three model files are the smallest complete case. The model files in it are game art and are not MIT: see `TERMS.md` in the SDK folder.

## The three files

A model is three files in `art/models/`, all named after the item:

```
com.example.my-hardware/
  manifest.json
  hardware/hosts.json
  art/models/box_render.glb
  art/models/box_proxy.glb
  art/models/box.pips.json
```

| File | What it holds |
|---|---|
| `<item>_render.glb` | The near model. |
| `<item>_proxy.glb` | The far model, drawn at a distance. |
| `<item>.pips.json` | Where the ports, lights, fans, power inlets, module bays and nameplate sit. |

The manifest lists `"content": ["hardware", "art"]`. `<item>` is the `id` of a new host or switch row in the same pack. A model never changes what the hardware is. Ports, power supplies and bays always come from the base model, so the model must match their counts.

## Making the files

Export one GLB from your modelling tool and run [`uptime-mod model`](CLI.md#model):

```
uptime-mod model work/box.glb --proxy work/box_proxy.glb --pack com.example.my-hardware --item box
```

The command reads your node names into the sidecar, merges the drawn nodes into the near model, copies the far model you made, and runs the game's own pack checks on the result. [Naming the nodes](CLI.md#naming-the-nodes) lists the names it reads. You write the far model yourself, and the command copies it unchanged, so it must pass the same checks. You can also write all three files by hand.

## Authoring conventions

- Units are metres and the up axis is Y. The glTF format already uses both. In Blender, export with Y up.
- The front face of the device is at Z = 0 and the body runs back along negative Z. Port jacks, bezels and the nameplate face positive Z.
- The model sits inside the base model's rack slot. The slot's width is a 19-inch rack face, centred on X = 0. Its height is the base model's rack units at 44.5 mm each, starting at Y = 0. Its depth is the base model's depth, measured from its shipped render model. When the base model has no measured depth, a host uses 0.740 m and a switch uses its chassis depth. The game allows a little more than the slot: 1 mm at each side, above and below, 15 mm in front for handles and bezels, and 25 mm behind for rear plugs.
- The render file has one mesh drawn by one node. Every drawn node of your source is placed in model space and merged into one primitive per material, so the material count is the primitive count. Keep the node and material counts low at the source.
- The game draws every light from the sidecar. A light is a mesh with `LED` in its node name. `uptime-mod model` leaves these meshes out of the render file and writes a sidecar entry for each, sized to the mesh's box. The exception is a `LED_Link` or `LED_Activity` light that has no `Port_` node to bind to, or whose nearest port is a management pin or `Port_0`: the command drops it and prints a warning, see [Naming the nodes](CLI.md#naming-the-nodes). Give no material an emissive color.
- Name the parts the generator reads: `Port_<N>` for cabled ports numbered from 1, `Port_<name>` for named ports, `LED_Link...` and `LED_Activity...` under a port, `Bay_<N>` for module bays, names ending `_IEC` or `AcInlet` for power inlets, `FanRotor...` for fans, and `Nameplate_Plate` for the name placard. The full table is in [Naming the nodes](CLI.md#naming-the-nodes).
- A default export from a modelling tool usually fails the limits below. It tends to give each object its own node and primitive, lists extensions the game does not load, and includes cameras or lights. Turn off cameras and lights in the exporter, apply modifiers, join objects that share a material, and keep textures inside the GLB.

## Limits and rules

The game checks the model in the pack scan, and again in the client when it loads one. These rules apply to both GLB files unless a row says otherwise.

| Rule | Limit |
|---|---|
| File name | `<item>` is the id of a new host or switch row in the same pack. A model for any other id fails with `model_orphan_item`. So does a model for an override row. |
| Files | An item ships all three files or none, or it fails with `model_missing_file`. |
| Container | A GLB 2.0 file with one scene, one mesh and at most one buffer, which is the BIN chunk. Exactly one node draws the mesh. |
| Data inside | Nothing loads by uri. A buffer or image that points at another file fails with `model_external_uri`. |
| Counts | At most 32 nodes, 32 primitives, 32 materials and 4 images. A texture is a PNG of at most 2048 by 2048 pixels. |
| Near model | At most 32 MiB, 524,288 triangles and 524,288 vertices. |
| Far model | At most 512 KiB, 8,000 triangles and 12,000 vertices. |
| Geometry | Triangle lists only. The bounds, read from the vertex data and placed by the node, must fit the slot above, or the file fails with `model_outside_envelope`. |
| Not allowed | Animation, skins, cameras, morph targets and sparse accessors fail with `model_unsupported_feature`. |
| Extensions | The only extensions are `KHR_texture_transform`, `KHR_materials_specular` and `KHR_materials_ior`. A model may require none. |
| Materials | A material may not emit light or blend its alpha. Use `OPAQUE` or `MASK` for alpha. The rule is `model_material_forbidden`. |
| Nulls | A JSON `null` in the glTF fails with `model_malformed`, except under an `extras` key. So does an object written as an array. |

A GLB that is cut short or has bytes after its last chunk fails with `model_malformed`. One that reads past its buffer fails with `model_out_of_bounds`. A PNG with a bad header, checksum or size fails with `model_bad_image`. A file, texture or count over a limit in the table fails with `model_over_limit`. A list longer than its cap reads the same way, for example `/materials the list has more than 32 entries`.

One failing model makes the whole pack invalid. Nothing in the pack loads until every error is fixed, including its hardware rows. `uptime-mod validate` lists the errors.

## The .pips.json sidecar

The sidecar says where each port, light, fan, inlet, bay and the nameplate sit in model space. A file is at most 256 KiB. It holds one JSON object with these keys. Every key except `nameplate` is required, and an unknown key fails.

```json
{
  "pips": [
    { "kind": "port_link", "port": 1, "xform": [0.003, 0, 0, 0, 0.003, 0, 0, 0, 0.003, -0.2, 0.02, 0] },
    { "kind": "psu", "port": 1, "xform": [0.003, 0, 0, 0, 0.003, 0, 0, 0, 0.003, 0.2, 0.03, -0.1] }
  ],
  "ports": { "1": [0.012, 0, 0, 0, 0.012, 0, 0, 0, 0.012, -0.2, 0.02, 0] },
  "named_ports": { "Port_BMC": [0.012, 0, 0, 0, 0.012, 0, 0, 0, 0.012, -0.21, 0.02, 0] },
  "fans": [],
  "iec": [[0.02, 0, 0, 0, 0.02, 0, 0, 0, 0.02, 0.2, 0.02, -0.2]],
  "bays": {},
  "nameplate": [0.1, 0, 0, 0, 0.02, 0, 0, 0, 0.001, 0, 0.01, 0]
}
```

This one is for a base model with one cabled port, one power supply and no module bays. Your counts come from your base model.

| Key | Holds |
|---|---|
| `pips` | A list of at most 256 status lights, see below. |
| `ports` | An object from `"1"` up to the base model's cabled port count to a transform. The keys must be exactly that set, or the file fails with `pips_port_mismatch`. |
| `named_ports` | An object from a name to a transform, for ports outside the numbered set, such as `Port_BMC`. A light bound to one lights from the device port that carries that name, see `port_name` below. The name is `Port_` and then letters, digits or `_`, at most 64 bytes. It may be empty. |
| `fans` | A list of at most 16 transforms, one for each fan blade. It may be empty. |
| `iec` | A list of transforms, one power inlet for each power supply of the base model. Another count fails with `pips_iec_mismatch`. |
| `bays` | An object from `"1"` up to the base model's module bay count to a transform. It is empty, or holds exactly that set, or the file fails with `pips_bay_mismatch`. |
| `nameplate` | One transform for the name placard. Optional. Leave the key out for none. |

A cabled port is a numbered data port of the base model. A host counts the ports of all its port banks. A switch counts its `port_count`.

### Pip fields

Each entry of `pips` is an object with these keys, and an unknown key fails.

| Field | Required | Holds |
|---|---|---|
| `kind` | yes | What the light shows: `port_link`, `port_act`, `activity`, `power` or `psu`. |
| `port` | yes | A whole number, 0 or more. `uptime-mod model` writes the port number for `port_link` and `port_act` lights on a numbered port, the power supply number (1 is the rightmost) for `psu`, and 0 otherwise. |
| `port_name` | no | The named port a `port_link` or `port_act` light belongs to. Same pattern as a `named_ports` key. The game finds the device port whose label is the name after `Port_`, ignoring case, so `Port_BMC` follows a port labelled `BMC`; `Port_MlagKeepalive` follows the keepalive port. A name no device port carries passes every check and stays dark. `Port_PSU`, `Port_Mgmt` and `Port_Out` are refused, because their labels repeat on one device and the game binds a light to the first port with that label, so the name cannot say which port is meant. |
| `role` | no | A status role such as `sys`, `alm` or `fan`: lowercase letters, digits and `_`, at most 64 bytes. On an `activity` light the game paints by it: `alm` shows a fault and is dark otherwise, `fan` never shows a fault, and `sys` or no role follows the device's indicator. Any other text is accepted and paints as no role. |
| `xform` | yes | A transform, see below. |

### Transforms

Every transform is a list of exactly 12 numbers. The first nine are the basis columns x, y and z, three numbers each. The last three are the origin. The basis scales a unit box, so a light or port jack is a small basis. The rules, all `pips_bad_xform`:

- Every number is finite.
- Each basis column is at least 0.000001 long and at most 2.0 long.
- The three columns do not lie in one plane. The volume they span, divided by the product of their lengths, is at least 0.001.
- The origin sits inside the rack slot, with the same slack as the geometry.

The game checks the origin only. It does not check that the whole light or port fits.

### What the sidecar refuses

- A JSON `null` anywhere, including `"nameplate": null`. Leave `nameplate` out instead. This fails as `pips_malformed`.
- An unknown key at any level. Keys that the built-in models carry, such as `has_side_panels` and `rack_anchors`, are unknown here, and pack models do not use them.
- A key given twice in `ports`, `named_ports` or `bays`.
- An array where an object belongs, such as `ports` written as a list.
- A file over 256 KiB, more than 256 pips or more than 16 fans, as `pips_over_limit`.

## When a model fails

The pack scan refuses a bad model and the whole pack with it, see [What refuses a pack](CONTENT_PACKS.md#what-refuses-a-pack). That is the normal failure and `uptime-mod validate` shows it.

The client checks the bytes again when it loads a pack, and then imports them with the engine's glTF importer. A model that fails the check or the import draws as its base model. The game shows a warning toast once for that item, "The model for X in the mod Y couldn't be loaded, so it looks like the model it's based on." Every other item of the pack is unaffected.

## Without a model

A new SKU with no model files draws as its base model. An override of a built-in SKU always draws as that SKU. A pack with `art` content is client-required, so a player who joins a multiplayer world that uses it must have it installed. See [Multiplayer](MULTIPLAYER.md).
