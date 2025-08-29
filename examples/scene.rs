// Copyright 2025 Natalie Baker // Apache License v2 //

use bevy::{
    prelude::*,
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    window::PresentMode,
    camera::ScalingMode,
};

use rand::prelude::*;
use rand_xoshiro::Xoshiro256Plus;
use shc_tiles::prelude::*;

pub const CHUNK_SIZE: u32 = 16;
pub const CHUNK_LEN:  u32 = 1024_u32.div_ceil(CHUNK_SIZE);

pub const CAMERA_SPEED_MOVE_SLOW: f32 =   64.0;
pub const CAMERA_SPEED_MOVE_FAST: f32 = 1024.0;

pub const CAMERA_SPEED_ZOOM_SLOW: f32 =   8.0;
pub const CAMERA_SPEED_ZOOM_FAST: f32 = 512.0;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    present_mode: PresentMode::AutoNoVsync,
                    ..default()
                }),
                ..default()
            }),
            FrameTimeDiagnosticsPlugin::default(),
            LogDiagnosticsPlugin::default(),
            PluginsTileRender,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update,  random_chunks)
        .add_systems(Startup, camera_setup)
        .add_systems(Update,  camera_controls)
        .run();
}

fn setup(
    mut commands: Commands,
    assets: Res<AssetServer>,
    r_mta: Res<Assets<TileAtlas>>,
) {
    //  // Load Atlas Textures // //

    let atlas = r_mta.reserve_handle();
    let mut atlas_queue = TileAtlasBuildQueue::new();

    let handle = assets.load("tile_wall.png");
    atlas_queue.insert_image("base", "tile_wall", handle, TileSetSettings::new(1, 1));

    atlas_queue.lock_queue();
    commands.spawn((TileAtlasBuildQueueTarget::new(atlas.clone()), atlas_queue));

    //  // Create Dense Chunks // //

    for x in 0..CHUNK_LEN {
        for y in 0..CHUNK_LEN {
            spawn_chunk(&mut commands, CHUNK_SIZE, 4.0, -1.0, atlas.clone(), x, y);
        }
    }

}


fn random_chunks(
    mut q_chunks: Query<&mut TileGridDense>,
    r_atlases: Res<Assets<TileAtlas>>,
    mut l_rng: Local<Option<Xoshiro256Plus>>,
) {
    let Some((_, atlas)) = r_atlases.iter().next() else { return; };
    let Some(tile_wall) = atlas.get_entry("base", "tile_wall").and_then(|e| TileAtlasSlot::new(e.index)) else { return; };
    let tile_air  = TileAtlasSlot::EMPTY;
    
    let l_rng = l_rng.get_or_insert_with(Xoshiro256Plus::from_os_rng);
    for mut chunk in &mut q_chunks {
        for _ in 0..8 {
            let position = UVec2::new(
                l_rng.random_range(0..CHUNK_SIZE),
                l_rng.random_range(0..CHUNK_SIZE),
            );

            let value = if l_rng.random_bool(0.5) { tile_air } else { tile_wall };
            if value == chunk.get(position) { continue; }
            chunk.set(position, value);
        }
    }
}

fn spawn_chunk(commands: &mut Commands, size: u32, time_scale: f64, depth: f32, atlas: Handle<TileAtlas>, x: u32, y: u32) {
    commands.spawn((
        TileGridDenseBuilder::new(UVec2::splat(16), 1.0)
            .with_atlas(Some(atlas))
            .with_y_depth_scale(1.0)
            .build_with_transform_xyz((x*size) as f32, (y*size) as f32, depth),
        TileGridAnimator::new(0, 0.0, time_scale),
    ));
}

// // Camera Controls // //

fn camera_setup(
    mut commands: Commands,
)  {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            near:      0.0,
            far: 100_000.0,
            scaling_mode: ScalingMode::AutoMin { min_width: 64.0, min_height: 64.0 },
            ..OrthographicProjection::default_2d()
        }),
        Transform::from_xyz(
            (CHUNK_SIZE*CHUNK_LEN) as f32/2.0,
            (CHUNK_SIZE*CHUNK_LEN) as f32/2.0,
            0.0,
        ),
    ));
}

fn camera_controls(
    mut q_cameras: Query<(&mut Transform, &mut Projection), With<Camera>>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    // Motion //

    let mut direction = Vec2::ZERO;
    if keys.pressed(KeyCode::KeyW) { direction += Vec2::Y; }
    if keys.pressed(KeyCode::KeyA) { direction -= Vec2::X; }
    if keys.pressed(KeyCode::KeyS) { direction -= Vec2::Y; }
    if keys.pressed(KeyCode::KeyD) { direction += Vec2::X; }

    if let Some(direction) = direction.try_normalize() {
        let speed = if keys.pressed(KeyCode::ShiftLeft) { CAMERA_SPEED_MOVE_FAST } else { CAMERA_SPEED_MOVE_SLOW };
        let dt = (direction * time.delta_secs() * speed).extend(0.0);
        q_cameras.iter_mut().for_each(|(mut t, _)| { t.translation += dt; });
    }
     
    // Zoom //

    let mut zoom = 0.0;
    if keys.pressed(KeyCode::KeyQ) { zoom += 1.0; }
    if keys.pressed(KeyCode::KeyE) { zoom -= 1.0; }

    if zoom != 0.0 {
        let delta = time.delta_secs() * zoom * if keys.pressed(KeyCode::ShiftLeft) { CAMERA_SPEED_ZOOM_FAST } else { CAMERA_SPEED_ZOOM_SLOW };
        q_cameras.iter_mut().for_each(|(_, mut p)| { 
            if let Projection::Orthographic(p) = &mut *p {
                let value = (delta + if let ScalingMode::AutoMin { min_width, min_height } = p.scaling_mode {
                    min_width.min(min_height)
                } else {
                    1.0
                }).max(0.1);
                p.scaling_mode = ScalingMode::AutoMin { min_width: value, min_height: value }
            }
        });
    }
    
}
