mod intersection;

use std::time::Duration;

use bevy::{
    prelude::*,
    render::{camera::Viewport, view::RenderLayers},
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use rand::Rng;

#[derive(Component)]
struct Line {
    length: f32,
}

#[derive(Component)]
struct Player {}

#[derive(Component)]
struct PlayerLine {}

#[derive(Component)]
struct Velocity {
    translation: Vec2,
    rotation: f32,
}
impl Velocity {
    fn new() -> Self {
        Self {
            translation: Vec2::new(0.0, 0.0),
            rotation: 0.0,
        }
    }
}

#[derive(Component)]
struct MainCamera {}

#[derive(Component)]
struct MinimapCamera {}

fn setup_main_camera(mut commands: Commands) {
    commands
        .spawn(Camera2dBundle::default())
        .insert(MainCamera {});
}

fn add_line(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    let length = rand::thread_rng().gen_range(100.0..500.0);
    commands
        .spawn(MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(Capsule2d::new(2.0, length))),
            material: materials.add(Color::rgb(1.0, 1.0, 1.0)),
            transform: Transform::from_xyz(
                rand::thread_rng().gen_range(-5000.0..5000.0),
                rand::thread_rng().gen_range(-5000.0..5000.0),
                0.0,
            )
            .with_rotation(Quat::from_rotation_z(
                rand::thread_rng().gen_range(0.0_f32..360.0_f32.to_radians()),
            )),
            ..default()
        })
        .insert(Line { length })
        .insert(Velocity::new());
}

#[derive(Component)]
struct StickSpawnTimer {
    timer: Timer,
}

fn setup_spawn_timer(mut commands: Commands) {
    add_spawn_timer(&mut commands);
}

fn add_spawn_timer(commands: &mut Commands) {
    commands.spawn(StickSpawnTimer {
        timer: Timer::new(
            Duration::from_millis(rand::thread_rng().gen_range(1..2000)),
            TimerMode::Once,
        ),
    });
}

fn spawn_sticks(
    mut commands: Commands,
    mut q: Query<(Entity, &mut StickSpawnTimer)>,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (entity, mut timer) in q.iter_mut() {
        timer.timer.tick(time.delta());

        if timer.timer.finished() {
            commands.entity(entity).despawn();
            add_line(&mut commands, &mut meshes, &mut materials);
            add_spawn_timer(&mut commands);
        }
    }
}

fn setup_minimap(mut commands: Commands) {
    let minimap_size = 256;
    commands
        .spawn((Camera2dBundle {
            camera: Camera {
                // renders after / on top of other cameras
                order: 2,
                // set the viewport to a 256x256 square in the top left corner
                viewport: Some(Viewport {
                    physical_position: UVec2::new(0, 0),
                    physical_size: UVec2::new(minimap_size, minimap_size),
                    ..default()
                }),
                ..default()
            },
            ..default()
        },))
        .insert(MinimapCamera {});
}

fn setup_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands
        .spawn(MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(Capsule2d::new(2.0, 200.0))),
            material: materials.add(Color::rgb(0.0, 1.0, 0.0)),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        })
        .insert(Line {length: 200.0})
        .insert(Player {})
        .insert(PlayerLine {})
        .insert(Velocity::new());
}

fn handle_keys(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut players: Query<&mut Velocity, With<Player>>,
) {
    for mut velocity in players.iter_mut() {
        let velocity_per_sec = 2000.0;
        let max_velocity = 200.0;
        let rotational_velocity_per_sec = 450.0_f32.to_radians();
        let max_rotational_velocity = 45.0_f32.to_radians();
        if keys.pressed(KeyCode::KeyW) {
            velocity.translation.y += velocity_per_sec * time.delta_seconds();
        }
        if keys.pressed(KeyCode::KeyS) {
            velocity.translation.y -= velocity_per_sec * time.delta_seconds();
        }
        if keys.pressed(KeyCode::KeyA) {
            velocity.translation.x -= velocity_per_sec * time.delta_seconds();
        }
        if keys.pressed(KeyCode::KeyD) {
            velocity.translation.x += velocity_per_sec * time.delta_seconds();
        }
        if keys.pressed(KeyCode::KeyQ) {
            velocity.rotation += rotational_velocity_per_sec * time.delta_seconds();
        }
        if keys.pressed(KeyCode::KeyE) {
            velocity.rotation -= rotational_velocity_per_sec * time.delta_seconds();
        }
        velocity.translation.x = velocity.translation.x.clamp(-max_velocity, max_velocity);
        velocity.translation.y = velocity.translation.y.clamp(-max_velocity, max_velocity);
        velocity.rotation = velocity
            .rotation
            .clamp(-max_rotational_velocity, max_rotational_velocity);
    }
}

fn sync_cameras(
    mut main_camera: Query<
        &mut Transform,
        (
            With<MainCamera>,
            Without<MinimapCamera>,
            Without<PlayerLine>,
        ),
    >,
    mut minimap_camera: Query<&mut Transform, (With<MinimapCamera>, Without<PlayerLine>)>,
    player_lines: Query<(&Transform, &Line), With<PlayerLine>>,
    window: Query<&Window>,
    time: Res<Time>,
) {
    let window = window.single();
    let mut main_camera = main_camera.single_mut();
    let mut minimap_camera = minimap_camera.single_mut();

    let (min, max) = player_lines.iter().fold(
        (
            Vec2::new(f32::INFINITY, f32::INFINITY),
            Vec2::new(f32::NEG_INFINITY, f32::NEG_INFINITY),
        ),
        |curr, (line_transform, line)| {
            let line_endpoints = intersection::convert_to_endpoints(
                line_transform.translation.xy(),
                line.length,
                line_transform.rotation.to_euler(EulerRot::XYZ).2 + std::f32::consts::PI / 2.0,
            );
            (
                Vec2::new(
                    curr.0.x.min(line_endpoints.0.x).min(line_endpoints.1.x),
                    curr.0.y.min(line_endpoints.0.y).min(line_endpoints.1.y),
                ),
                Vec2::new(
                    curr.1.x.max(line_endpoints.0.x).max(line_endpoints.1.x),
                    curr.1.y.max(line_endpoints.0.y).max(line_endpoints.1.y),
                ),
            )
        },
    );

    {
        let center = (min + max) / 2.0;
        let new_translation = Vec3::new(center.x, center.y, 0.0);
        main_camera.translation = main_camera
            .translation
            .lerp(new_translation, time.delta_seconds() * 5.0);
        minimap_camera.translation = new_translation;
    }

    let full_player_size = max - min;
    let window_size = {
        let window_width = window.width();
        let window_height = window.height();
        Vec2::new(window_width, window_height)
    };

    let width_ratio = full_player_size.x / window_size.x;
    let height_ratio = full_player_size.y / window_size.y;
    let max_dimension_ratio = width_ratio.max(height_ratio);

    let target_max_ratio = 0.50;
    let scale_factor = target_max_ratio / max_dimension_ratio;

    let new_camera_scale = Vec3::new(1.0 / scale_factor, 1.0 / scale_factor, 1.0);
    main_camera.scale = main_camera
        .scale
        .lerp(new_camera_scale, time.delta_seconds());
    minimap_camera.scale = Vec3::new(
        1.0 / (scale_factor / 50.0),
        1.0 / (scale_factor / 50.0),
        1.0,
    );
}

fn move_velocity(
    time: Res<Time>,
    mut players: Query<(&mut Velocity, &mut Transform), With<Player>>,
    mut player_lines: Query<&mut Transform, (With<PlayerLine>, Without<Player>)>,
) {
    let (mut player_velocity, mut player_transform) = players.single_mut();
    let rotation = Quat::from_rotation_z(player_velocity.rotation * time.delta_seconds());
    for mut transform in player_lines.iter_mut() {
        transform.translation += Vec3::new(
            player_velocity.translation.x,
            player_velocity.translation.y,
            0.0,
        ) * time.delta_seconds();
        {
            let relative_position = transform.translation - player_transform.translation;
            let rotated_position = rotation * relative_position;
            transform.translation = rotated_position + player_transform.translation;
            transform.rotate(rotation);
        }
    }

    player_transform.translation += Vec3::new(
        player_velocity.translation.x,
        player_velocity.translation.y,
        0.0,
    ) * time.delta_seconds();
    player_transform.rotate(rotation);
    player_velocity.translation *= 1.0 - (0.95 * time.delta_seconds());
    player_velocity.rotation *= 1.0 - (0.95 * time.delta_seconds());
}

fn handle_collisions(
    mut commands: Commands,
    player_lines: Query<(&Transform, &Line), With<PlayerLine>>,
    items: Query<(Entity, &Transform, &Line), Without<PlayerLine>>,
) {
    for (player_transform, player_line) in player_lines.iter() {
        let player_endpoints = intersection::convert_to_endpoints(
            player_transform.translation.xy(),
            player_line.length,
            player_transform.rotation.to_euler(EulerRot::XYZ).2 + std::f32::consts::PI / 2.0,
        );
        for item in items.iter() {
            // TODO: This should be calculated outside of the outer loop and stored in case there
            // are multiple players, which there aren't right now so it isn't an issue I guess
            let item_endpoints = intersection::convert_to_endpoints(
                item.1.translation.xy(),
                item.2.length,
                item.1.rotation.to_euler(EulerRot::XYZ).2 + std::f32::consts::PI / 2.0,
            );
            let intersection_point = intersection::lines_intersect(
                player_endpoints.0,
                player_endpoints.1,
                item_endpoints.0,
                item_endpoints.1,
            );
            if intersection_point.is_some() {
                commands.entity(item.0).insert(PlayerLine {});
            }
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup_main_camera)
        .add_systems(Startup, setup_minimap.after(setup_main_camera))
        .add_systems(Startup, setup_player)
        .add_systems(Startup, setup_spawn_timer)
        .add_systems(Update, handle_keys)
        .add_systems(Update, sync_cameras)
        .add_systems(Update, move_velocity)
        .add_systems(Update, handle_collisions)
        .add_systems(Update, spawn_sticks)
        .run();
}
