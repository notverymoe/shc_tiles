// Copyright 2025 Natalie Baker // Apache License v2 //

use bevy::prelude::*;

use crate::atlas::{builder::{proccess_tile_atlas_build_queue, process_tile_atlas_build_queues_with_target}, TileAtlas};

pub struct PluginTileAtlas;

impl Plugin for PluginTileAtlas {

    fn build(&self, app: &mut bevy::app::App) {
        app.init_asset::<TileAtlas>()
            .add_systems(
                Last, 
                (
                    proccess_tile_atlas_build_queue,
                    process_tile_atlas_build_queues_with_target,
                ).chain()
            );
    }

}