// Copyright 2025 Natalie Baker // Apache License v2 //

use bevy::{asset::{load_internal_asset, uuid_handle}, camera::{primitives::Aabb, visibility::{NoFrustumCulling, VisibilitySystems}}, core_pipeline::core_2d::{AlphaMask2d, Opaque2d, Transparent2d}, prelude::*, render::{render_phase::AddRenderCommand, render_resource::SpecializedRenderPipelines, Render, RenderApp, RenderSystems}};

mod command;
pub use command::*;

mod extract;
pub use extract::*;

mod pipeline;
pub use pipeline::*;

mod prepare_bind_group;
pub use prepare_bind_group::*;

mod prepare_buffers;
pub use prepare_buffers::*;

mod queue;
pub use queue::*;

use crate::sparse::TileGridSparse;

pub const HANDLE_TILE_GRID_SPARSE_SHADER: Handle<Shader> = uuid_handle!("9312854d-2cfa-4592-8731-3e17e3e3941d");

pub struct PluginTileGridSparseRender;

impl Plugin for PluginTileGridSparseRender {

    fn build(&self, app: &mut App) {
        load_internal_asset!(app, HANDLE_TILE_GRID_SPARSE_SHADER, "shader_tile_grid_sparse.wgsl", Shader::from_wgsl);
        
        app
            .add_systems(PostUpdate, tile_grid_sparse_calculate_aabbs.in_set(VisibilitySystems::CalculateBounds))
            .get_sub_app_mut(RenderApp).unwrap()
                .init_resource::<SpecializedRenderPipelines<TileGridSparsePipeline>>()
                .add_render_command::<Transparent2d, TileGridSparseDrawCommands>()
                .add_render_command::<Opaque2d,      TileGridSparseDrawCommands>()
                .add_render_command::<AlphaMask2d,   TileGridSparseDrawCommands>()
                .add_systems(ExtractSchedule, tile_grid_sparse_extract_updates)
                .add_systems(Render, (
                    (tile_grid_sparse_pipeline_init, tile_grid_sparse_prepare_buffers)
                        .chain()
                        .in_set(RenderSystems::PrepareResources),
                    tile_grid_sparse_prepare_bind_groups.in_set(RenderSystems::PrepareBindGroups),
                    tile_grid_sparse_queue_draw_commands.in_set(RenderSystems::Queue),
                ));
    }

}

type FilterUpdateOrMissingBounds = (Or<(Without<Aabb>, Changed<TileGridSparse>)>, Without<NoFrustumCulling>);

fn tile_grid_sparse_calculate_aabbs(
    mut commands: Commands,
    q_recalculate_aabb: Query<(Entity, &TileGridSparse), FilterUpdateOrMissingBounds>,
) {
    q_recalculate_aabb.iter().for_each(|(entity, tile_grid_sparse)| {
        let bounds = tile_grid_sparse.calculate_bounds();
        commands.entity(entity).insert(Aabb::from_min_max(
            bounds.min.into(),
            bounds.max.into()
        ));
    });
}
