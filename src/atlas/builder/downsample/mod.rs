// Copyright 2025 Natalie Baker // Apache License v2 //

mod bilinear_srgb;
pub use bilinear_srgb::*;

pub trait DownsampleAlgorithm {
    fn downsample(
        &self,
        src: &[u8],
        src_size: usize,

        dst: &mut [u8],
    );
}

