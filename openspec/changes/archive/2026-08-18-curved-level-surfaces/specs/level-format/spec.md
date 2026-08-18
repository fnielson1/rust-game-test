## ADDED Requirements

### Requirement: A segment may declare a control point to become a curve
A segment SHALL accept an optional control point. A segment that declares one SHALL describe a
quadratic Bézier curve that begins at its start point, ends at its end point, and bends toward its
control point. A segment that omits the control point SHALL remain a straight line between its
endpoints, and existing level files that predate the control point SHALL continue to load unchanged.

#### Scenario: Segment without a control point stays straight
- **WHEN** a segment declares only a start point, an end point, and a thickness
- **THEN** the resulting surface is a straight line between the endpoints, identical to what the same segment produced before curves existed

#### Scenario: Control point above the line produces a hill
- **WHEN** a segment runs from `[-400, -345]` to `[100, -345]` with a control point at `[-150, -150]`
- **THEN** the resulting surface rises from `[-400, -345]`, peaks between the endpoints, and returns to `[100, -345]`

#### Scenario: Control point below the line produces a vale
- **WHEN** a segment runs from `[-400, -345]` to `[100, -345]` with a control point at `[-150, -500]`
- **THEN** the resulting surface dips below the straight line between the endpoints and returns to `[100, -345]`

#### Scenario: Curve passes through its endpoints but not its control point
- **WHEN** a curved segment is built
- **THEN** the surface touches both declared endpoints, and does not pass through the control point unless the control point lies on the line between the endpoints

#### Scenario: Off-center control point produces an asymmetric slope
- **WHEN** a segment's control point is nearer its start point than its end point
- **THEN** the resulting surface rises more steeply from the start than it falls toward the end

#### Scenario: Control point on the line between the endpoints
- **WHEN** a segment declares a control point that lies on the straight line between its start and end points
- **THEN** the resulting surface is straight, occupying the same space as the same segment with no control point

### Requirement: Curve smoothness is automatic and may be overridden
The system SHALL approximate a curved segment closely enough that the approximation is not visible
at normal viewing scale, and SHALL choose how finely to approximate it from the curve's own size, so
that a small curve and a large curve are equally smooth without the author stating anything. A
segment SHALL accept an optional subdivision count that overrides that choice.

#### Scenario: Large and small curves are equally smooth
- **WHEN** one curved segment spans 100 units and another spans 1000 units, neither declaring a subdivision count
- **THEN** both surfaces read as smooth curves, with the larger one approximated by proportionally more pieces

#### Scenario: Author overrides the subdivision count
- **WHEN** a curved segment declares a subdivision count of `3`
- **THEN** the resulting surface is built from exactly three straight pieces, producing a deliberately faceted surface

#### Scenario: Subdivision count is bounded
- **WHEN** a curved segment declares a subdivision count far larger than the supported maximum
- **THEN** the surface is built using the maximum supported count rather than the declared one, and the level still loads

## MODIFIED Requirements

### Requirement: A segment's angle is derived from its endpoints
The system SHALL derive a straight segment's orientation and length from its two endpoints, so a
segment may lie at any angle. The system SHALL derive a curved segment's shape from its two
endpoints together with its control point. The level file SHALL NOT require the author to state an
angle, a length, or a curvature separately.

#### Scenario: Horizontal segment
- **WHEN** a segment runs from `[-640, -330]` to `[-200, -330]`
- **THEN** the resulting surface is horizontal and spans 440 units

#### Scenario: Sloped segment
- **WHEN** a segment runs from `[0, 0]` to `[100, 100]`
- **THEN** the resulting surface is angled at 45 degrees and spans the distance between those two points

#### Scenario: Segment declared right-to-left
- **WHEN** a segment runs from `[200, 0]` to `[0, 0]`
- **THEN** the resulting surface occupies the same space as one declared from `[0, 0]` to `[200, 0]`

#### Scenario: Curved segment declared right-to-left
- **WHEN** a curved segment's start and end points are swapped and its control point is left unchanged
- **THEN** the resulting surface occupies the same space as before the swap

### Requirement: Thickness controls how thick a segment's surface is
Each segment SHALL carry a thickness that determines the surface's extent perpendicular to the
segment. For a straight segment, the surface SHALL be centered on the line between its endpoints. For
a curved segment, the surface SHALL be centered on the curve and follow it, extending half the
thickness to either side of the curve along its length.

#### Scenario: Thickness applied perpendicular to the segment
- **WHEN** a horizontal segment at `y = -330` declares a thickness of 30
- **THEN** the resulting surface spans from `y = -345` to `y = -315`

#### Scenario: Thickness followed around a curve
- **WHEN** a curved segment declares a thickness of 30
- **THEN** the surface is 30 units thick measured perpendicular to the curve at every point along it, not only at its endpoints

### Requirement: Degenerate segments are rejected without failing the level
The system SHALL reject any segment whose start and end points are identical, whose thickness is zero
or negative, whose control point has a non-finite coordinate, or whose curve bends more sharply than
its own thickness allows — that is, where the curve's radius of curvature falls below half the
thickness, so that the inner side of the surface would fold through itself. A rejected segment SHALL
be skipped, SHALL NOT produce a surface, and SHALL be reported with a diagnostic message identifying
which segment was rejected and why. The remaining segments in the file SHALL still load.

#### Scenario: Zero-length segment
- **WHEN** a level file contains a segment whose start and end points are both `[0, 0]`
- **THEN** that segment produces no surface, a warning identifying it is logged, and every other segment in the file still loads

#### Scenario: Non-positive thickness
- **WHEN** a level file contains a segment with a thickness of `0`
- **THEN** that segment produces no surface, a warning identifying it is logged, and every other segment in the file still loads

#### Scenario: Non-finite control point
- **WHEN** a level file contains a segment whose control point has a coordinate that is not a finite number
- **THEN** that segment produces no surface, a warning identifying it is logged, and every other segment in the file still loads

#### Scenario: Curve bent too sharply for its thickness
- **WHEN** a level file contains a curved segment whose thickness is greater than twice the tightest radius of its curve
- **THEN** that segment produces no surface, a warning naming the segment and explaining that it is bent too sharply for its thickness is logged, and every other segment in the file still loads
