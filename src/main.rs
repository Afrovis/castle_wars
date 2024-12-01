use std::{f32::consts::FRAC_PI_2, ops::Range};
use bevy::{
    prelude::*,
    window::CursorGrabMode,
    input::mouse::MouseMotion,
};


const CHUNK_SIZE:i16 = 64; 
const CHUNK_SIZE_HALF:i16 = CHUNK_SIZE/2; 



#[derive(Debug, Resource)]
struct CameraSettings {
    pub speed: f32,
    pub sensitivity: f32,
    pub pitch_range: Range<f32>,
}

impl Default for CameraSettings {
    fn default() -> Self {
        let pitch_limit = FRAC_PI_2 - 0.01;
        Self {
            speed: 5.0,
            sensitivity: 0.003,
            pitch_range: -pitch_limit..pitch_limit,
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<CameraSettings>()
        .add_systems(Startup, (setup, grab_cursor))
        .add_systems(Update, player_movement)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Name::new("Camera"),
        Camera3d::default(),
        Transform::from_xyz(4.0, 4.0, 4.0) // Typical Minecraft eye height
            .looking_at(Vec3::ZERO, Vec3::Z), // Look at origin, using Z as up

    ));

    // Ground plane
    // commands.spawn((
    //     Name::new("Plane"),
    //     Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0))),
    //     MeshMaterial3d(materials.add(StandardMaterial {
    //         base_color: Color::srgb(0.3, 0.5, 0.3),
    //         cull_mode: None,
    //         ..default()
    //     })),
    // ));

    // Add some test cubes
    for x in -CHUNK_SIZE_HALF..=CHUNK_SIZE_HALF {
        for z in -CHUNK_SIZE_HALF..=CHUNK_SIZE_HALF {
            commands.spawn((
                Name::new("Cube"),
                Mesh3d(meshes.add(Cuboid::default())),
                MeshMaterial3d(materials.add(Color::srgb(0.8, 0.7, 0.6))),
                Transform::from_xyz(x as f32 * 2.0, 0.5, z as f32 * 2.0),
            ));
        }
    }

    commands.spawn((
        Name::new("Light"),
        PointLight::default(),
        Transform::from_xyz(3.0, 8.0, 5.0),
    ));
}

fn grab_cursor(mut windows: Query<&mut Window>) {
    let mut window = windows.single_mut();
    window.cursor_options.grab_mode = CursorGrabMode::Locked;
    window.cursor_options.visible = false;
}

fn player_movement(
    mut camera_query: Query<&mut Transform, With<Camera>>,
    camera_settings: Res<CameraSettings>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut mouse_motion: EventReader<MouseMotion>,
    time: Res<Time>,
) {
    let mut camera = camera_query.single_mut();
    
    // Handle mouse look
    let (mut yaw, mut pitch, _) = camera.rotation.to_euler(EulerRot::YXZ);
    
    for event in mouse_motion.read() {
        pitch -= event.delta.y * camera_settings.sensitivity;
        yaw -= event.delta.x * camera_settings.sensitivity;
    }
    
    pitch = pitch.clamp(
        camera_settings.pitch_range.start,
        camera_settings.pitch_range.end,
    );
    
    camera.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);

    // Handle keyboard input
    let mut velocity = Vec3::ZERO;
    let local_z = camera.forward();
    let local_x = camera.right();

    let forward = local_z;
    let right = local_x;

    // Only use x and z components for movement
    let forward = Vec3::new(forward.x, 0.0, forward.z).normalize();
    let right = Vec3::new(right.x, 0.0, right.z).normalize();

    if keyboard.pressed(KeyCode::KeyW) {
        velocity += forward;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        velocity -= forward;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        velocity -= right;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        velocity += right;
    }
    if keyboard.pressed(KeyCode::Space) {
        velocity += Vec3::Y;
    }
    if keyboard.pressed(KeyCode::ShiftLeft) {
        velocity -= Vec3::Y;
    }

    if velocity != Vec3::ZERO {
        velocity = velocity.normalize();
    }

    camera.translation += velocity * camera_settings.speed * time.delta_secs();
}