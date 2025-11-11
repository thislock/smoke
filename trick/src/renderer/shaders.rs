use std::{collections::HashMap, fs, path::Path, sync::Arc};
use arc_swap::ArcSwap;
use asset_manager::AssetManager;
use sdl3::render;
use wgpu::util::DeviceExt;

pub struct PipelineManager {
  device: Arc<wgpu::Device>,
  pipelines: Vec<ArcSwap<ShaderPipeline>>,
  asset_manager: asset_manager::AssetManager,
}

fn load_integrated_pipelines(
  device: &wgpu::Device,
  surface_config: &wgpu::SurfaceConfiguration,
) -> Vec<ArcSwap<ShaderPipeline>> {
  let mut shaders = vec![];

  shaders.push(ArcSwap::new(ShaderPipeline::new(
    device,
    surface_config,
    include_str!("./shaders/sample.wgsl"),
    "vs_main",
    "fs_main",
    &[],
  ).into()));

  shaders
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
  pub name: String,
  pub module: wgpu::ShaderModule,
  pub pipeline: wgpu::RenderPipeline,
  pub layout: wgpu::PipelineLayout,
  pub bind_group_layouts: Vec<wgpu::BindGroupLayout>,
}

impl ShaderPipeline {
  pub fn new(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
    shader_code: &str,
    vertex_entry: &str,
    fragment_entry: &str,
    vertex_layouts: &[wgpu::VertexBufferLayout<'_>],
  ) -> Self {
    let label = "PLACEHOLDER_LABEL";

    // Create shader module
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
      label: Some(&format!("{label}_module")),
      source: wgpu::ShaderSource::Wgsl(shader_code.into()),
    });

    // Example: create a simple layout
    let bind_group_layouts =
      vec![
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
          label: Some(&format!("{label}_bgl")),
          entries: &[],
        }),
      ];

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some(&format!("{label}_layout")),
      bind_group_layouts: &[],
      push_constant_ranges: &[],
    });

    // Create pipeline
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: Some(&format!("{label}_pipeline")),
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
      name: label.to_string(),
      module,
      pipeline,
      layout: pipeline_layout,
      bind_group_layouts,
    }
  }
}

/// forgot what this was for
struct ShaderManager {
  pub pipelines: HashMap<String, Arc<ShaderPipeline>>,
}

impl ShaderManager {
  pub fn new() -> Self {
    Self {
      pipelines: HashMap::new(),
    }
  }

  pub fn load_shader(
    &mut self,
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
    shader_code: &str,
    vertex_entry: &str,
    fragment_entry: &str,
    vertex_layouts: &[wgpu::VertexBufferLayout<'_>],
  ) {
    let name = "PLACEHOLDER_NAME";

    let pipeline = ShaderPipeline::new(
      device,
      config,
      shader_code,
      vertex_entry,
      fragment_entry,
      vertex_layouts,
    );
    self.pipelines.insert(name.to_string(), Arc::new(pipeline));
  }

  pub fn get(&self, name: &str) -> Option<Arc<ShaderPipeline>> {
    self.pipelines.get(name).cloned()
  }

  /// Example recompile method for hot reload
  pub fn reload_shader(
    &mut self,
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
    name: &str,
    shader_code: &str,
    vertex_entry: &str,
    fragment_entry: &str,
    vertex_layouts: &[wgpu::VertexBufferLayout<'_>],
  ) {
    println!("Reloading shader: {name}");
    self.load_shader(
      device,
      config,
      shader_code,
      vertex_entry,
      fragment_entry,
      vertex_layouts,
    );
  }
}
