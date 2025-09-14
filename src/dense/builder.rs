// Copyright 2025 Natalie Baker // Apache License v2 //

use bevy::prelude::*;

use crate::{atlas::{TileAtlas, TileAtlasSlot}, dense::{TileGridDenseData, TileGridDenseInfo}, shared::RenderPass2d};

#[derive(Debug, Clone)]
pub struct TileGridDenseBuilder {
    offset: Vec2,
    size:  UVec2,
    scale: f32,
    atlas: Option<Handle<TileAtlas>>,
    filled: TileAtlasSlot,
    render_pass: RenderPass2d,
    y_depth_scale: f32,
}

impl TileGridDenseBuilder {

    #[must_use]
    pub const fn new(size: UVec2, scale: f32) -> Self {
        Self {
            size,
            scale,
            offset: Vec2::ZERO,
            atlas: None,
            filled: TileAtlasSlot::EMPTY,
            render_pass: RenderPass2d::Opaque,
            y_depth_scale: 0.0,
        }
    }

    #[must_use]
    pub fn with_offset(self, offset: Vec2) -> Self {
        Self { offset, ..self }
    }

    #[must_use]
    pub fn with_atlas(self, atlas: Option<Handle<TileAtlas>>) -> Self {
        Self { atlas, ..self }
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
    pub fn with_fill(self, filled: impl Into<TileAtlasSlot>) -> Self {
        Self { filled: filled.into(), ..self }
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
    pub fn build(self) -> (TileGridDenseData, TileGridDenseInfo) {
        (
            TileGridDenseData::new(
                self.size,
                self.filled,
            ),
            TileGridDenseInfo::new(
                self.size,
                self.atlas,
                self.offset,
                self.scale,
                self.y_depth_scale,
                self.render_pass,
            )
        )
    }

    #[must_use]
    pub fn build_with_transform(self, origin: Vec3) -> (Transform, (TileGridDenseData, TileGridDenseInfo)) {
        self.build_with_transform_xyz(origin.x, origin.y, origin.z)
    }

    #[must_use]
    pub fn build_with_transform_xyz(self, x: f32, y: f32, z: f32) -> (Transform, (TileGridDenseData, TileGridDenseInfo)) {
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
