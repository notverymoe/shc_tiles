# SofterHeartsClub Tile Grid for Bevy

The `shc_tile` crate provides tile rendering for the bevy game engine. This
solution does not attempt to provide highly-flexible renderers, instead it
focuses on regular grids of square ratio tiles and assumes that camera has
an orthogonal projection, +Y is up and -Z is forward. It may become more
flexible in the future as needs expand.

## License

This work is Copyright ©2025 by Natalie Baker and is provided to you under
the Apache v2.0 license, please see the included (LICENSE.md)[LICENSE.md] file,
or if it's missing, it can be found online here:
https://www.apache.org/licenses/LICENSE-2.0

## Tile Atlas

This crate provides a custom texture solution. It creates an tile atlas
using a 2D texture array. Each page contains 16x16 tiles with no gap, and
up to 256 pages, making for 2^16 (65536) addressable tile textures. A
limitation of the provided shader is that the 65536-th texture won't be 
accessible as one value is reserved for an empy texture. 

### Tile Atlas Build Queue

Currently the tile atlas queue is provided to help with the construction of
a tile atlas. Simply create an entity with an `TileAtlasBuildQueue` and a
`TileAtlasBuildQueueTarget` contianing a reserved handle for a `TileAtlas`. 

Load images and register them with the queue, when all assets are added,
lock the queue and the plugin will transform the queue into the final
TileAtlas asset. The current implementation doesn't throttle loading the
images and will process them immediately as they become availble. Later
implementations should yield cooperatively.

### Tile Atlas Builder

The queue contains an atlas builder which performs the actual work of
assembling the information to create a new tile atlas. It has an
on-disk representation and can be created from one or more of these
files. In the future, this will be implemented into the queue and
as a native asset.

## Dense Grid

The dense grid provides rendering for a mostly-full grid. It can save
data by encoding the tile index implicitly in the instance index. Updates
involve copying the entire grid's data a few times, so this can be
expensive to update frequently - though the number of tiles updated has
no influence.
 
## Sparse Grid

The sparse grid renders a sequence of tiles at given X-Y tile coordinates. 
This uses considerably more memory, but can sparsely cover a large area
with much less overhead than the dense grid. At present updating the gpu
data isn't optimized.

## Other Features

### Mipmapping

Mipmapping the texture is supported by the builder, it can create
downsampled textures automatically with a call or accept extentally
provided textures. These mipmap levels are also stored in the on-disk
format.

### Dense Texture Storage

Tile textures are stored densely in the texture with no border pixels to
maximise storage efficiency. To prevent pixel bleed, we clamp the sampling
positions to half a texel from the edge. This varies on mipmap level and
requires us to manually select and blend mipmap levels.

### Animations

Animations are supported on the grid using the `TileGridAnimator` component.
Tiles loaded as a sequence will be store sequentially, allowing them to be
used as an animation sequence. The initial frame, number of frames, duration
of each frame and the starting offset can currently be set per tile.

This is still in heavy development and will change.

### Depth

The current implementation allows for the Y-position of a tile's verticies to be
added to the depth of the tile with a scaling factor, implementing the standard
`depth = -y` approach used in 3/4th view RPGs.

This is still in heavy development and will change.

### Limits

The WGPU backend will limit texture sizes to the host machine's capabilities. In
the future it will allow imposing limitations, such as to maximise compatibility.