use bevy::{
    gltf::GltfAssetLabel,
    input::mouse::MouseMotion,
    math::primitives::{Cuboid, Plane3d, Sphere},
    prelude::*,
    window::PrimaryWindow,
};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use bevy_rapier3d::prelude::*;
use bevy_rapier3d::render::DebugRenderContext;

#[derive(Component)]
struct FlyCamera {
    speed: f32,
    boost: f32,
}

impl Default for FlyCamera {
    fn default() -> Self {
        Self {
            speed: 5.0,
            boost: 12.0,
        }
    }
}
pub fn build_app() -> App {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Truck mesh with Bevy".into(),
                ..Default::default()
            }),
            ..Default::default()
        }),
        RapierPhysicsPlugin::<NoUserData>::default(),
        RapierDebugRenderPlugin::default().disabled(),
        EguiPlugin::default(),
        WorldInspectorPlugin::new(),
    ))
    .register_type::<SimControls>()
    .init_resource::<CursorRay>()
    .init_resource::<Selection>()
    .init_resource::<SimControls>()
    .init_resource::<SpawnQueue>()
    .add_systems(Startup, setup)
    .add_systems(
        Update,
        (
            camera_movement,
            model_rotation,
            force_unlit_materials,
            update_cursor_ray,
            click_select,
            apply_sim_controls,
            process_spawn_queue,
        ),
    );
    app
}

pub fn run() {
    build_app().run();
}

#[derive(Component)]
struct ModelRoot;
#[derive(Component)]
struct MainCamera;
#[derive(Component)]
struct Selectable;

#[derive(Resource, Default)]
struct CursorRay {
    origin: Vec3,
    dir: Vec3,
    valid: bool,
}

#[derive(Resource, Default)]
struct Selection {
    selected: Option<Entity>,
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
struct SimControls {
    physics_on: bool,
    gravity_on: bool,
    debug_draw: bool,
}

impl Default for SimControls {
    fn default() -> Self {
        Self {
            physics_on: true,
            gravity_on: true,
            debug_draw: false,
        }
    }
}

#[derive(Default, Resource)]
struct SpawnQueue {
    cubes: u32,
    spheres: u32,
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(ClearColor(Color::srgb(0.06, 0.06, 0.08)));
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        // Brighten ambient to soften shadows while keeping some contrast.
        brightness: 1.8,
        affects_lightmapped_meshes: true,
    });

    // Basic camera with fly controller
    commands
        .spawn((
            Camera3d::default(),
            Transform::from_xyz(5.0, 4.0, 5.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
            FlyCamera::default(),
            MainCamera,
        ))
        .with_children(|parent| {
            // Light follows the camera orientation so shaded faces shift as you orbit.
            parent.spawn((
                DirectionalLight {
                    shadows_enabled: false,
                    illuminance: 4_500.0,
                    ..Default::default()
                },
                Transform::from_xyz(1.0, -1.0, -1.0).looking_at(Vec3::ZERO, Vec3::Y),
            ));
        });

    // Stationary fill and rim lights to avoid flat shading.
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            // Minimal shadowing—just enough to pick out edges.
            illuminance: 300.0,
            ..Default::default()
        },
        Transform::from_xyz(-2.0, -1.5, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        PointLight {
            intensity: 3_000.0,
            range: 20.0,
            shadows_enabled: false,
            ..Default::default()
        },
        Transform::from_xyz(0.0, 3.0, 0.0),
    ));

    commands.spawn((
        PointLight {
            intensity: 2_500.0,
            range: 18.0,
            shadows_enabled: false,
            color: Color::hsl(40.0, 0.1, 0.6),
            ..Default::default()
        },
        Transform::from_xyz(4.0, 2.5, 4.0),
    ));

    commands.spawn((
        PointLight {
            intensity: 2_500.0,
            range: 18.0,
            shadows_enabled: false,
            color: Color::hsl(220.0, 0.1, 0.5),
            ..Default::default()
        },
        Transform::from_xyz(-4.0, 2.5, -4.0),
    ));

    // Soft rim light to pick out silhouettes.
    commands.spawn((
        DirectionalLight {
            shadows_enabled: false,
            illuminance: 1_000.0,
            ..Default::default()
        },
        Transform::from_xyz(0.0, 1.5, -1.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Let Bevy load and watch the GLTF; with BEVY_ASSET_WATCHER=poll, edits under assets/ hot-reload.
    // Change the file name below (e.g., to "cube.gltf" or "bottle.gltf") to view a different export.
    let scene_path = GltfAssetLabel::Scene(0).from_asset("organic.gltf");
    let scene: Handle<Scene> = asset_server.load(scene_path);
    commands.spawn((
        RigidBody::Dynamic,
        // Rough bounding box; swap for a mesh collider once loaded if needed.
        Collider::cuboid(1.5, 1.0, 3.0),
        Selectable,
        SceneRoot(scene),
        ModelRoot,
        Transform::default(),
        GlobalTransform::default(),
    ));

    // Simple physics ground so the model has a floor to rest on.
    commands.spawn((
        Mesh3d(meshes.add(Mesh::from(Plane3d::default().mesh().size(40.0, 40.0)))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.05, 0.05, 0.06),
            ..Default::default()
        })),
        Transform::from_xyz(0.0, -0.05, 0.0),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        RigidBody::Fixed,
        Collider::cuboid(20.0, 0.05, 20.0),
        Selectable,
    ));
}

fn camera_movement(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&FlyCamera, &mut Transform)>,
) {
    for (controller, mut transform) in &mut query {
        let mut direction = Vec3::ZERO;

        let forward = transform.forward().as_vec3();
        let right = transform.right().as_vec3();
        let flat_forward = Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero();
        let flat_right = Vec3::new(right.x, 0.0, right.z).normalize_or_zero();

        if keys.pressed(KeyCode::KeyW) {
            direction += forward;
        }
        if keys.pressed(KeyCode::KeyS) {
            direction -= forward;
        }
        if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
            direction -= right;
        }
        if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
            direction += right;
        }
        // Arrow keys move on the ground plane in the camera's view directions (no vertical motion).
        if keys.pressed(KeyCode::ArrowUp) {
            direction += flat_forward;
        }
        if keys.pressed(KeyCode::ArrowDown) {
            direction -= flat_forward;
        }
        if keys.pressed(KeyCode::ArrowLeft) {
            direction -= flat_right;
        }
        if keys.pressed(KeyCode::ArrowRight) {
            direction += flat_right;
        }
        if keys.pressed(KeyCode::Space) {
            direction += Vec3::Y;
        }
        if keys.pressed(KeyCode::ShiftLeft) {
            direction -= Vec3::Y;
        }

        let speed = if keys.pressed(KeyCode::ShiftRight) {
            controller.boost
        } else {
            controller.speed
        };

        if direction != Vec3::ZERO {
            transform.translation += direction.normalize() * speed * time.delta_secs();
        }
    }
}

fn update_cursor_ray(
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut cursor_ray: ResMut<CursorRay>,
) {
    cursor_ray.valid = false;
    let Some(window) = windows.iter().next() else {
        return;
    };
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    let Some((camera, transform)) = cameras.iter().next() else {
        return;
    };

    if let Ok(ray) = camera.viewport_to_world(transform, cursor_pos) {
        cursor_ray.origin = ray.origin;
        cursor_ray.dir = ray.direction.into();
        cursor_ray.valid = true;
    }
}

fn click_select(
    buttons: Res<ButtonInput<MouseButton>>,
    cursor_ray: Res<CursorRay>,
    mut selection: ResMut<Selection>,
    rapier_context: ReadRapierContext<With<DefaultRapierContext>>,
    pickables: Query<(), With<Selectable>>,
) {
    if !buttons.just_pressed(MouseButton::Left) || !cursor_ray.valid {
        return;
    }

    let Ok(context) = rapier_context.single() else {
        return;
    };

    if let Some((entity, _toi)) = context.cast_ray(
        cursor_ray.origin,
        cursor_ray.dir,
        10_000.0,
        true,
        QueryFilter::default(),
    ) {
        if pickables.get(entity).is_ok() {
            selection.selected = Some(entity);
            info!("Selected entity {:?}", entity);
            return;
        }
    }

    selection.selected = None;
    info!("Cleared selection");
}

fn apply_sim_controls(
    controls: Res<SimControls>,
    mut configs: Query<&mut RapierConfiguration, With<DefaultRapierContext>>,
    mut debug_render: ResMut<DebugRenderContext>,
) {
    if let Ok(mut config) = configs.single_mut() {
        config.physics_pipeline_active = controls.physics_on;
        config.gravity = if controls.gravity_on {
            Vec3::new(0.0, -9.81, 0.0)
        } else {
            Vec3::ZERO
        };
    }

    debug_render.enabled = controls.debug_draw;
}

fn process_spawn_queue(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut queue: ResMut<SpawnQueue>,
) {
    if queue.cubes == 0 && queue.spheres == 0 {
        return;
    }

    let spawn_height = 3.0;
    for _ in 0..queue.cubes {
        commands.spawn((
            Mesh3d(meshes.add(Mesh::from(Cuboid::default()))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.35, 0.45, 0.65),
                ..Default::default()
            })),
            Transform::from_xyz(0.0, spawn_height, 0.0),
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
            RigidBody::Dynamic,
            Collider::cuboid(0.5, 0.5, 0.5),
            Selectable,
        ));
    }

    for _ in 0..queue.spheres {
        commands.spawn((
            Mesh3d(meshes.add(Mesh::from(Sphere { radius: 0.6 }))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.55, 0.45, 0.35),
                ..Default::default()
            })),
            Transform::from_xyz(0.0, spawn_height, 1.5),
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
            RigidBody::Dynamic,
            Collider::ball(0.6),
            Selectable,
        ));
    }

    queue.cubes = 0;
    queue.spheres = 0;
}

fn model_rotation(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: MessageReader<MouseMotion>,
    mut query: Query<&mut Transform, With<ModelRoot>>,
) {
    if !mouse_buttons.pressed(MouseButton::Right) {
        mouse_motion.clear();
        return;
    }

    let mut delta = Vec2::ZERO;
    for motion in mouse_motion.read() {
        delta += motion.delta;
    }

    if delta == Vec2::ZERO {
        return;
    }

    let sensitivity = 0.003;
    // Flip signs so drag direction matches visual rotation.
    let yaw = delta.x * sensitivity;
    let pitch = delta.y * sensitivity;

    for mut transform in &mut query {
        let yaw_rot = Quat::from_axis_angle(Vec3::Y, yaw);
        let pitch_rot = Quat::from_axis_angle(Vec3::X, pitch);
        transform.rotation = yaw_rot * pitch_rot * transform.rotation;
    }
}

// Keep materials lit but very rough to soften highlights/shadows.
fn force_unlit_materials(mut materials: ResMut<Assets<StandardMaterial>>) {
    for (_, mat) in materials.iter_mut() {
        mat.unlit = false;
        mat.metallic = 0.0;
        mat.reflectance = 0.0;
        mat.perceptual_roughness = 1.0;
    }
}
