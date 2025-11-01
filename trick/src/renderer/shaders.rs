use std::{collections::HashMap, fs, path::Path, sync::Arc};
use arc_swap::ArcSwap;
use wgpu::util::DeviceExt;

pub struct PipelineManager {
  device: Arc<wgpu::Device>,
  pipelines: Vec<ArcSwap<ShaderPipeline>>,
  asset_manager: asset_manager::AssetManager,
}

/// Represents one shader + its own render pipeline + its configuration
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

    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some(&format!("{label}_layout")),
      bind_group_layouts: &bind_group_layouts.iter().collect::<Vec<_>>(),
      push_constant_ranges: &[],
    });

    // Create pipeline
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: Some(&format!("{label}_pipeline")),
      layout: Some(&layout),
      vertex: wgpu::VertexState {
        module: &module,
        entry_point: Some(vertex_entry),
        buffers: vertex_layouts,
        compilation_options: todo!(),
      },
      fragment: Some(wgpu::FragmentState {
        module: &module,
        entry_point: Some(fragment_entry),
        targets: &[Some(wgpu::ColorTargetState {
          format: config.format,
          blend: Some(wgpu::BlendState::ALPHA_BLENDING),
          write_mask: wgpu::ColorWrites::ALL,
        })],
        compilation_options: todo!(),
      }),
      primitive: wgpu::PrimitiveState::default(),
      depth_stencil: None,
      multisample: wgpu::MultisampleState::default(),
      multiview: None,
      cache: todo!(),
    });

    Self {
      name: label.to_string(),
      module,
      pipeline,
      layout,
      bind_group_layouts,
    }
  }
}

/// Handles multiple shader pipelines and hot-reload support
pub struct ShaderManager {
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
