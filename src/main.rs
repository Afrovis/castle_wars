use std::{f32::consts::FRAC_PI_2, ops::Range};
use bevy::{
    prelude::*,
    window::CursorGrabMode,
    window::Window,
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
        .add_systems(Update, place_block)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera
    commands.spawn((
        Name::new("Camera"),
        Camera3d::default(),
        Transform::from_xyz(4.0, 4.0, 4.0)
            .looking_at(Vec3::ZERO, Vec3::Z),
    ));

    // Light
    commands.spawn((
        Name::new("Light"),
        PointLight::default(),
        Transform::from_xyz(3.0, 8.0, 5.0),
    ));

    // Create shared mesh and material for instancing
    let cube_mesh = meshes.add(Cuboid::default());
    let cube_material = materials.add(Color::srgb(0.8, 0.7, 0.6));

    // Spawn cubes using the same mesh and material handles
    for x in -CHUNK_SIZE_HALF..=CHUNK_SIZE_HALF {
        for z in -CHUNK_SIZE_HALF..=CHUNK_SIZE_HALF {
            commands.spawn((
                Name::new("Cube"),
                Mesh3d(cube_mesh.clone()),
                MeshMaterial3d(cube_material.clone()),
                Transform::from_xyz(x as f32 * 1.0, 0.5, z as f32 * 1.0),
            ));
        }
    }
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

fn place_block(
    camera_query: Query<(&Camera, &GlobalTransform)>,
    window_query: Query<&Window>,
    block_query: Query<&Transform>, // Simplified query
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    let (camera, camera_transform) = camera_query.single();
    let window = window_query.single();

    if let Some(cursor_position) = window.cursor_position() {
        if let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) {
            let max_distance = 10.0;
            let ray_direction = ray.direction.normalize();
            let ray_origin = ray.origin;

            let mut hit_position = None;
            let mut hit_normal = None;
            let mut closest_distance = max_distance;

            // Check for intersections with existing blocks
            for transform in block_query.iter() {
                let block_pos = transform.translation;
                // Define block bounds (assuming 1x1x1 blocks)
                let min = block_pos - Vec3::splat(0.5);
                let max = block_pos + Vec3::splat(0.5);

                // Ray-box intersection check
                if let Some((t, normal)) = ray_box_intersection(ray_origin, ray_direction, min, max) {
                    if t < closest_distance {
                        closest_distance = t;
                        hit_position = Some(ray_origin + ray_direction * t);
                        hit_normal = Some(normal);
                    }
                }
            }

            // If we hit a block, place a new one adjacent to it
            if let (Some(hit_pos), Some(normal)) = (hit_position, hit_normal) {
                // Round the hit position first
                let hit_pos_rounded = Vec3::new(
                    hit_pos.x.round(),
                    hit_pos.y.round()-0.5,
                    hit_pos.z.round(),
                );
                
                // Then add the normal to get the new block position
                let grid_pos = hit_pos_rounded + normal;

                println!("Hit pos: {:?}", hit_pos);
                println!("Hit pos rounded: {:?}", hit_pos_rounded);
                println!("Normal: {:?}", normal);
                println!("Final grid pos: {:?}", grid_pos);

                // Create shared mesh and material
                let cube_mesh = meshes.add(Cuboid::default());
                let cube_material = materials.add(Color::srgb(0.8, 0.7, 0.6));

                // Spawn a new cube at the grid position
                commands.spawn((
                    Name::new("Cube"),
                    Mesh3d(cube_mesh),
                    MeshMaterial3d(cube_material),
                    Transform::from_translation(grid_pos),
                ));
            }
        }
    }
}



fn ray_box_intersection(
    ray_origin: Vec3,
    ray_direction: Vec3,
    box_min: Vec3,
    box_max: Vec3,
) -> Option<(f32, Vec3)> {
    let mut tmin = (box_min.x - ray_origin.x) / ray_direction.x;
    let mut tmax = (box_max.x - ray_origin.x) / ray_direction.x;

    if tmin > tmax {
        std::mem::swap(&mut tmin, &mut tmax);
    }

    let mut tymin = (box_min.y - ray_origin.y) / ray_direction.y;
    let mut tymax = (box_max.y - ray_origin.y) / ray_direction.y;

    if tymin > tymax {
        std::mem::swap(&mut tymin, &mut tymax);
    }

    if tmin > tymax || tymin > tmax {
        return None;
    }

    if tymin > tmin {
        tmin = tymin;
    }

    if tymax < tmax {
        tmax = tymax;
    }

    let mut tzmin = (box_min.z - ray_origin.z) / ray_direction.z;
    let mut tzmax = (box_max.z - ray_origin.z) / ray_direction.z;

    if tzmin > tzmax {
        std::mem::swap(&mut tzmin, &mut tzmax);
    }

    if tmin > tzmax || tzmin > tmax {
        return None;
    }

    if tzmin > tmin {
        tmin = tzmin;
    }

    if tzmax < tmax {
        tmax = tzmax;
    }

    if tmin < 0.0 {
        return None;
    }

    // Calculate the hit point and normal
    let hit_point = ray_origin + ray_direction * tmin;
    let center = (box_min + box_max) * 0.5;
    let half_size = (box_max - box_min) * 0.5;
    
    // Use a smaller epsilon value
    const EPSILON: f32 = 0.0001;
    
    // Calculate the relative position from the center
    let relative_pos = (hit_point - center).abs();
    
    // Determine which face was hit by comparing distances
    let normal = if (relative_pos.x - half_size.x).abs() < EPSILON {
        Vec3::new(if hit_point.x > center.x { 1.0 } else { -1.0 }, 0.0, 0.0)
    } else if (relative_pos.y - half_size.y).abs() < EPSILON {
        Vec3::new(0.0, if hit_point.y > center.y { 1.0 } else { -1.0 }, 0.0)
    } else {
        Vec3::new(0.0, 0.0, if hit_point.z > center.z { 1.0 } else { -1.0 })
    };

    Some((tmin, normal))
}