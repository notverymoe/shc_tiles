// Copyright 2025 Natalie Baker // Apache License v2 //

use super::DownsampleAlgorithm;

#[derive(Debug, Clone, Copy, Default)]
pub struct DownsampleBilinearSRGB;

impl DownsampleAlgorithm for DownsampleBilinearSRGB {

    fn downsample(
        &self,
        src: &[u8],
        src_size: usize,

        dst: &mut [u8],
    ) {
        assert!(src_size.trailing_zeros() > 0);

        let dst_size = src_size.unbounded_shr(1);

        for y in 0..dst_size {
            for x in 0..dst_size { 
                let s_x = x*2;
                let s_y = y*2;

                let sample = bilinear_interp_srgba([
                    get_rgba(src, (s_x  ) + src_size*(s_y  )),
                    get_rgba(src, (s_x+1) + src_size*(s_y  )),
                    get_rgba(src, (s_x  ) + src_size*(s_y+1)),
                    get_rgba(src, (s_x+1) + src_size*(s_y+1)),
                ]);

                set_rgba(dst, y*dst_size + x, sample);
            }
        }
    }
    
}

fn set_rgba(src: &mut [u8], offset: usize, value: [u8; 4]) {
    src[offset*4  ] = value[0];
    src[offset*4+1] = value[1];
    src[offset*4+2] = value[2];
    src[offset*4+3] = value[3];
}

fn get_rgba(src: &[u8], offset: usize) -> [u8; 4] {
    src[offset*4..offset*4+4].try_into().unwrap()
}

fn bilinear_interp_srgba(samples: [[u8; 4]; 4]) -> [u8; 4] {
    let samples = samples.map(|v| premul_alpha(srgba_to_linear(v.map(u8_to_f32_norm))));
    linear_to_srgba(demul_alpha([
        f32::midpoint(f32::midpoint(samples[0][0], samples[1][0]), f32::midpoint(samples[2][0], samples[3][0])),
        f32::midpoint(f32::midpoint(samples[0][1], samples[1][1]), f32::midpoint(samples[2][1], samples[3][1])),
        f32::midpoint(f32::midpoint(samples[0][2], samples[1][2]), f32::midpoint(samples[2][2], samples[3][2])),
        f32::midpoint(f32::midpoint(samples[0][3], samples[1][3]), f32::midpoint(samples[2][3], samples[3][3])),
    ])).map(f32_norm_to_u8)
}


fn u8_to_f32_norm(v: u8) -> f32 {
    (v as f32)/255.0
}


fn f32_norm_to_u8(v: f32) -> u8 {
    (v*255.0) as u8
}

fn srgba_channel_to_linear(v: f32) -> f32 {
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055)/1.055).powf(2.4)
    }
}

fn srgba_to_linear(v: [f32; 4]) -> [f32; 4] {
    [
        srgba_channel_to_linear(v[0]),
        srgba_channel_to_linear(v[1]),
        srgba_channel_to_linear(v[2]),
        v[3]
    ]
}

fn linear_channel_to_srgb(v: f32) -> f32 {
    if v <= 0.04045 {
        v * 12.92
    } else {
        (v.powf(1.0/2.4)*1.055) - 0.055
    }
}

fn linear_to_srgba(v: [f32; 4]) -> [f32; 4] {
    [
        linear_channel_to_srgb(v[0]),
        linear_channel_to_srgb(v[1]),
        linear_channel_to_srgb(v[2]),
        v[3]
    ]
}


fn premul_alpha(v: [f32; 4]) -> [f32; 4] {
    [
        v[0]*v[3],
        v[1]*v[3],
        v[2]*v[3],
        v[3]
    ]
}

fn demul_alpha(v: [f32; 4]) -> [f32; 4] {
    [
        v[0]/v[3],
        v[1]/v[3],
        v[2]/v[3],
        v[3]
    ]
}
