// Copyright 2025 Natalie Baker // Apache License v2 //

use bevy::prelude::*;

use crate::{atlas::TileAtlas, shared::RenderPass2d, sparse::TileGridSparse};

#[derive(Debug, Clone)]
pub struct TileGridSparseBuilder {
    offset: Vec2,
    size:   UVec2,
    scale:  f32,
    atlas:  Option<Handle<TileAtlas>>,
    render_pass: RenderPass2d,
    y_depth_scale: f32,
}

impl TileGridSparseBuilder {

    #[must_use]
    pub const fn new(scale: f32) -> Self {
        Self {
            scale,
            offset: Vec2::ZERO,
            size:   UVec2::splat(u8::MAX as u32),
            atlas:  None,
            render_pass: RenderPass2d::Transparent,
            y_depth_scale: 0.0,
        }
    }

    #[must_use]
    pub fn with_atlas(self, atlas: Option<Handle<TileAtlas>>) -> Self {
        Self { atlas, ..self }
    }

    #[must_use]
    pub fn with_offset(self, offset: Vec2) -> Self {
        Self { offset, ..self }
    }

    #[must_use]
    pub fn with_size(self, size: UVec2) -> Self {
        Self { size, ..self }
    }

    #[must_use]
    pub fn with_scale(self, scale: f32) -> Self {
        Self { scale, ..self }
    }

    #[must_use]
    pub fn with_render_pass(self, render_pass: RenderPass2d) -> Self {
        Self { render_pass, ..self }
    }

    #[must_use]
    pub fn with_y_depth_scale(self, y_depth_scale: f32) -> Self {
        Self { y_depth_scale, ..self }
    }

    #[must_use]
    pub fn build(self) -> TileGridSparse {
        TileGridSparse::new(
            self.offset,
            self.size,
            self.scale,
            self.atlas,
            self.render_pass,
            self.y_depth_scale,
        )
    }

    #[must_use]
    pub fn build_with_transform(self, origin: Vec3) -> (Transform, TileGridSparse) {
        self.build_with_transform_xyz(origin.x, origin.y, origin.z)
    }

    #[must_use]
    pub fn build_with_transform_xyz(self, x: f32, y: f32, z: f32) -> (Transform, TileGridSparse) {
        (
            Transform::from_xyz(
                self.scale*x, 
                self.scale*y, 
                z + self.y_depth_scale*self.scale*y
            ),
            self.build()
        )
    }

}
