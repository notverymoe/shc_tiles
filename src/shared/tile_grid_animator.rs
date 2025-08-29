// Copyright 2025 Natalie Baker // Apache License v2 //

use bevy::prelude::*;

#[derive(Debug, Clone, Copy, Component)]
pub struct TileGridAnimator {
    count:      u32,
    accum:      f64,
    frames_per_seconds: f64,
}

impl TileGridAnimator {

    #[must_use]
    pub const fn new(
        count:      u32,
        accum:      f64,
        time_scale: f64,
    ) -> Self {
        Self { count, accum, frames_per_seconds: time_scale}
    }

    pub const fn accumulate(&mut self, delta: f64) {
        self.accum += delta * self.frames_per_seconds;
        self.count += self.accum as u32; 
        self.accum = self.accum.fract();
    }

    #[must_use]
    pub const fn frame_count(&self) -> u32 {
        self.count
    }

}

pub fn update_tile_grid_animator(
    mut q_tile_grid_animator: Query<&mut TileGridAnimator>,
    r_time: Res<Time>,
) {
    q_tile_grid_animator.iter_mut().for_each(|mut animator| {
        animator.accumulate(r_time.delta_secs_f64());
    });
}