use std::{collections::HashMap, fs, path::Path, sync::Arc};
use arc_swap::ArcSwap;
use asset_manager::AssetManager;
use wgpu::util::DeviceExt;

trait WgpuVertex {
  const VERTEX_BUFFER_LAYOUT: wgpu::VertexBufferLayout<'static>;
}

// lib.rs
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct ColoredVertex {
  position: [f32; 3],
  color: [f32; 3],
}

impl WgpuVertex for ColoredVertex {
  const VERTEX_BUFFER_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
      array_stride: std::mem::size_of::<ColoredVertex>() as wgpu::BufferAddress, // 1.
      step_mode: wgpu::VertexStepMode::Vertex,                            // 2.
      attributes: &[
        // 3.
        wgpu::VertexAttribute {
          offset: 0,                             // 4.
          shader_location: 0,                    // 5.
          format: wgpu::VertexFormat::Float32x3, // 6.
        },
        wgpu::VertexAttribute {
          offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
          shader_location: 1,
          format: wgpu::VertexFormat::Float32x3,
        },
      ],
    };
}

const STATIC_TEST_MODEL: &[ColoredVertex] = &[
  ColoredVertex {
    position: [0.0, 0.5, 0.0],
    color: [1.0, 0.0, 0.0],
  },
  ColoredVertex {
    position: [-0.5, -0.5, 0.0],
    color: [0.0, 1.0, 0.0],
  },
  ColoredVertex {
    position: [0.5, -0.5, 0.0],
    color: [0.0, 0.0, 1.0],
  },
];

pub struct PipelineManager {
  device: Arc<wgpu::Device>,
  pipelines: Vec<ArcSwap<ShaderPipeline>>,
  asset_manager: asset_manager::AssetManager,
}

/// Usage:
/// let shaders = add_known_shader!(device, config; "a.wgsl", "b.wgsl");
macro_rules! load_compile_time_shaders {
    // im sorry if this is terrible, but macros are completely insane in the way they're written,
    // so i just cheated with chatgpt so i didn't have to learn the forbiden arts
    ($vertex_layouts:expr, $device:expr, $surface_config:expr; $( $shader_filename:literal ),+ $(,)?) => {{
        let mut v: Vec<arc_swap::ArcSwapAny<std::sync::Arc<crate::renderer::shaders::ShaderPipeline>>> = Vec::new();

        $(
            v.push(arc_swap::ArcSwapAny::new(std::sync::Arc::new(
                crate::renderer::shaders::ShaderPipeline::new(
                    $shader_filename,
                    $device,
                    $surface_config,
                    include_str!(concat!("shaders", "/", $shader_filename)),
                    "vs_main",
                    "fs_main",
                    $vertex_layouts,
                ),
            )));
        )*

        v
    }};
}

fn load_integrated_pipelines(
  device: &wgpu::Device,
  surface_config: &wgpu::SurfaceConfiguration,
) -> Vec<ArcSwap<ShaderPipeline>> {

  let colored_vertex_desc = ColoredVertex::VERTEX_BUFFER_LAYOUT;

  load_compile_time_shaders!(
    &[colored_vertex_desc], device, surface_config;
    "colored_vertex.wgsl",
  )
}

impl PipelineManager {
  pub fn new(device: Arc<wgpu::Device>, surface_config: &wgpu::SurfaceConfiguration) -> Self {
    let asset_manager = AssetManager::new_local_filesystem();

    Self {
      device: device.clone(),
      pipelines: load_integrated_pipelines(&device, surface_config),
      asset_manager,
    }
  }

  pub fn render_all(&mut self, render_pass: &mut wgpu::RenderPass) -> anyhow::Result<()> {
    for pipeline in self.pipelines.iter() {
      let pipeline = pipeline.load();
      render_pass.set_pipeline(&pipeline.pipeline);
      render_pass.set_vertex_buffer(0, pipeline.vertex_buffer.slice(..));
      render_pass.draw(0..3, 0..1);
    }

    Ok(())
  }
}

/// Represents one shader + its own render pipeline + its configuration
#[derive(Clone)]
pub struct ShaderPipeline {
  pub filename: &'static str,
  pub module: wgpu::ShaderModule,
  pub pipeline: wgpu::RenderPipeline,
  pub layout: wgpu::PipelineLayout,
  pub bind_group_layouts: Vec<wgpu::BindGroupLayout>,
  pub vertex_buffer: wgpu::Buffer,
}

impl ShaderPipeline {
  pub fn new(
    shader_filename: &'static str,
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
    shader_code: &str,
    vertex_entry: &str,
    fragment_entry: &str,
    vertex_layouts: &[wgpu::VertexBufferLayout<'_>],
  ) -> Self {
    let label: String = format!("{shader_filename}");

    // Create shader module
    let module = init_shader_module(device, shader_code, &label);

    // Example: create a simple layout
    let bind_group_layouts =
      vec![
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
          label: Some(&format!("{label}_bind_group_layout")),
          entries: &[],
        }),
      ];

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some(&format!("{label}_pipeline_layout")),
      bind_group_layouts: &[],
      push_constant_ranges: &[],
    });

    // Create pipeline
    let pipeline = create_render_pipeline(
      device,
      config,
      vertex_entry,
      fragment_entry,
      vertex_layouts,
      &module,
      &pipeline_layout,
      &label,
    );

    let vertex_buffer = init_vertex_buffer(device);

    Self {
      vertex_buffer,
      filename: shader_filename,
      module,
      pipeline,
      layout: pipeline_layout,
      bind_group_layouts,
    }
  }
}

fn init_vertex_buffer(device: &wgpu::Device) -> wgpu::Buffer {
  device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
    label: Some("Vertex Buffer"),
    contents: bytemuck::cast_slice(STATIC_TEST_MODEL),
    usage: wgpu::BufferUsages::VERTEX,
  })
}

fn create_render_pipeline(
  device: &wgpu::Device,
  config: &wgpu::SurfaceConfiguration,
  vertex_entry: &str,
  fragment_entry: &str,
  vertex_layouts: &[wgpu::VertexBufferLayout<'_>],
  module: &wgpu::ShaderModule,
  pipeline_layout: &wgpu::PipelineLayout,
  label: &str,
) -> wgpu::RenderPipeline {
  device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
    label: Some(&format!("{label}_render_pipeline")),
    layout: Some(pipeline_layout),
    // vertex shader config
    vertex: wgpu::VertexState {
      module: module,
      entry_point: Some(vertex_entry),
      buffers: vertex_layouts,
      compilation_options: Default::default(),
    },
    // fragment shader config
    fragment: Some(wgpu::FragmentState {
      module: module,
      entry_point: Some(fragment_entry),
      targets: &[Some(wgpu::ColorTargetState {
        format: config.format,
        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
        write_mask: wgpu::ColorWrites::ALL,
      })],
      compilation_options: Default::default(),
    }),
    // geometry config
    primitive: wgpu::PrimitiveState {
      topology: wgpu::PrimitiveTopology::TriangleList, // 1.
      strip_index_format: None,
      front_face: wgpu::FrontFace::Ccw, // 2.
      cull_mode: Some(wgpu::Face::Back),
      // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
      polygon_mode: wgpu::PolygonMode::Fill,
      // Requires Features::DEPTH_CLIP_CONTROL
      unclipped_depth: false,
      // Requires Features::CONSERVATIVE_RASTERIZATION
      conservative: false,
    },
    depth_stencil: None,
    multisample: wgpu::MultisampleState::default(),
    multiview: None,
    // add this later
    cache: None,
  })
}

fn init_shader_module(device: &wgpu::Device, shader_code: &str, label: &str) -> wgpu::ShaderModule {
  let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
    label: Some(&format!("{label}_module")),
    source: wgpu::ShaderSource::Wgsl(shader_code.into()),
  });
  module
}
