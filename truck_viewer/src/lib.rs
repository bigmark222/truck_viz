use bevy::{gltf::GltfAssetLabel, input::mouse::MouseMotion, prelude::*};

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
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Truck mesh with Bevy".into(),
            ..Default::default()
        }),
        ..Default::default()
    }))
    .add_systems(Startup, setup)
    .add_systems(Update, (camera_movement, model_rotation, force_unlit_materials));
    app
}

pub fn run() {
    build_app().run();
}

#[derive(Component)]
struct ModelRoot;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
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
        SceneRoot(scene),
        ModelRoot,
        Transform::default(),
        GlobalTransform::default(),
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
