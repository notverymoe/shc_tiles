// Copyright 2025 Natalie Baker // Apache License v2 //

use bevy::{camera::visibility::VisibilityClass, prelude::*, render::sync_world::SyncToRenderWorld};

use crate::{atlas::TileAtlas, shared::RenderPass2d};

#[derive(Debug, Clone, Component)]
#[require(GlobalTransform, Visibility, VisibilityClass, SyncToRenderWorld)]
#[component(on_add = bevy::camera::visibility::add_visibility_class::<TileGridDenseInfo>)]
pub struct TileGridDenseInfo {
    size:  UVec2,

    atlas: Option<Handle<TileAtlas>>,

    offset: Vec2,
    scale:  f32,
    y_depth_scale: f32,

    render_pass: RenderPass2d,
}

impl TileGridDenseInfo {

    #[must_use]
    pub const fn new(
        size:  UVec2,
        atlas: Option<Handle<TileAtlas>>,
        offset: Vec2,
        scale: f32,
        y_depth_scale: f32,
        render_pass: RenderPass2d,
    ) -> Self {
        Self {
            size,
            atlas,
            offset,
            scale,
            y_depth_scale,
            render_pass,
        }
    }
}

impl TileGridDenseInfo {

    #[must_use]
    pub const fn size(&self) -> UVec2 {
        self.size
    }

    #[must_use]
    pub const fn atlas(&self) -> &Option<Handle<TileAtlas>> {
        &self.atlas
    }

    pub fn set_atlas(&mut self, atlas: Option<Handle<TileAtlas>>) {
        self.atlas = atlas;
    }

    #[must_use]
    pub const fn offset(&self) -> Vec2 {
        self.offset
    }

    pub const fn set_offset(&mut self, offset: Vec2) {
        self.offset = offset;
    }

    #[must_use]
    pub const fn scale(&self) -> f32 {
        self.scale
    }

    pub const fn set_scale(&mut self, scale: f32) {
        self.scale = scale;
    }

    #[must_use]
    pub const fn y_depth_scale(&self) -> f32 {
        self.y_depth_scale
    }

    pub const fn set_y_depth_scale(&mut self, y_depth_scale: f32) {
        self.y_depth_scale = y_depth_scale;
    }

    #[must_use]
    pub const fn render_pass(&self) -> RenderPass2d {
        self.render_pass
    }

    pub const fn set_render_pass(&mut self, render_pass: RenderPass2d) {
        self.render_pass = render_pass;
    }

}

impl TileGridDenseInfo {

    #[must_use]
    pub fn calculate_bounds(&self) -> [Vec3; 2] {
        let p0 = self.scale*self.offset;
        let p1 = p0 + self.scale*self.size.as_vec2();

        let p0 = p0.extend(0.0);
        let p1 = p1.extend(self.y_depth_scale*(self.size.y as f32));

        [p0.min(p1), p0.max(p1)]
    }


    #[must_use]
    pub fn local_from_world(&self, position: Vec2) -> Option<UVec2> {
        let position = position*self.scale;
        (
            position.x >= 0.0 && 
            position.x < (self.size.x as f32) &&  
            position.y >= 0.0 && 
            position.y < (self.size.y as f32)
        ).then(|| position.as_uvec2())
    }

    #[must_use]
    pub fn world_from_local(&self, position: UVec2) -> Vec2 {
        position.as_vec2()/self.scale
    }

}
