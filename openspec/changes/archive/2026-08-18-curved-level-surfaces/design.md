## Context

Levels are JSON files listing segments — `start`, `end`, `thickness` — and `spawn_level` turns each
one into a single entity: a `Rectangle` mesh and a matching `Collider::rectangle`, rotated to the
angle between the endpoints by `Segment::transform`. The whole pipeline (asset loader → validation →
`create_static_solid` → one `LevelSegment` entity) is built around "a segment is a rotated
rectangle."

Curves break that assumption in exactly one place — the shape of a segment — and nowhere else. The
loader, the despawn/rebuild flow, the hot-patch path, the file watcher, and the grounding check all
stay as they are. The design's job is therefore narrow: define how a curve is authored, how it is
turned into a mesh and a collider, and how it fails safely.

Two existing properties constrain the approach:

- **Rebuilds are frequent.** Saving a level file rebuilds every segment in it, live. Anything done
  per-segment at spawn time runs on every keystroke-to-save cycle, so it has to be cheap — this rules
  out expensive collider construction like convex decomposition.
- **The player is a ball resting on surfaces.** `update_grounded` reads contact normals off a
  shapecast. A surface built from facets that don't join cleanly doesn't just look wrong, it snags a
  rolling ball and makes grounding flicker. Joint quality is a correctness requirement.

## Goals / Non-Goals

**Goals:**

- Author hills and vales in the level file with one added field, keeping every existing level file
  valid and unchanged.
- Produce a surface a ball rolls across smoothly — no visible facets at normal zoom, no snagging at
  joints, no gaps between the drawn surface and the collided surface.
- Keep one entity per segment, so despawn/rebuild, hot patching, and the `LevelSegment` marker's
  meaning are all untouched.
- Fail an unbuildable curve the same way an unbuildable straight segment already fails: skip it, name
  it in a diagnostic, load the rest of the level.

**Non-Goals:**

- Cubic Béziers, splines, or multi-span curves through a shared point list. One control point per
  segment; chain segments for anything more elaborate.
- Curve-aware physics beyond geometry — no changes to friction, restitution, grounding thresholds, or
  the player controller.
- An editor, a preview, or any authoring tool. The file stays hand-edited.
- Closed shapes, filled regions, or curves used as anything other than a surface.

## Decisions

### A curved segment is a quadratic Bézier defined by an optional `control` point

`Segment` gains `control: Option<[f32; 2]>`. Absent means straight — the existing code path, byte-for-
byte identical output, so `level1.level.json` and every other existing file keep working untouched.
Present means the segment is the quadratic Bézier from `start` to `end` bent toward `control`.

Quadratic specifically, rather than cubic or a circular arc:

- A quadratic Bézier is always a simple, convex arc. It cannot self-intersect, cannot cusp, and has
  no inflection point. Every curve an author can write is therefore a well-formed hill or vale, which
  removes a whole class of validation and offsetting problems that cubics would introduce.
- It reads naturally as a hill: put `control` above the line for a rise, below it for a dip, and off-
  center for a lopsided slope. A circular arc (radius + direction) can't do lopsided at all, and makes
  the author solve for a radius that yields the height they wanted.
- Its second derivative is constant, which gives an exact, closed-form subdivision count (below)
  instead of a recursive flattening loop.

Trade-off accepted: the curve passes through `start` and `end` but only *toward* `control`, peaking at
roughly half the control point's offset. Authors adjust by eye, which is the normal Bézier bargain.

### Flattening uses a closed-form subdivision count from a fixed error tolerance

For a quadratic Bézier, `B''(t)` is the constant `2 * (P0 - 2*P1 + P2)`. Splitting the curve into `n`
equal steps in `t` bounds the distance between the true curve and its chords at
`|P0 - 2*P1 + P2| / (4 * n^2)`. Inverting that for a chosen tolerance gives the count directly:

```
n = ceil( sqrt( |P0 - 2*P1 + P2| / (4 * TOLERANCE) ) )
```

with `TOLERANCE` a world-unit constant (start at `0.5`, roughly half a pixel at normal zoom) and `n`
clamped to a sane range (`1..=64`) so a wild control point can't ask for thousands of pieces.

This is preferred over the two usual alternatives. A fixed count (say, always 16) over-tessellates a
gentle 40-unit rise and under-tessellates a 900-unit one — the same file gets inconsistent quality
depending on scale. Recursive adaptive flattening solves that too, but it is a loop with a stack for
a case where the closed form is exact, and it runs on every level rebuild.

An optional `subdivisions: Option<u32>` on the segment overrides the computed count, for deliberately
faceted surfaces or for turning the cost down. It is an override, not a hint: if present it is used
as given (clamped to the same range).

### Mesh and collider are built from the same offset ribbon

Flattening yields `n + 1` points along the curve. At each point, the unit normal to the curve's
tangent gives an outer point at `+thickness/2` and an inner point at `-thickness/2`. Consecutive
pairs form `n` quads that share their edges exactly.

- **Mesh**: one `Mesh` with `PrimitiveTopology::TriangleList`, two triangles per quad. Because
  adjacent quads share vertices, the ribbon is watertight — no seams, no notches.
- **Collider**: `Collider::compound` of one `Collider::convex_hull` per quad, each a genuinely convex
  four-point shape. Convex pieces are what avian's solver handles best, and building the collider from
  *the same four points as the drawn quad* means the collided surface and the drawn surface are the
  same surface. That is the property that keeps a rolling ball from catching: there is no outward V
  notch at a joint for it to hit.

Alternatives rejected:

- *Spawn each chord as its own straight segment entity.* Cheapest to implement — it reuses the
  existing path completely — but it breaks one-entity-per-segment (so the `LevelSegment` and
  `HotReloadable` accounting gets murkier), and centered rectangles leave a small V notch on the
  convex side of every joint, exactly the snagging case above.
- *`Collider::polyline`.* Zero thickness. A fast-moving ball can tunnel through, and there is nothing
  for the visible ribbon's thickness to correspond to.
- *`Collider::trimesh` over the ribbon triangles.* Concave, and internal shared edges are the classic
  source of ghost collisions for a body sliding across them.
- *`Collider::convex_decomposition`.* Solves the shape correctly but is far too expensive to re-run on
  every file save.

Using the analytic normal at each sample (rather than a mitered join) makes the ribbon fractionally
narrower than `thickness` at the joints, by a factor of `cos` of half the turn angle. At the
tolerance above that angle is small enough for the error to be well under a pixel, and accepting it
keeps the offset math to one line.

### The entity's transform is a translation to the ribbon's centroid, with no rotation

Straight segments keep `Segment::transform` — midpoint plus rotation — unchanged. A curved segment has
no single meaningful angle, so its entity sits at the centroid of its sample points with identity
rotation, and the mesh and collider vertices are expressed relative to that point. Putting the
geometry in world space with an origin transform would work too, but it would give every curve a
position of `(0, 0)` and a bounding volume spanning the level, which is worse for culling and for the
physics broadphase.

### Validation extends rather than changes

`Segment::validate` already rejects non-positive or non-finite thickness and zero or non-finite
length. Curves add two cases:

- A `control` point with a non-finite coordinate is rejected, for the same reason the existing NaN
  checks exist: it would otherwise spawn an invisible entity at an undefined position.
- A curve whose radius of curvature drops below `thickness / 2` is rejected. Below that, the inner
  offset edge crosses itself, the quads there invert, and the resulting collider is garbage in a way
  that is very hard to see on screen. Practically this is a curve bent much more sharply than it is
  thick; the diagnostic should say so, since "make it thinner or bend it less" is the fix.

Both follow the existing contract: warn naming the segment index and the reason, skip that segment,
load the rest.

## Risks / Trade-offs

- **Facet joints snag a rolling ball** → The collider quads are built from the same shared vertices as
  the mesh, so the surface is watertight with no outward notch at any joint. If snagging still shows
  up in play, `TOLERANCE` is one constant to lower.
- **Bevy's 2D pipeline may want mesh attributes beyond positions** → `ColorMaterial` on `Mesh2d`
  specializes on the attributes present, but the cheap insurance is to supply `POSITION`, `UV_0`, and
  `NORMAL` on the generated mesh. Doing that from the start avoids a debugging detour into a
  vertex-layout mismatch that surfaces as an invisible or garbled surface.
- **Rebuild cost grows with curve count** → A curve costs up to 64 quads of mesh building and one
  compound collider, per rebuild, and rebuilds happen on every save. Level 1 has a handful of
  segments, so this is comfortably fine now; the `1..=64` clamp is what keeps it bounded if a level
  grows.
- **`control` interacts with `deny_unknown_fields`** → The new fields must be declared with
  `#[serde(default)]` or an old file is fine but a new file is only readable by a new build. That is
  the intended direction (new fields, old files still valid), but it does mean a level file using
  `control` fails to parse on an older binary — with a clear parse error naming the file, which the
  existing failure path already logs.
- **Tolerance is a single global constant** → A level authored at a very different zoom or scale would
  want a different value. Left as a constant deliberately; the per-segment `subdivisions` override is
  the escape hatch, and a real need for per-level tuning can promote it to a field later.
