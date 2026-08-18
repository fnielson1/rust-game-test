## ADDED Requirements

### Requirement: A body rolls across a curved surface without catching on its approximation
A curved surface SHALL present a continuous face to colliding bodies. The pieces the system uses
internally to approximate the curve SHALL join without leaving a step, notch, or gap between them, so
that a body rolling along the surface is not slowed, deflected, or stopped at a joint, and does not
lose ground contact while crossing one.

#### Scenario: Player rolls over a hill
- **WHEN** the player rolls along a surface spawned from a curved segment forming a hill
- **THEN** the player travels continuously up and over it without catching at any point along the curve

#### Scenario: Player stays grounded across the curve
- **WHEN** the player rolls along a curved surface
- **THEN** the player is reported as grounded for the whole traverse, without flickering in and out of grounded state at the joins between the curve's internal pieces

#### Scenario: Player gathers speed in a vale
- **WHEN** the player rolls down into a surface spawned from a curved segment forming a vale
- **THEN** the player accelerates down the near side and carries that speed up the far side

### Requirement: The collided surface of a curve matches its drawn surface
The surface a body collides with SHALL occupy the same space as the surface drawn on screen, for
curved segments as for straight ones. A body SHALL NOT collide with anything outside the drawn
surface, and SHALL NOT pass through any part of it.

#### Scenario: Contact matches the visible curve
- **WHEN** the player comes to rest on a curved surface
- **THEN** the player rests visibly against the drawn surface, neither floating above it nor sinking into it

## MODIFIED Requirements

### Requirement: Segments spawn as collidable static surfaces
Each valid segment in a loaded level SHALL become one static, collidable surface that dynamic bodies
rest on and collide with, carrying the same surface properties as the game's other solid geometry.
This SHALL hold whether the segment is straight or curved: a curved segment SHALL produce a single
surface following its curve, not a chain of separate surfaces.

#### Scenario: Player rests on a horizontal segment
- **WHEN** the player falls onto a surface spawned from a horizontal segment
- **THEN** the player comes to rest on that surface and is reported as grounded

#### Scenario: Player collides with an angled segment
- **WHEN** the player falls onto a surface spawned from a sloped segment
- **THEN** the player collides with the sloped surface rather than passing through it

#### Scenario: Player collides with a curved segment
- **WHEN** the player falls onto a surface spawned from a curved segment
- **THEN** the player collides with the curved surface rather than passing through it, and is reported as grounded where the surface is shallow enough to rest on

#### Scenario: Player falls through a gap
- **WHEN** the player moves over a gap between two segments
- **THEN** the player falls through that space and is not reported as grounded

#### Scenario: A curved segment is one surface
- **WHEN** a level containing curved segments is loaded
- **THEN** the running game contains exactly as many level surfaces as the file has valid segments, regardless of how finely any curve is approximated internally

### Requirement: Saving an edited level file rebuilds the level in the running game
While running a build configured for live editing, the system SHALL detect that a loaded level file
has changed on disk and SHALL rebuild that level's surfaces from the new contents without requiring a
restart. This SHALL apply equally to changes that add, remove, or alter a curve.

#### Scenario: Segment moved while the game runs
- **WHEN** a segment's endpoints are edited and the level file saved while the game is running
- **THEN** the corresponding surface moves to its new position in the running game

#### Scenario: Segment added while the game runs
- **WHEN** a new segment is added to the level file and saved while the game is running
- **THEN** a new surface appears in the running game

#### Scenario: Control point edited while the game runs
- **WHEN** a segment's control point is moved, added, or removed and the level file saved while the game is running
- **THEN** the corresponding surface takes on the new shape in the running game, straightening if the control point was removed

#### Scenario: Rebuild does not duplicate surfaces
- **WHEN** a level file is saved repeatedly while the game is running
- **THEN** the running game contains only the surfaces described by the current file contents, with no accumulated copies from earlier loads
