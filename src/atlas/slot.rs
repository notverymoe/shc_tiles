// Copyright 2025 Natalie Baker // Apache License v2 //

use bytemuck::{Pod, Zeroable};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Pod, Zeroable)]
#[repr(C, align(4))]
pub struct TileAtlasSlot {
    slot: u16,
    anim: u16,
}

impl TileAtlasSlot {

    pub const EMPTY: Self = Self{
        slot: 0,
        anim: 0,
    };

    #[must_use]
    pub fn new(slot: u16) -> Option<Self> {
        Some(Self { 
            slot: slot.checked_add(1)?, 
            anim: 0,
        })
    }

    #[must_use]
    pub const fn new_unchecked(slot: u16) -> Self {
        Self { 
            slot: slot.wrapping_add(1), 
            anim: 0,
        }
    }
    
}

impl TileAtlasSlot {

    #[must_use]
    pub const fn with_animation(
        self,
        frame_count:    u16,
        frame_duration: u16,
        frame_delay:    u16,
    ) -> Self {
        Self {
            slot: self.slot,
            anim:  (frame_count    & 0x000F)
                | ((frame_duration & 0x000F) << 4)
                | ((frame_delay    & 0x000F) << 8)
        }
    }

    #[must_use]
    pub const fn with_frame_count(
        self,
        frame_count: u16
    ) -> Self {
        self.with_animation(frame_count, self.frame_duration(), self.frame_delay())
    }

    #[must_use]
    pub const fn with_frame_duration(
        self,
        frame_duration: u16
    ) -> Self {
        self.with_animation(self.frame_count(), frame_duration, self.frame_delay())
    }

    #[must_use]
    pub const fn with_frame_delay(
        self,
        frame_delay: u16
    ) -> Self {
        self.with_animation(self.frame_count(), self.frame_duration(), frame_delay)
    }
    
}

impl TileAtlasSlot {

    #[must_use]
    pub const fn slot(&self) -> Option<u16> {
        self.slot.checked_sub(1)
    }

    #[must_use]
    pub const fn frame_count(&self) -> u16 {
        self.anim & 0x000F
    }

    #[must_use]
    pub const fn frame_duration(&self) -> u16 {
        (self.anim >> 4) & 0x0003
    }

    #[must_use]
    pub const fn frame_delay(&self) -> u16 {
        (self.anim >> 8) & 0x00FF
    }

}
