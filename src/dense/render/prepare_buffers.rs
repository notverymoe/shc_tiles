// Copyright 2025 Natalie Baker // Apache License v2 //

use core::num::NonZero;

use bevy::{prelude::*, render::{render_resource::{encase::private::BufferMut, Buffer, BufferDescriptor, BufferUsages}, renderer::{RenderDevice, RenderQueue}}};

use crate::shared::RenderPass2d;

use super::{TileGridDenseExtactedUpdate, TileGridDenseUniforms, TileGridDenseExtracted};

// TODO PERF render_pass and depth changes shouldnt cause bindgroup recreations, but do
#[derive(Debug, Default, Component)]
pub struct PreparedTileGridDense {
    pub(super) depth: f32,
    pub(super) texture: Option<Handle<Image>>,
    pub(super) buffer: Option<Buffer>,
    pub(super) render_pass: RenderPass2d,
}

pub fn tile_grid_dense_prepare_buffers(
    mut q_tilemaps: Query<
        (&TileGridDenseExtracted, &mut TileGridDenseExtactedUpdate, &mut PreparedTileGridDense),
        Changed<TileGridDenseExtactedUpdate>
    >,
    render_device: Res<RenderDevice>,
    render_queue:  Res<RenderQueue>,
) {
    // PERF we shouldn't mutable deref prepared unless necessary
    q_tilemaps.iter_mut().for_each(|(extracted, mut update, mut prepared)| {

        // // // Ensure Buffer Size // //
        let buffer_size = calculate_buffer_size(extracted.uniforms.size);
        if should_buffer_resize(buffer_size, prepared.buffer.as_ref()) {
            let new_buffer = create_buffer_with_size(&render_device, buffer_size);
            {
                let mut view = new_buffer.slice(..).get_mapped_range_mut();
                let uniforms = update.uniforms.take().unwrap_or(extracted.uniforms);
                view.write_slice(0, bytemuck::bytes_of(&uniforms));
                view.write_slice(TileGridDenseUniforms::BINDING_OFFSET, bytemuck::cast_slice(&update.data.take().unwrap()));
            }
            new_buffer.unmap();
            prepared.buffer = Some(new_buffer);
        }

        // // Update Buffer Data // //
        if let Some(data) = update.data.take() {
            let uniforms = update.uniforms.take().unwrap_or(extracted.uniforms);
            let mut view = render_queue.write_buffer_with(prepared.buffer.as_ref().unwrap(), 0, NonZero::new(buffer_size).unwrap()).unwrap();
            view.write_slice(0, bytemuck::bytes_of(&uniforms));
            view.write_slice(TileGridDenseUniforms::BINDING_OFFSET, bytemuck::cast_slice(&data));
        } else if update.uniforms.is_some() {
            let uniforms = update.uniforms.take().unwrap();
            render_queue.write_buffer(prepared.buffer.as_ref().unwrap(), 0, bytemuck::bytes_of(&uniforms));
        } 
        
        // // Update Texture Data // //
        if let Some(texture_update) = update.texture.take() {
            prepared.texture = Some(texture_update);
        }

        // // Update Alpha Blend // //
        if let Some(render_pass_update) = update.render_pass.take() {
            prepared.render_pass = render_pass_update;
        }
    });

}

const fn calculate_buffer_size(data_len_hint: UVec2) -> u64 {
    let data_len_hint = (core::mem::size_of::<u32>() as u64) * (data_len_hint.x as u64) * (data_len_hint.y as u64);
    TileGridDenseUniforms::BINDING_OFFSET as u64 + data_len_hint
}

#[must_use]
fn should_buffer_resize(size: u64, buffer: Option<&Buffer>) -> bool {
    if let Some(buffer) = buffer {
        buffer.size() < size
    } else {
        true
    }
}

fn create_buffer_with_size(render_device: &RenderDevice, size: u64) -> Buffer {
    render_device.create_buffer(&BufferDescriptor { 
        label: Some("TileGridDense Buffer"), 
        size,  
        usage: BufferUsages::COPY_DST | BufferUsages::STORAGE, 
        mapped_at_creation: true 
    })
}
