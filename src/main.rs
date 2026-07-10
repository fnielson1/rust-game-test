// Wireframe rendering isn't supported on wasm32, so only pull in these imports on native targets.
#[cfg(not(target_arch = "wasm32"))]
use bevy::{
    input::common_conditions::input_just_pressed,
    sprite_render::{Wireframe2dConfig, Wireframe2dPlugin},
};
// Core Bevy imports used on every target: the input-condition helper and the prelude (App, Commands, etc.).
use bevy::{input::common_conditions::input_toggle_active, prelude::*};

fn main() {
    // Create the Bevy App that will own all plugins, systems, and resources.
    let mut app = App::new();
    app.add_plugins((
        // Bevy's standard plugin bundle: windowing, rendering, input, asset loading, etc.
        DefaultPlugins,
        // Only add the wireframe-rendering plugin on native builds (not supported on wasm32).
        #[cfg(not(target_arch = "wasm32"))]
        Wireframe2dPlugin::default(),
    ))
        // Run `setup` once at startup to spawn the camera, shapes, and UI text.
        .add_systems(Startup, setup);
    // On native targets, pressing Space toggles the wireframe overlay.
    #[cfg(not(target_arch = "wasm32"))]
    app.add_systems(
        Update,
        toggle_wireframe.run_if(input_just_pressed(KeyCode::Space)),
    );
    // Run `rotate` every frame, but only while rotation hasn't been toggled off by pressing R
    // (input_toggle_active(false, ...) starts in the "active" state and flips each press of R).
    app.add_systems(
        Update,
        rotate.run_if(input_toggle_active(false, KeyCode::KeyR)),
    );
    // Start the Bevy event loop; this blocks until the app exits.
    app.run();
}

// Total horizontal span (in world units) across which shapes/rings are spread out.
const X_EXTENT: f32 = 1000.;
// Vertical offset (in world units) of the shape row above/below center.
const Y_EXTENT: f32 = 150.;
// Line thickness used when converting solid shapes into ring (outline) versions.
const THICKNESS: f32 = 5.0;

// Startup system: spawns the camera, a row of solid 2D shapes, a row of their ring/outline
// counterparts, and an on-screen instructions text.
fn setup(
    mut commands: Commands,
    // Mesh storage; `meshes.add(...)` uploads geometry and returns a handle to it.
    mut meshes: ResMut<Assets<Mesh>>,
    // Material storage; `materials.add(color)` creates a solid-color material and returns a handle.
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Spawn a 2D camera so anything rendered below is actually visible.
    commands.spawn(Camera2d);

    // Build one mesh handle for each primitive 2D shape type Bevy provides.
    let shapes = [
        meshes.add(Circle::new(50.0)),
        meshes.add(CircularSector::new(50.0, 1.0)),
        meshes.add(CircularSegment::new(50.0, 1.25)),
        meshes.add(Ellipse::new(25.0, 50.0)),
        meshes.add(Annulus::new(25.0, 50.0)),
        meshes.add(Capsule2d::new(25.0, 50.0)),
        meshes.add(Rhombus::new(75.0, 100.0)),
        meshes.add(Rectangle::new(50.0, 100.0)),
        meshes.add(RegularPolygon::new(50.0, 6)),
        meshes.add(Triangle2d::new(
            Vec2::Y * 50.0,
            Vec2::new(-50.0, -50.0),
            Vec2::new(50.0, -50.0),
        )),
        meshes.add(Segment2d::new(
            Vec2::new(-50.0, 50.0),
            Vec2::new(50.0, -50.0),
        )),
        meshes.add(Polyline2d::new(vec![
            Vec2::new(-50.0, 50.0),
            Vec2::new(0.0, -50.0),
            Vec2::new(50.0, 50.0),
        ])),
    ];
    // Number of shapes, used below to evenly space them out and to spread hues across them.
    let num_shapes = shapes.len();

    // Spawn one entity per shape, positioned in a row above center.
    for (i, shape) in shapes.into_iter().enumerate() {
        // Distribute colors evenly across the rainbow.
        let color = Color::hsl(360. * i as f32 / num_shapes as f32, 0.95, 0.7);

        commands.spawn((
            // Attach the mesh geometry to this entity.
            Mesh2d(shape),
            // Attach a freshly-created color material for this shape.
            MeshMaterial2d(materials.add(color)),
            Transform::from_xyz(
                // Distribute shapes from -X_EXTENT/2 to +X_EXTENT/2.
                -X_EXTENT / 2. + i as f32 / (num_shapes - 1) as f32 * X_EXTENT,
                // Row of solid shapes sits above the vertical center.
                Y_EXTENT / 2.,
                0.0,
            ),
        ));
    }

    // Build the ring (outline) version of most of the same shapes, to display in a second row.
    let rings = [
        meshes.add(Circle::new(50.0).to_ring(THICKNESS)),
        // this visually produces an arc segment but this is not technically accurate
        meshes.add(Ring::new(
            CircularSector::new(50.0, 1.0),
            CircularSector::new(45.0, 1.0),
        )),
        meshes.add(CircularSegment::new(50.0, 1.25).to_ring(THICKNESS)),
        meshes.add({
            // This is an approximation; Ellipse does not implement Inset as concentric ellipses do not have parallel curves
            let outer = Ellipse::new(25.0, 50.0);
            let mut inner = outer;
            inner.half_size -= Vec2::splat(THICKNESS);
            Ring::new(outer, inner)
        }),
        // this is equivalent to the Annulus::new(25.0, 50.0) above
        meshes.add(Ring::new(Circle::new(50.0), Circle::new(25.0))),
        meshes.add(Capsule2d::new(25.0, 50.0).to_ring(THICKNESS)),
        meshes.add(Rhombus::new(75.0, 100.0).to_ring(THICKNESS)),
        meshes.add(Rectangle::new(50.0, 100.0).to_ring(THICKNESS)),
        meshes.add(RegularPolygon::new(50.0, 6).to_ring(THICKNESS)),
        meshes.add(
            Triangle2d::new(
                Vec2::Y * 50.0,
                Vec2::new(-50.0, -50.0),
                Vec2::new(50.0, -50.0),
            )
                .to_ring(THICKNESS),
        ),
    ];
    // Allow for 2 empty spaces (there are fewer rings than shapes since not every shape has a ring form).
    let num_rings = rings.len() + 2;

    // Spawn one entity per ring, positioned in a row below center.
    for (i, shape) in rings.into_iter().enumerate() {
        // Distribute colors evenly across the rainbow.
        let color = Color::hsl(360. * i as f32 / num_rings as f32, 0.95, 0.7);

        commands.spawn((
            Mesh2d(shape),
            MeshMaterial2d(materials.add(color)),
            Transform::from_xyz(
                // Distribute shapes from -X_EXTENT/2 to +X_EXTENT/2.
                -X_EXTENT / 2. + i as f32 / (num_rings - 1) as f32 * X_EXTENT,
                // Row of rings sits below the vertical center.
                -Y_EXTENT / 2.,
                0.0,
            ),
        ));
    }

    // Base instructions text shown on every target.
    #[allow(unused_mut)]
    let mut text = "Press 'R' to pause/resume rotation".to_string();
    // Wireframe toggle only exists on native builds, so only mention it there.
    #[cfg(not(target_arch = "wasm32"))]
    text.push_str("\nPress 'Space' to toggle wireframes");

    // Spawn a UI text node pinned to the top-left corner of the screen.
    commands.spawn((
        Text::new(text),
        Node {
            // Absolute positioning relative to the UI root, not the layout flow.
            position_type: PositionType::Absolute,
            top: px(12),
            left: px(12),
            ..default()
        },
    ));
}

// System (native-only) that flips the global wireframe-rendering flag when Space is pressed.
#[cfg(not(target_arch = "wasm32"))]
fn toggle_wireframe(mut wireframe_config: ResMut<Wireframe2dConfig>) {
    wireframe_config.global = !wireframe_config.global;
}

// System that spins every mesh entity around the Z axis each frame, unless paused via `R`.
fn rotate(mut query: Query<&mut Transform, With<Mesh2d>>, time: Res<Time>) {
    for mut transform in &mut query {
        // Rotate at a constant angular speed, scaled by elapsed frame time for frame-rate independence.
        transform.rotate_z(time.delta_secs() / 2.0);
    }
}
