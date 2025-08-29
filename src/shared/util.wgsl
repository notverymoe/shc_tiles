// Copyright 2025 Natalie Baker // Apache License v2 //

#define_import_path sf_tile_render::util

fn util_decode_u32_to_rgba(data: u32) -> vec4<f32> {
    return vec4<f32>(
        f32(data >> 24 & 0xFF)/255.0,
        f32(data >> 16 & 0xFF)/255.0,
        f32(data >>  8 & 0xFF)/255.0,
        f32(data       & 0xFF)/255.0,
    );
}

fn util_blend_colours(base: vec4<f32>, overlay: vec4<f32>) -> vec4<f32> {
    let base_premul    = base.rgb    * base.a;
    let overlay_premul = overlay.rgb * overlay.a;
    
    let rgb = overlay_premul + (base_premul * (1.0 - overlay.a));
    let a   = overlay.a      + (base.a      * (1.0 - overlay.a));
    return vec4<f32>(rgb/a, a);
}
