## ADDED Requirements

### Requirement: Level files describe geometry as a list of line segments
A level file SHALL be a JSON document containing a list of line segments. Each segment SHALL be
described by a start point, an end point, and a thickness. The system SHALL accept any number of
segments in a single level, including zero.

#### Scenario: Level with multiple segments
- **WHEN** a level file lists three segments
- **THEN** the level consists of exactly three surfaces, one per listed segment

#### Scenario: Empty level
- **WHEN** a level file lists zero segments
- **THEN** the level loads successfully and contains no surfaces

### Requirement: A segment's angle is derived from its endpoints
The system SHALL derive each segment's orientation and length from its two endpoints, so a
segment may lie at any angle. The level file SHALL NOT require the author to state an angle
separately.

#### Scenario: Horizontal segment
- **WHEN** a segment runs from `[-640, -330]` to `[-200, -330]`
- **THEN** the resulting surface is horizontal and spans 440 units

#### Scenario: Sloped segment
- **WHEN** a segment runs from `[0, 0]` to `[100, 100]`
- **THEN** the resulting surface is angled at 45 degrees and spans the distance between those two points

#### Scenario: Segment declared right-to-left
- **WHEN** a segment runs from `[200, 0]` to `[0, 0]`
- **THEN** the resulting surface occupies the same space as one declared from `[0, 0]` to `[200, 0]`

### Requirement: Thickness controls how thick a segment's surface is
Each segment SHALL carry a thickness that determines the surface's extent perpendicular to the
line between its endpoints. The surface SHALL be centered on that line, extending half the
thickness to either side.

#### Scenario: Thickness applied perpendicular to the segment
- **WHEN** a horizontal segment at `y = -330` declares a thickness of 30
- **THEN** the resulting surface spans from `y = -345` to `y = -315`

### Requirement: Gaps are expressed by omitting segments
The system SHALL treat the space between two segments that do not share endpoints as empty. A
level file SHALL NOT need any explicit gap, hole, or spacer entry to describe a gap.

#### Scenario: Gap between two segments
- **WHEN** one segment ends at `[-200, -330]` and the next begins at `[-100, -330]`
- **THEN** no surface exists between `x = -200` and `x = -100`, and a falling body passes through that space

#### Scenario: Adjoining segments leave no gap
- **WHEN** one segment ends at `[-200, -330]` and the next begins at that same point
- **THEN** the two surfaces meet with no gap between them

### Requirement: Degenerate segments are rejected without failing the level
The system SHALL reject any segment whose start and end points are identical, or whose thickness
is zero or negative. A rejected segment SHALL be skipped, SHALL NOT produce a surface, and SHALL
be reported with a diagnostic message identifying which segment was rejected and why. The
remaining segments in the file SHALL still load.

#### Scenario: Zero-length segment
- **WHEN** a level file contains a segment whose start and end points are both `[0, 0]`
- **THEN** that segment produces no surface, a warning identifying it is logged, and every other segment in the file still loads

#### Scenario: Non-positive thickness
- **WHEN** a level file contains a segment with a thickness of `0`
- **THEN** that segment produces no surface, a warning identifying it is logged, and every other segment in the file still loads

### Requirement: Malformed and missing level files are reported, not fatal
The system SHALL report a level file that cannot be found or cannot be parsed with a diagnostic
message naming the path it attempted. The system SHALL NOT panic or terminate in either case.

#### Scenario: Level file is absent
- **WHEN** the game is started and the level file it references does not exist
- **THEN** an error naming the attempted path is logged and the game continues running

#### Scenario: Level file contains invalid JSON
- **WHEN** the level file cannot be parsed as JSON matching the segment schema
- **THEN** an error describing the parse failure is logged and the game continues running
