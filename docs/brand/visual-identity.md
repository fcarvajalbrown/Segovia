# Segovia — Visual Identity Rule Book

A reproducible spec for the Segovia logo/cover aesthetic. Any agent should be able to read this and
generate an on-brand image. The look: **dark, scientific, calm — a glowing stream flowing across the
segmented arches of an aqueduct.** Minimal, no clutter, no stock-photo gloss.

---

## 1. Concept (always honor this)

The single motif is a **segmented conduit carrying a luminous stream** — read at once as the arches
of the **Aqueduct of Segovia** and as a chunked data stream: thick warm spans separated by small
gaps, with one **bright pulse travelling across a span**. A continuous signal carried over discrete
segments is the metaphor for chunked, span-by-span streaming. The name honors **Claudio Segovia**, a
friend who died of leukemia at 26. Every Segovia visual should contain this motif. Do not replace it
with brains, neurons-with-dendrites, circuit boards, or generic "AI" imagery.

## 2. Color palette

| Role | Hex | Notes |
|---|---|---|
| Background top | `#0B1020` | Deep navy. |
| Background bottom | `#161B2E` | Slightly lighter navy — vertical gradient top→bottom. |
| Span gradient left | `#DEA584` | Warm sand (the lighter Rust tone). |
| Span gradient right | `#CE422B` | Rust orange-red (Rust's brand color). Conduit runs L→R light→dark. |
| Gap markers | `#5A6B8C` | Muted slate-blue dots in the gaps between spans. |
| Pulse glow | `#FFD9A0` | Warm halo, fades into the background. |
| Pulse core | `#FFE7C2` | Near-white hot center. |
| Text — primary | `#F5F7FA` | Title. |
| Text — accent | `#DEA584` | Tagline (matches span-left). |
| Text — secondary | `#8B97B0` | Sub-tagline. |
| Text — dim caps | `#5A6B8C` | Small tracked metadata line. |

Rule: warm (rust/amber) only on the signal and the title accent; everything structural is cool
navy/slate. The warmth = the living signal; the cool = the substrate.

## 3. Typography

- Family: **Segoe UI** (system; falls back to Arial/Helvetica). No serif, no display fonts.
- Title "Segovia": **bold**, very large (≈120px on a 630px-tall canvas), letter-spacing ~2.
- Tagline: regular, accent color.
- Sub-tagline: regular, secondary color.
- Metadata line: small, ALL CAPS, wide letter-spacing, dim color, separated by `·`.
- Left-align everything to a single left margin. No centered text.

## 4. Composition

- **Conduit is horizontal, vertically centered**, spanning most of the width with a small margin.
- **Title sits above** the conduit; **taglines below**, all sharing the same left margin (~10% of width).
- **One** pulse only, positioned at a gap roughly in the left-center third (not dead center).
- 4–5 spans across the canvas. Spans use rounded ends (caps). Gaps ≈ 2× the gap-dot diameter.
- Generous negative space. The image should feel quiet.

## 5. The glow

Render the pulse as concentric circles fading from `#FFD9A0` at center to the background color at
the rim (≈60px radius on the cover), then a small solid `#FFE7C2` core (≈13px). No hard ring.

## 6. Do / Don't

- ✅ Keep it flat and minimal; ✅ one signal pulse; ✅ rust gradient strictly left→right.
- ✅ Honest, technical tone; ✅ readable at small sizes (test the thumbnail).
- ❌ No drop shadows on text, no 3D, no gradients on text, no emojis.
- ❌ No more than the palette above; ❌ no photos; ❌ no neuron/brain clip-art.

## 7. Output specs

| Use | Size | Ratio |
|---|---|---|
| LinkedIn / blog article cover | 1200 × 630 | 1.91:1 |
| Feed / square post | 1080 × 1080 | 1:1 |
| GitHub social preview | 1280 × 640 | 2:1 |

Scale all coordinates proportionally between sizes; keep the same margins and motif placement.

## 8. How to generate

**Canonical: hand-author an SVG** (see `assets/segovia-cover.svg`, `assets/segovia-feed.svg`). SVG
keeps the file tiny, editable, and resolution-independent.

**Rasterize to PNG with Pillow** (this machine has no cairo, so `cairosvg`/`svglib`/`rlPyCairo` all
fail with `no library called "cairo-2"`). Use the Pillow renderer at `assets/logo_render.py`, which
reproduces the spec directly (vertical bg gradient, per-x rust gradient along the conduit, gap dots,
concentric-circle glow, Segoe UI text). Run: `python assets/logo_render.py`. To add a new size,
copy a `render(...)` call and scale the coordinates.

(If a future machine has cairo/Inkscape/ImageMagick, converting the SVG directly is also fine and
gives identical results.)

## 9. Copy-paste prompt for a future session

> Generate a Segovia brand image at <SIZE>. Follow `docs/brand/visual-identity.md` exactly:
> dark navy vertical-gradient background (`#0B1020`→`#161B2E`); a horizontal segmented conduit
> (aqueduct-like), vertically centered, drawn as 4–5 rounded warm spans with a left→right gradient
> (`#DEA584`→`#CE422B`) separated by small gaps marked with `#5A6B8C` dots; one
> glowing signal pulse travelling across a left-of-center gap (warm halo `#FFD9A0` fading to background,
> hot core `#FFE7C2`). Left-aligned text: "Segovia" in large bold Segoe UI (`#F5F7FA`) above the
> conduit; an accent tagline (`#DEA584`) and a secondary sub-tagline (`#8B97B0`) below; a small dim
> ALL-CAPS tracked metadata line (`#5A6B8C`). Minimal, flat, quiet, no shadows/photos/clip-art.
> Author it as SVG, then rasterize to PNG with Pillow (no cairo on this machine) via a small
> renderer like `assets/logo_render.py`.
