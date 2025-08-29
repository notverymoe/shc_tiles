// Copyright 2025 Natalie Baker // Apache License v2 //

#define_import_path sf_tile_render::atlas

const ATLAS_TILE_COUNT_XY_U: u32 = 16;
const ATLAS_TILE_COUNT_XY_F: f32 = 16;

fn atlas_get_tile_size(t: texture_2d_array<f32>) -> f32 {
    return f32(textureDimensions(t).x)/ATLAS_TILE_COUNT_XY_F;
}

fn atlas_get_uv_pixel_size(t: texture_2d_array<f32>, level: u32) -> f32 {
    return 1.0/f32(textureDimensions(t, level).x);
}

fn atlas_calculate_page_uv(slot: vec2<u32>, uv: vec2<f32>, uv_pixel: f32) -> vec2<f32> {
    let scale  = 1.0/ATLAS_TILE_COUNT_XY_F;
    var uv_min = 0.5*uv_pixel;
    var uv_max = 1.0/ATLAS_TILE_COUNT_XY_F - 0.5*uv_pixel;
    var uv_scaled_clamped = clamp(uv * scale, vec2<f32>(uv_min, uv_min), vec2<f32>(uv_max, uv_max));
    return uv_scaled_clamped + vec2<f32>(slot)*scale;
}

fn atlas_get_mipmap_level(t: texture_2d_array<f32>, uv: vec2<f32>) -> f32 {
    var tile_size = atlas_get_tile_size(t);
    var max_level = f32(textureNumLevels(t) - 1);

    var dx_vtc = dpdxFine(uv*tile_size);
    var dy_vtc = dpdyFine(uv*tile_size);
    var delta_max_sqr = max(dot(dx_vtc, dx_vtc), dot(dy_vtc, dy_vtc));

    return min(
        0.5 * log2(delta_max_sqr),
        max_level
    );
}

fn atlas_sample_texture(
    t: texture_2d_array<f32>, 
    s: sampler, 
    slot: u32, 
    uv: vec2<f32>,
) -> vec4<f32> {

    let slot_xy   = vec2<u32>(slot & 0x0F, (slot >> 4) & 0x0F);
    let slot_page = (slot >> 8) & 0xFF;

    // Limit the mipmap level to the texture's max level
    var level = atlas_get_mipmap_level(t, uv);

    // We sample the lower and upper mipmap levels seperately and interpolate ourselves
    //   this is required to prevent texture bleed as the half-texel offset is per-mip-level
    var level_low  = floor(level);
    var level_high = ceil(level);
    var level_mix  = level - f32(level_low);

    var page_uv_low  = atlas_calculate_page_uv(slot_xy, uv, atlas_get_uv_pixel_size(t, u32(level_low )));
    var page_uv_high = atlas_calculate_page_uv(slot_xy, uv, atlas_get_uv_pixel_size(t, u32(level_high)));

    var sample_low  = textureSampleLevel(t, s, page_uv_low,  slot_page, level_low );
    var sample_high = textureSampleLevel(t, s, page_uv_high, slot_page, level_high);

    return mix(sample_low, sample_high, level_mix);
}