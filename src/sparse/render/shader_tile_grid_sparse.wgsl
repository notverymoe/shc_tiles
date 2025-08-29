// Copyright 2025 Natalie Baker // Apache License v2 //

#import bevy_render::view::View
#import sf_tile_render::tile_grid::{
    tile_grid_sparse_position,
    tile_grid_vertex,
    tile_grid_animation_slot_offset,
    tile_grid_atlas_slot,
    tile_grid_fragment,
}

@group(0) @binding(0) var<uniform> view: View;

@group(1) @binding(0) var atlas_textures: texture_2d_array<f32>;
@group(1) @binding(1) var atlas_sampler:  sampler;

@group(1) @binding(2) var<storage, read> tile_grid_uniforms: TileGridSparseUniforms;
@group(1) @binding(3) var<storage, read> tile_grid_data:     array<u64>;

struct TileGridSparseUniforms {
    origin: vec2<f32>,
    scale:  f32,
    depth:  f32,
    y_depth_scale: f32,
    frame_time: u32,
}

struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) @interpolate(flat) slot: u32,
    @location(1) uv: vec2<f32>,
};

struct VertexInput {
    @builtin(instance_index) instance_index: u32,
    @builtin(  vertex_index) vertex_index:   u32,
}

// // /////////////////////// // //
// // //// VERTEX SHADER //// // //
// // /////////////////////// // //

@vertex
fn vertex(in: VertexInput) -> VertexOutput {
    // // Determine tile data & position // //
    let tile_raw   = tile_grid_data[in.instance_index];
    let tile       = u32(tile_raw);
    let tile_local = tile_grid_sparse_position(u32(tile_raw >> 32));

    // // Calculate Vertex Data // //
    let tile_vertex = tile_grid_vertex(
        view,
        in.vertex_index,
        tile_grid_uniforms.origin,
        tile_grid_uniforms.scale,
        tile_grid_uniforms.depth,
        tile_grid_uniforms.y_depth_scale,
        tile_grid_uniforms.frame_time,
        tile,
        tile_local
    );

    // // Write output // //
    var out: VertexOutput;
    out.clip_pos = tile_vertex.position_clip;
    out.slot     = tile_vertex.slot;
    out.uv       = tile_vertex.uv;
    return out;
}

// // //////////////////////// // //
// // //// FRAMENT SHADER //// // //
// // //////////////////////// // //

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return tile_grid_fragment(atlas_textures, atlas_sampler, in.slot, in.uv);
}
