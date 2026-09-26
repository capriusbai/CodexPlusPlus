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

## Provenance of the current master

The mark is the Aether logo supplied by the project owner. The source artwork
was a 248x249 bilevel PNG (black on transparent); it was vector-traced with
`potrace` so it scales cleanly, then placed on an Engineering Graphite ground
at 83% of the tile. Only the ground and the mark colour were chosen here — the
geometry is the supplied logo, unaltered.

```
aethercodex-mark.svg  sha256 251388ea18ece6f3fd12c1f99a3eba2bd9bf1f595250c420f755a98d115a1ca7
```

Notes and limits:

- The mark holds together down to 32px. At 24px and 16px the eye and the wave
  begin to merge, which is inherent to line art at that size; those slots are
  only used for the Windows small icon and the favicon. A simplified 16px
  variant would be the fix if that ever matters.
- The light-on-graphite treatment is set in two places in the SVG: the `rect`
  fill (`#0B0D10`) and the mark group's `fill` (`#F7F9FC`). Swap them for a
  graphite-on-light tile.
- This is a working product treatment, not an approved print or packaging
  master. The Archai registry still lists the approved vector master as
  pending, so CMYK, Pantone, clear-space and minimum-size rules are not
  settled here. Confirm those before print, packaging or regulatory artwork.

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
