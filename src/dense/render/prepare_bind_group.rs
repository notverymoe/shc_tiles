// Copyright 2025 Natalie Baker // Apache License v2 //

use core::num::NonZeroU64;

use bevy::{prelude::*, render::{render_asset::RenderAssets, render_resource::{BindGroup, BindGroupEntry, BindingResource, BufferBinding}, renderer::RenderDevice, texture::GpuImage}};

use super::{TileGridDensePipeline, PreparedTileGridDense, TileGridDenseUniforms};

#[derive(Debug, Default, Component)]
pub struct TileGridDenseBindGroups {
    pub(super) bind_group: Option<BindGroup>,
}

pub fn tile_grid_dense_prepare_bind_groups(
    mut q_tilemaps: Query<(&PreparedTileGridDense, &mut TileGridDenseBindGroups), Changed<PreparedTileGridDense>>,
    pipeline:       Res<TileGridDensePipeline>,
    image_assets:   Res<RenderAssets<GpuImage>>,
    render_device:  Res<RenderDevice>,
) {
    q_tilemaps.iter_mut().for_each(|(PreparedTileGridDense{texture, buffer, depth: _, render_pass: _}, mut bindings)| {
        if let Some(buffer) = &buffer && let Some(gpu_image) = texture.as_ref().and_then(|texture| image_assets.get(texture)) {
            bindings.bind_group = Some(render_device.create_bind_group(
                "layer_material_bind_group", 
                &pipeline.layout_tilemap,
                &[
                    BindGroupEntry{
                        binding: 0,
                        resource: BindingResource::TextureView(&gpu_image.texture_view),
                    },
                    BindGroupEntry{
                        binding: 1,
                        resource: BindingResource::Sampler(&gpu_image.sampler),
                    },
                    BindGroupEntry{
                        binding: 2,
                        resource: BindingResource::Buffer(BufferBinding {
                            buffer,
                            offset: 0,
                            size: NonZeroU64::new(TileGridDenseUniforms::BINDING_SIZE as u64),
                        })
                    },
                    BindGroupEntry{
                        binding: 3,
                        resource: BindingResource::Buffer(BufferBinding {
                            buffer,
                            offset: TileGridDenseUniforms::BINDING_OFFSET as u64,
                            size: None,
                        })
                    }
                ]
            ));
        } else {
            bindings.bind_group = None;
        }

    });
}