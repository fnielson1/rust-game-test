## Why

Level geometry is limited to straight line segments, so every surface the player rolls across is a
flat plane or a chain of visible facets. A ball-based platformer wants hills to roll up and vales to
gather speed in, and today the only way to approximate one is to hand-author a long chain of
segments whose joints are still visibly angular and tedious to keep smooth while editing.

## What Changes

- A level segment may carry an optional `control` point, turning it from a straight line into a
  quadratic Bézier curve from `start` to `end` bent toward `control`. Segments without `control`
  behave exactly as they do today, so existing level files are unaffected.
- A curved segment is drawn as one smooth ribbon of the segment's `thickness`, and collides as one
  entity — the player rolls across a hill or vale continuously rather than bumping over facets.
- The curve is approximated internally by short straight pieces; how finely is chosen automatically
  from the curve's size, with an optional per-segment override for authors who want a deliberately
  faceted surface or a cheaper one.
- Curves with a degenerate or non-finite `control` point are rejected the same way degenerate
  straight segments already are: the segment is skipped with a diagnostic and the rest of the level
  still loads.
- Curved segments participate in live editing and hot patching identically to straight ones.

## Capabilities

### New Capabilities

_None._ Curves extend the existing level format and loading capabilities rather than introducing a
separate concern; splitting "curved geometry" into its own spec would leave two specs describing one
file format.

### Modified Capabilities

- `level-format`: the segment schema gains an optional `control` point and an optional subdivision
  override; validation rules extend to cover them; the existing "angle is derived from endpoints"
  requirement is restated so it applies to straight segments while curved ones derive their shape
  from all three points.
- `level-loading`: spawning a segment must handle a curved segment, which becomes one collidable
  entity approximating the curve rather than a single rectangle.

## Impact

- `src/levels/level_asset.rs` — `Segment` gains optional fields; `validate` covers them; the
  straight-line `transform` helper is joined by curve sampling that yields the pieces a curved
  segment is built from.
- `src/levels/spawn_level.rs` — the spawn loop branches on whether a segment is curved, building a
  ribbon mesh and a multi-piece collider for curved ones.
- `src/create_static_solid.rs` — unchanged; it already accepts an arbitrary `Mesh` and an arbitrary
  `Collider`, which is exactly what a curved segment needs.
- `assets/levels/level1.level.json` — gains at least one hill and one vale so the feature is
  exercised by the running game.
- No new crate dependencies: `avian2d` already provides compound colliders, and Bevy already
  provides direct mesh construction.
- Physics behaviour: the player's grounding check reads contact normals and needs no change, but a
  curve's facet joints are the place a rolling ball could snag, so smoothness is a correctness
  concern and not only a visual one.
