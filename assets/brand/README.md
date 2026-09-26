# AetherCodex brand assets

Everything visual in this repository derives from two files in this directory.
Replace them and rerun the generator; nothing else hardcodes an icon.

```
assets/brand/aethercodex-mark.svg   master (vector, preferred)
assets/brand/aethercodex-mark.png   master (raster, >= 1024x1024, fallback)
```

```sh
bash scripts/brand/generate-icons.sh          # regenerate every platform icon
bash scripts/brand/generate-icons.sh --check  # fail if icons drift from the master
```

The generator writes the Windows `.ico`, the macOS `.icns`, the Tauri app icon
and the Linux hicolor PNG. `--check` runs in CI, so an icon can never silently
fall out of sync with the master.

## Status of the current master

> **The mark in this directory is a WORKING PLACEHOLDER, not the approved
> Archai master.**

The Archai brand asset registry (`ARCHAI-CIVI-001`) lists the approved lockups
at `docs/brand/archai/datasheet/assets/archai_logo_{dark,light}_text.png` inside
the AetherWorks workspace, and lists *"approved Archai vector master"* under
`pending_assets`. Neither was reachable when this placeholder was drawn, so it
was constructed from the published colour and geometry tokens alone:

- square geometry, no gradient, no shadow;
- Archai Orange `#ED9527` as the brand anchor;
- Aether Blue `#2F6FED` as the connection/interface mark;
- Engineering Graphite `#0B0D10` ground, Hairline `#DDE3EC` inset rule.

It is not approved for print, packaging, regulatory artwork or any external
release. Swap in the approved master before any of those.

## Colour semantics

Defined once in `apps/aethercodex-manager/src/styles.css` as CSS custom
properties, and mirrored in the installer artwork. They are not
interchangeable:

| Token | sRGB | Meaning |
| --- | --- | --- |
| Archai Orange | `#ED9527` | brand origin, section anchor. Never small body copy on a light surface (2.35:1 on white). |
| Aether Blue | `#2F6FED` | connection, interface, link, editable working field. |
| Evidence Green | `#2F855A` | verified / released / closed status backed by evidence **only**. |
| Engineering Graphite | `#0B0D10` | primary typography and technical authority. |
| Secondary Ink | `#626A78` | metadata and secondary copy. |
| Soft Surface | `#F7F9FC` | non-semantic surface layering. |
| Hairline | `#DDE3EC` | rules and table boundaries. |

Blue and green are lightened in dark mode and darkened in light mode so both
clear WCAG AA body contrast (4.5:1) against the surface they sit on; the
derived values are in `styles.css` next to the masters.

Never use green because a screen needs more colour, and never put orange text
on a light surface.
