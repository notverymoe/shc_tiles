// Copyright 2025 Natalie Baker // Apache License v2 //

use core::num::NonZeroU64;

use bevy::{prelude::*, render::{render_asset::RenderAssets, settings::WgpuLimits, sync_world::RenderEntity, texture::GpuImage, Extract}};
use bytemuck::{Pod, Zeroable};

use crate::{atlas::TileAtlas, dense::{render::{PreparedTileGridDense, TileGridDenseBindGroups}, TileGridDense}, shared::{RenderPass2d, TileGridAnimator}};

#[derive(Debug, Clone, Copy, Zeroable, PartialEq, Pod)]
#[repr(C)]
#[allow(clippy::partial_pub_fields)]
pub struct TileGridDenseUniforms {
    pub size:   UVec2,
    pub origin: Vec2,
    pub scale:  f32,
    pub depth:  f32,
    pub y_depth_scale: f32,
    pub frame_time: u32,
}

impl TileGridDenseUniforms {
    pub const BINDING_SIZE:   usize = core::mem::size_of::<TileGridDenseUniforms>();
    pub const BINDING_SIZE_NZ: NonZeroU64 = NonZeroU64::new(Self::BINDING_SIZE as u64).unwrap();
    pub const BINDING_OFFSET: usize = Self::BINDING_SIZE.next_multiple_of(WgpuLimits::downlevel_defaults().min_storage_buffer_offset_alignment as usize);
}

#[derive(Debug, Component)]
#[require(PreparedTileGridDense, TileGridDenseBindGroups)]
pub struct TileGridDenseExtracted {
    pub(super) data_change_tick: usize,
    pub(super) uniforms: TileGridDenseUniforms,
    pub(super) texture:  Option<Handle<Image>>,
    pub(super) render_pass: RenderPass2d,
}

#[derive(Debug, Component)]
pub struct TileGridDenseExtactedUpdate {
    pub(super) data:     Option<Box<[u32]>>,
    pub(super) uniforms: Option<TileGridDenseUniforms>,
    pub(super) texture:  Option<Handle<Image>>,
    pub(super) render_pass: Option<RenderPass2d>,
} 

pub fn tile_grid_dense_extract_updates(
    mut commands: Commands,
    q_extracted: Extract<Query<(&RenderEntity, &TileGridDense, Option<&TileGridAnimator>, &GlobalTransform, &ViewVisibility)>>,
    q_tile_grid_dense_s: Query<&TileGridDenseExtracted>,
    r_tex_atlases: Extract<Res<Assets<TileAtlas>>>,
    image_assets:   Res<RenderAssets<GpuImage>>,
) {
    q_extracted.iter().for_each(|(entity, tile_grid_dense, animator, transform, visibility)| {
        if !visibility.get() { return; }

        let uniforms = TileGridDenseUniforms {
            origin:         transform.translation().xy() + tile_grid_dense.offset() * tile_grid_dense.scale(),
            depth:          transform.translation().z,
            size:           tile_grid_dense.size(),
            scale:          tile_grid_dense.scale(),
            y_depth_scale:  tile_grid_dense.y_depth_scale(),
            frame_time:     animator.map_or(0, TileGridAnimator::frame_count),
        };

        let render_pass = tile_grid_dense.render_pass();
        let texture = tile_grid_dense.atlas().as_ref()
            .and_then(|h| r_tex_atlases.get(h))
            .map(TileAtlas::image)
            .and_then(|texture| image_assets.get(texture).is_some().then_some(texture)) // TODO HACK how do we update when GPUImage updates
            .cloned();

        let (update_data, update_uniforms, update_texture, update_blend) = if let Ok(dst) = q_tile_grid_dense_s.get(entity.entity()) {
            (
                dst.data_change_tick != tile_grid_dense.data_change_tick(),
                dst.uniforms != uniforms,
                dst.texture  != texture,
                dst.render_pass != render_pass,
            )
        } else {
            (true, true, true, true)
        };

        if !update_data && !update_uniforms && !update_texture && !update_blend { return; }

        commands
            .entity(entity.entity())
            .insert((
                TileGridDenseExtactedUpdate{
                    data:     update_data.then(|| bytemuck::cast_slice_box(tile_grid_dense.data_clone())),
                    uniforms: update_uniforms.then_some(uniforms),
                    texture:  if update_texture { texture.clone() } else { None },
                    render_pass: update_blend.then_some(render_pass),
                },
                TileGridDenseExtracted{
                    texture, 
                    data_change_tick: tile_grid_dense.data_change_tick(),
                    uniforms,
                    render_pass,
                }
            ));
    });
}
