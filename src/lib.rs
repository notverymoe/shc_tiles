// Copyright 2025 Natalie Baker // Apache License v2 //

pub mod atlas;
pub mod dense;
pub mod shared;
pub mod sparse;
pub mod plugin;

pub mod prelude {
    pub use super::dense::TileGridDenseBuilder;
    pub use super::dense::TileGridDense;
    
    pub use super::sparse::TileGridSparseBuilder;
    pub use super::sparse::TileGridSparse;

    pub use super::atlas::TileAtlas;
    pub use super::atlas::TileAtlasSlot;
    pub use super::atlas::TileAtlasGroup;
    pub use super::atlas::TileAtlasEntry;

    pub use super::atlas::builder::TileAtlasBuilder;
    pub use super::atlas::builder::TileAtlasBuilderGroup;
    pub use super::atlas::builder::TileAtlasBuilderSet;

    pub use super::atlas::builder::TileSetSettings;
    
    pub use super::atlas::builder::TileAtlasBuilderImageSequence;
    pub use super::atlas::builder::TileAtlasBuildQueue;
    pub use super::atlas::builder::TileAtlasBuildQueueTarget;

    pub use super::atlas::builder::DownsampleBilinearSRGB;

    pub use super::shared::TileGridAnimator;

    pub use super::plugin::PluginsTileRender;
}