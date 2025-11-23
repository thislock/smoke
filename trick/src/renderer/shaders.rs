use std::{collections::HashMap, fs, path::Path, sync::Arc};
use arc_swap::ArcSwap;
use asset_manager::AssetManager;
use wgpu::util::DeviceExt;

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
    ($device:expr, $surface_config:expr; $( $shader_filename:literal ),+ $(,)?) => {{
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
                    &[],
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
  load_compile_time_shaders!(
    device, surface_config;
    "sample.wgsl",
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
      render_pass.set_pipeline(&pipeline.load().pipeline);
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
    let label = format!("{shader_filename}");

    // Create shader module
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
      label: Some(&format!("{label}_module")),
      source: wgpu::ShaderSource::Wgsl(shader_code.into()),
    });

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
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: Some(&format!("{label}_render_pipeline")),
      layout: Some(&pipeline_layout),
      // vertex shader config
      vertex: wgpu::VertexState {
        module: &module,
        entry_point: Some(vertex_entry),
        buffers: vertex_layouts,
        compilation_options: Default::default(),
      },
      // fragment shader config
      fragment: Some(wgpu::FragmentState {
        module: &module,
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
    });

    Self {
      filename: shader_filename,
      module,
      pipeline,
      layout: pipeline_layout,
      bind_group_layouts,
    }
  }
}
