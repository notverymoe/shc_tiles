// Copyright 2025 Natalie Baker // Apache License v2 //

use bevy::{prelude::*, render::{render_asset::RenderAssets, settings::WgpuLimits, sync_world::RenderEntity, texture::GpuImage, Extract}};
use bytemuck::{Pod, Zeroable};

use crate::{atlas::TileAtlas, shared::{RenderPass2d, TileGridAnimator}, sparse::{render::{PreparedTileGridSparse, TileGridSparseBindGroups}, TileGridSparse}};

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
#[allow(clippy::partial_pub_fields)]
pub struct TileGridSparseUniforms {
    pub origin: Vec2,
    pub scale:  f32,
    pub depth:  f32,
    pub y_depth_scale: f32,
    pub frame_time: u32,
}

impl PartialEq for TileGridSparseUniforms {
    fn eq(&self, other: &Self) -> bool {
        // Don't compare padding
        self.origin         == other.origin && 
        self.scale          == other.scale && 
        self.depth          == other.depth &&
        self.y_depth_scale  == other.y_depth_scale &&
        self.frame_time == other.frame_time
    }
}

impl TileGridSparseUniforms {
    pub const BINDING_SIZE:   usize = core::mem::size_of::<TileGridSparseUniforms>();
    pub const BINDING_OFFSET: usize = Self::BINDING_SIZE.next_multiple_of(WgpuLimits::downlevel_defaults().min_storage_buffer_offset_alignment as usize);
}

#[derive(Debug, Component)]
#[require(PreparedTileGridSparse, TileGridSparseBindGroups)]
pub struct TileGridSparseExtracted {
    pub(super) data_change_tick: usize,
    pub(super) uniforms: TileGridSparseUniforms,
    pub(super) texture:  Option<Handle<Image>>,
    pub(super) draw_count: u32,
    pub(super) render_pass: RenderPass2d,
}

#[derive(Debug, Component)]
pub struct TileGridSparseExtactedUpdate {
    pub(super) data:     Option<Box<[u64]>>,
    pub(super) uniforms: Option<TileGridSparseUniforms>,
    pub(super) texture:  Option<Handle<Image>>,
    pub(super) render_pass: Option<RenderPass2d>,
}

pub fn tile_grid_sparse_extract_updates(
    mut commands: Commands,
    q_extracted: Extract<Query<(&RenderEntity, &TileGridSparse, Option<&TileGridAnimator>, &GlobalTransform, &ViewVisibility)>>,
    q_tile_grid_sparse: Query<&TileGridSparseExtracted>,
    r_tex_atlases: Extract<Res<Assets<TileAtlas>>>,
    image_assets:   Res<RenderAssets<GpuImage>>,
) {
    q_extracted.iter().for_each(|(entity, tile_grid_sparse, animator, transform, visibility)| {
        if !visibility.get() { return; }

        let uniforms = TileGridSparseUniforms {
            origin: transform.translation().xy() + tile_grid_sparse.offset() * tile_grid_sparse.scale(),
            depth:  transform.translation().z,
            scale:  tile_grid_sparse.scale(),
            y_depth_scale: tile_grid_sparse.y_depth_scale(),
            frame_time: animator.map_or(0, TileGridAnimator::frame_count),
        };

        let render_pass = tile_grid_sparse.render_pass();
        let texture = tile_grid_sparse.atlas().as_ref()
            .and_then(|h| r_tex_atlases.get(h))
            .map(TileAtlas::image)
            .and_then(|texture| image_assets.get(texture).is_some().then_some(texture)) // TODO HACK how do we update when GPUImage updates
            .cloned();

        let (update_data, update_uniforms, update_texture, update_blend) = if let Ok(dst) = q_tile_grid_sparse.get(entity.entity()) {
            (
                dst.data_change_tick != tile_grid_sparse.data_change_tick(),
                dst.uniforms    != uniforms,
                dst.texture     != texture,
                dst.render_pass != render_pass,
            )
        } else {
            (true, true, true, true)
        };

        if !update_data && !update_uniforms && !update_texture && !update_blend { return; }

        commands
            .entity(entity.entity())
            .insert((
                TileGridSparseExtactedUpdate{
                    data:     update_data.then(|| bytemuck::cast_slice_box(tile_grid_sparse.data_clone())),
                    uniforms: update_uniforms.then_some(uniforms),
                    texture:  if update_texture { texture.clone() } else { None },
                    render_pass: update_blend.then_some(render_pass),
                },
                TileGridSparseExtracted{
                    texture,
                    data_change_tick: tile_grid_sparse.data_change_tick(),
                    uniforms,
                    draw_count: tile_grid_sparse.len() as u32,
                    render_pass,
                }
            ));
    });
}
