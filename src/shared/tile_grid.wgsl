// Copyright 2025 Natalie Baker // Apache License v2 //

#define_import_path sf_tile_render::tile_grid

#import bevy_render::view::View
#import sf_tile_render::atlas::atlas_sample_texture

// // Passes // //

struct TileGridVertexOutput {
    position_world: vec3<f32>,
    position_clip:  vec4<f32>,
    uv: vec2<f32>,
    slot: u32,
}

fn tile_grid_vertex(
    view: View,

    vertex_index: u32,

    origin:  vec2<f32>,
    scale:   f32,
    depth:   f32,
    depth_y_scale: f32,
    frame_time: u32,

    tile:    u32,
    tile_local: vec2<u32>,

) -> TileGridVertexOutput {
    let tile_uv = tile_grid_uv(vertex_index, tile);
    let position_world = tile_grid_world_vertex_position(
        origin,
        scale,
        vec2<f32>(tile_local),
        tile_uv.uv_base,
        depth
    );
    let depth_adj          = vec3<f32>(0, 0, -depth_y_scale * scale * (f32(tile_local.y) + tile_uv.uv_base.y));
    let position_world_adj = position_world + depth_adj;
    let position_clip      = view.clip_from_world * vec4<f32>(position_world_adj, 1.0);

    let atlas_slot = tile_grid_atlas_slot(tile);
    let slot_offset = tile_grid_animation_slot_offset(frame_time, tile >> 16);

    var out: TileGridVertexOutput;
    out.position_world = position_world_adj;
    out.position_clip  = position_clip;
    out.slot           = atlas_slot + slot_offset;
    out.uv             = tile_uv.uv;
    return out;
}

fn tile_grid_fragment(
    t: texture_2d_array<f32>,
    s: sampler,
    slot: u32,
    uv: vec2<f32>
) -> vec4<f32> {
    return select(
        atlas_sample_texture(t, s, slot - 1, uv), 
        vec4(0.0, 0.0, 0.0, 0.0), 
        slot == 0
    );
}

// // Shared // //

fn tile_grid_atlas_slot(data: u32) -> u32 {
    return data & 0x0000FFFF;
}

fn tile_grid_animation_slot_offset(time: u32, tile: u32) -> u32 {
    let anim = tile_grid_animation(tile);
    return (time + anim.delay)/max(anim.duration, 1) % max(anim.count, 1);
}

// // Sparse // //

fn tile_grid_sparse_position(tile: u32) -> vec2<u32> {
    return vec2<u32>(
         tile        & 0x0000FFFF, 
        (tile >> 16) & 0x0000FFFF
    );
}

// // Dense // //

fn tile_grid_dense_position(
    instance_index: u32,
    size: vec2<u32>,
) -> vec2<u32> {
    return vec2<u32>( 
        instance_index % size.x,
        instance_index / size.x
    );
}

fn tile_grid_dense_index_u32(tile_pos: vec2<u32>, size: vec2<u32>) -> u32 {
    return tile_pos.x + tile_pos.y*size.x;
}

fn tile_grid_dense_index_u16(tile_pos: vec2<u32>, size: vec2<u32>) -> vec2<u32> {
    let idx = tile_pos.x + tile_pos.y*size.x;
    return vec2<u32>(idx/2, (idx % 2)*16);
}

// // Internal // //

fn tile_grid_world_vertex_position(
    grid_origin:  vec2<f32>,
    tile_scale:   f32,
    tile_pos:     vec2<f32>,
    tile_uv_base: vec2<f32>,
    tile_depth:   f32,
) -> vec3<f32> {
    let tile_pos_local = tile_scale * (tile_uv_base + tile_pos);
    let world_pos      = grid_origin + tile_pos_local;
    return vec3<f32>(world_pos, tile_depth);
}

struct TileGridAnim {
    count:    u32,
    duration: u32,
    delay:    u32,
}

fn tile_grid_animation(
    tile: u32,
) -> TileGridAnim {
    var out: TileGridAnim;
    out.count    =  tile       & 0x000F; 
    out.duration = (tile >> 4) & 0x000F; 
    out.delay    = (tile >> 8) & 0x00FF; 
    return out;
}

struct TileGridUV {
    uv_base: vec2<f32>,
    uv:      vec2<f32>,
}

fn tile_grid_uv(
    vertex_index: u32,
    data: u32,
) -> TileGridUV {
    var out: TileGridUV;

    let uv_bases = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 1.0),

        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 1.0),
    );
    out.uv_base = uv_bases[vertex_index];
    out.uv = vec2<f32>(out.uv_base.x, 1.0 - out.uv_base.y);

    return out;
}
