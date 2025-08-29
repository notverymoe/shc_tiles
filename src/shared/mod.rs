// Copyright 2025 Natalie Baker // Apache License v2 //

use bevy::{asset::{load_internal_asset, uuid_handle}, prelude::*, render::render_resource::BlendState};

mod tile_grid_animator;
pub use tile_grid_animator::*;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum RenderPass2d {
    #[default]
    Transparent,
    AlphaMask,
    Opaque,
}

impl From<RenderPass2d> for Option<BlendState> {
    fn from(value: RenderPass2d) -> Self {
        match value {
            RenderPass2d::Opaque      => Some(BlendState::REPLACE),
            RenderPass2d::AlphaMask   => Some(BlendState::ALPHA_BLENDING),
            RenderPass2d::Transparent => Some(BlendState::ALPHA_BLENDING),
        }
    }
}

pub const HANDLE_SHARED_ATLAS_SHADER:     Handle<Shader> = uuid_handle!("019841ac-2179-715f-a6b5-9439a49c9c60");
pub const HANDLE_SHARED_UTIL_SHADER:      Handle<Shader> = uuid_handle!("019841ac-1679-715f-a6b5-9439a49c9c60");
pub const HANDLE_SHARED_TILE_GRID_SHADER: Handle<Shader> = uuid_handle!("019841ac-1621-715f-a6b5-9439a49c9c60");

#[derive(Debug)]
pub struct PluginTileGridShared;

impl Plugin for PluginTileGridShared {

    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, update_tile_grid_animator);
    }

    fn finish(&self, app: &mut App) {
        load_internal_asset!(app, HANDLE_SHARED_ATLAS_SHADER,     "atlas.wgsl",     Shader::from_wgsl);
        load_internal_asset!(app, HANDLE_SHARED_UTIL_SHADER,      "util.wgsl",      Shader::from_wgsl);   
        load_internal_asset!(app, HANDLE_SHARED_TILE_GRID_SHADER, "tile_grid.wgsl", Shader::from_wgsl);
    }

}