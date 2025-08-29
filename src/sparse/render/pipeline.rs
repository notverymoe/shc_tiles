// Copyright 2025 Natalie Baker // Apache License v2 //

use core::num::NonZeroU32;

use bevy::{asset::Handle, core_pipeline::core_2d::CORE_2D_DEPTH_FORMAT, ecs::{resource::Resource, system::Commands, world::{FromWorld, World}}, image::BevyDefault, render::{render_resource::{BindGroupLayout, BindGroupLayoutEntry, BindingType, BlendState, BufferBindingType, ColorTargetState, ColorWrites, CompareFunction, DepthBiasState, DepthStencilState, Face, FragmentState, MultisampleState, PrimitiveState, RenderPipelineDescriptor, SamplerBindingType, ShaderStages, SpecializedRenderPipeline, StencilState, TextureFormat, TextureSampleType, TextureViewDimension, VertexState}, renderer::RenderDevice, view::ViewTarget}, shader::Shader, sprite_render::{Mesh2dPipeline, Mesh2dPipelineKey}};

use super::HANDLE_TILE_GRID_SPARSE_SHADER;

#[derive(Debug, Resource)]
pub struct TileGridSparsePipeline {
    pub(super) shader:         Handle<Shader>,
    pub(super) layout_view:    BindGroupLayout,
    pub(super) layout_tilemap: BindGroupLayout,
}

pub fn tile_grid_sparse_pipeline_init(mut commands: Commands) {
    commands.init_resource::<TileGridSparsePipeline>();
}

impl FromWorld for TileGridSparsePipeline {
    fn from_world(render_world: &mut World) -> Self {
        let render_device = render_world.get_resource::<RenderDevice>().unwrap();
        
        Self { 
            shader: HANDLE_TILE_GRID_SPARSE_SHADER,
            layout_view: render_world.resource::<Mesh2dPipeline>().view_layout.clone(),
            layout_tilemap: render_device.create_bind_group_layout(
                "pipeline_tilemap_bind_group", 
                &[
                    BindGroupLayoutEntry{
                        binding: 0,
                        count: NonZeroU32::new(1),
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture{
                            multisampled: false,
                            sample_type: TextureSampleType::Float{ filterable: true },
                            view_dimension: TextureViewDimension::D2Array
                        }
                    },
                    BindGroupLayoutEntry{
                        binding: 1,
                        count: None,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    },

                    BindGroupLayoutEntry{
                        binding: 2,
                        count: None,
                        visibility: ShaderStages::VERTEX_FRAGMENT,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Storage{ read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None
                        }
                    },
                    BindGroupLayoutEntry{
                        binding: 3,
                        count: None,
                        visibility: ShaderStages::VERTEX,
                        ty: BindingType::Buffer{
                            ty: BufferBindingType::Storage{ read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                    },

                ]
            )
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct TileGridSparsePipelineKey {
    pub mesh_key: Mesh2dPipelineKey,
    pub blend:    Option<BlendState>,
}

impl SpecializedRenderPipeline for TileGridSparsePipeline {
    type Key = TileGridSparsePipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        
        let format = if key.mesh_key.contains(Mesh2dPipelineKey::HDR) {
            ViewTarget::TEXTURE_FORMAT_HDR
        } else {
            TextureFormat::bevy_default()
        };

        RenderPipelineDescriptor {
            label: Some("TileGridSparseRenderPipeline".into()),
            layout: vec![
                self.layout_view.clone(),
                self.layout_tilemap.clone()
            ],
            push_constant_ranges: vec![],
            vertex: VertexState {
                shader: self.shader.clone(),
                shader_defs: vec![],
                entry_point: Some("vertex".into()),
                buffers: vec![],
            },
            fragment: Some(FragmentState {
                shader: self.shader.clone(),
                shader_defs: vec![],
                entry_point: Some("fragment".into()),
                targets: vec![Some(ColorTargetState {
                    format,
                    blend: key.blend,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState{
                cull_mode: Some(Face::Back),
                ..PrimitiveState::default()
            },
            depth_stencil: Some(DepthStencilState {
                format: CORE_2D_DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Greater,
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            multisample: MultisampleState {
                count: key.mesh_key.msaa_samples(),
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            zero_initialize_workgroup_memory: false,
        }
    }
}