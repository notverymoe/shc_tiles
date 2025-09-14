// Copyright 2025 Natalie Baker // Apache License v2 //

use bevy::{asset::{load_internal_asset, uuid_handle}, camera::{primitives::Aabb, visibility::{NoFrustumCulling, VisibilitySystems}}, core_pipeline::core_2d::{AlphaMask2d, Opaque2d, Transparent2d}, prelude::*, render::{render_phase::AddRenderCommand, render_resource::SpecializedRenderPipelines, Render, RenderApp, RenderSystems}};

mod command;
pub use command::*;

mod extract;
pub use extract::*;

mod queue;
pub use queue::*;

mod pipeline;
pub use pipeline::*;

mod prepare_buffers;
pub use prepare_buffers::*;

mod prepare_bind_group;
pub use prepare_bind_group::*;

use super::TileGridDense;

pub const HANDLE_TILE_GRID_DENSE_SHADER: Handle<Shader> = uuid_handle!("7c3507b6-98f3-4d3a-9694-96c042ff9fde");

pub struct PluginTileGridDenseRender;

impl Plugin for PluginTileGridDenseRender {

    fn build(&self, app: &mut App) {
        load_internal_asset!(app, HANDLE_TILE_GRID_DENSE_SHADER, "shader_tile_grid_dense.wgsl", Shader::from_wgsl);
        
        app
            .add_systems(PostUpdate, tile_grid_dense_calculate_aabbs.in_set(VisibilitySystems::CalculateBounds))
            .get_sub_app_mut(RenderApp).unwrap()
                .init_resource::<SpecializedRenderPipelines<TileGridDensePipeline>>()
                .add_render_command::<Transparent2d, TileGridDenseDrawCommands>()
                .add_render_command::<Opaque2d,      TileGridDenseDrawCommands>()
                .add_render_command::<AlphaMask2d,   TileGridDenseDrawCommands>()
                .add_systems(
                    ExtractSchedule,
                    tile_grid_dense_extract_updates, // TODO HACK doing ExtractAssetsSet doesnt fix texture extract issue
                )
                .add_systems(Render, (
                    (
                        tile_grid_dense_pipeline_init,
                        tile_grid_dense_prepare_buffers,
                    ).chain().in_set(RenderSystems::PrepareResources),
                    tile_grid_dense_prepare_bind_groups.in_set(RenderSystems::PrepareBindGroups),
                    tile_grid_dense_queue_draw_commands.in_set(RenderSystems::Queue),
                ));
    }

}

type FilterUpdateOrMissingBounds = (Or<(Without<Aabb>, Changed<TileGridDense>)>, Without<NoFrustumCulling>);

fn tile_grid_dense_calculate_aabbs(
    mut commands: Commands,
    q_recalculate_aabb: Query<(Entity, &TileGridDense), FilterUpdateOrMissingBounds>,
) {
    q_recalculate_aabb.iter().for_each(|(entity, tile_grid_dense)| {
        let [min, max] = tile_grid_dense.calculate_bounds();
        commands.entity(entity).insert(Aabb::from_min_max(min, max));
    });
}
