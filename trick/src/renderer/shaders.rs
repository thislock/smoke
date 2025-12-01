use std::{collections::HashMap, fs, path::Path, sync::Arc};
use asset_manager::AssetManager;
use std::sync::RwLock;
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
    step_mode: wgpu::VertexStepMode::Vertex,                                   // 2.
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

#[derive(Clone)]
pub struct Model {
  vertexes: Arc<[ColoredVertex]>,
  indicies: Arc<[u16]>,
}

impl Model {
  pub fn new(vertexes: &[ColoredVertex], indicies: &[u16]) -> Self {
    Self {
      vertexes: Arc::from(vertexes),
      indicies: Arc::from(indicies),
    }
  }
}

const STATIC_TEST_MODEL: &[ColoredVertex] = &[
  ColoredVertex {
    position: [-0.0868241, 0.49240386, 0.0],
    color: [0.5, 0.0, 0.5],
  }, // A
  ColoredVertex {
    position: [-0.49513406, 0.06958647, 0.0],
    color: [0.5, 0.0, 0.5],
  }, // B
  ColoredVertex {
    position: [-0.21918549, -0.44939706, 0.0],
    color: [0.5, 0.0, 0.5],
  }, // C
  ColoredVertex {
    position: [0.35966998, -0.3473291, 0.0],
    color: [0.5, 0.0, 0.5],
  }, // D
  ColoredVertex {
    position: [0.44147372, 0.2347359, 0.0],
    color: [0.5, 0.0, 0.5],
  }, // E
];

const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];

/// Usage:
/// let shaders = add_known_shader!(device, config; "a.wgsl", "b.wgsl");
macro_rules! load_compile_time_shaders {
    // im sorry if this is terrible, but macros are completely insane in the way they're written,
    // so i just cheated with chatgpt so i didn't have to learn the forbiden arts
    ($vertex_layouts:expr, $device:expr, $surface_config:expr; $( $shader_filename:literal ),+ $(,)?) => {{
        let mut v: Vec<RwLock<crate::renderer::shaders::ShaderPipeline>> = Vec::new();

        $(
            v.push(RwLock::new(
                crate::renderer::shaders::ShaderPipeline::new(
                    $shader_filename,
                    $device,
                    $surface_config,
                    include_str!(concat!("shaders", "/", $shader_filename)),
                    "vs_main",
                    "fs_main",
                    $vertex_layouts,
                ),
            ));
        )*

        v
    }};
}
pub struct PipelineManager {
  device: Arc<wgpu::Device>,
  pipelines: Vec<RwLock<ShaderPipeline>>,
  asset_manager: asset_manager::AssetManager,
}

fn load_integrated_pipelines(
  device: &wgpu::Device,
  surface_config: &wgpu::SurfaceConfiguration,
) -> Vec<RwLock<ShaderPipeline>> {
  let colored_vertex_desc = ColoredVertex::VERTEX_BUFFER_LAYOUT;

  load_compile_time_shaders!(
    &[colored_vertex_desc], device, surface_config;
    "colored_vertex.wgsl",
  )
}

impl PipelineManager {
  pub fn new(device: Arc<wgpu::Device>, surface_config: &wgpu::SurfaceConfiguration) -> Self {
    let asset_manager = AssetManager::new_local_filesystem();

    let test_model = Model::new(STATIC_TEST_MODEL, INDICES);

    let pipelines = load_integrated_pipelines(&device, surface_config);
    for pipeline in &pipelines {
      let mut write_pipeline = pipeline.write().unwrap();

      let geometry = GeometryBuffer::new(&device, test_model.clone());
      write_pipeline.geometry.push(RwLock::new(geometry));
    }

    Self {
      device: device.clone(),
      pipelines,
      asset_manager,
    }
  }

  pub fn render_all(&mut self, render_pass: &mut wgpu::RenderPass) -> anyhow::Result<()> {
    for pipeline in self.pipelines.iter() {
      let pipeline = pipeline.read().expect("PIPELINE UNWRAP OVERLAP");
      render_pass.set_pipeline(&*&pipeline.pipeline);
      for pipeline_geometry in (*pipeline.geometry).iter() {
        // rendering isn't essential, the program wont go down because i need to render something lol
        // might cause some random flickering though, but not that big a deal.
        if let Ok(read_geometery) = pipeline_geometry.read() {
          read_geometery.render_with_current_pipeline(render_pass);
        }
      }
    }

    Ok(())
  }
}

pub struct GeometryBuffer {
  pub vertex_buffer: wgpu::Buffer,
  pub index_buffer: wgpu::Buffer,
  pub model: Model,
}

impl GeometryBuffer {
  fn new(device: &wgpu::Device, model: Model) -> Self {
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("Vertex Buffer"),
      contents: bytemuck::cast_slice(&model.vertexes),
      usage: wgpu::BufferUsages::VERTEX,
    });

    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("Index Buffer"),
      contents: bytemuck::cast_slice(&model.indicies),
      usage: wgpu::BufferUsages::INDEX,
    });

    Self {
      vertex_buffer,
      index_buffer,
      model,
    }
  }

  #[inline]
  pub fn get_indicies(&self) -> u32 {
    self.model.indicies.len() as u32
  }

  fn render_with_current_pipeline(&self, render_pass: &mut wgpu::RenderPass) {
    render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
    render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
    render_pass.draw_indexed(0..self.get_indicies(), 0, 0..1);
  }
}

/// a material, with a bunch of mesh data to boot
pub struct ShaderPipeline {
  pub filename: &'static str,
  pub module: wgpu::ShaderModule,
  pub pipeline: wgpu::RenderPipeline,
  pub layout: wgpu::PipelineLayout,
  pub bind_group_layouts: Vec<wgpu::BindGroupLayout>,
  // reference to the pool of geometry
  pub geometry: Vec<RwLock<GeometryBuffer>>,
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

    Self {
      // more added later, as more meshes are applied to the same material.
      geometry: Vec::new(),

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
