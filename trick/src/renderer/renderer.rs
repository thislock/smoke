use std::sync::Arc;

use async_std::sync::Mutex;

use crate::{
  renderer::registry::{HardwareMessage, SyncRawWindow},
  update_manager::{channel, PostInit, Task, TaskResult},
};

pub const RENDERER_CHANNEL: &'static str = "IPEPIFSUIHDFIUHSIHGIHSFUIGHIYWHWRURUURURURURUUR"; // computers don't need clarity

pub struct RendererTask {
  wgpu: Option<WgpuRenderer>,
  channel_registry: Option<channel::ChannelRegistry<HardwareMessage>>,
  renderer_channel: Option<channel::TaskChannel<HardwareMessage>>,
}

impl RendererTask {
  fn sync_renderer_channel<'a>(
    &'a mut self,
  ) -> &'a mut Option<channel::TaskChannel<HardwareMessage>> {
    if let Some(_renderer_channel) = &mut self.renderer_channel {
      return &mut self.renderer_channel;
    }

    if let Some(channel_registry) = &self.channel_registry {
      let channel_request = channel_registry.get_or_create(RENDERER_CHANNEL);

      if let Some(channel_accepted) = channel_request {
        self.renderer_channel = Some(channel_accepted);
      }
    }

    return &mut self.renderer_channel;
  }
}

impl Default for RendererTask {
  fn default() -> Self {
    Self {
      wgpu: None,
      channel_registry: None,
      renderer_channel: None,
    }
  }
}

impl Task for RendererTask {
  fn start(
    &mut self,
    channel_registry: channel::ChannelRegistry<HardwareMessage>,
  ) -> anyhow::Result<PostInit> {
    self.channel_registry = Some(channel_registry);

    Ok(PostInit {
      name: "renderer task",
      requests: &[],
    })
  }

  fn update(&mut self) -> TaskResult {
    let is_wgpu_initialised = self.wgpu.is_none();
    let mut new_wgpu = None;

    if let Some(channel) = self.sync_renderer_channel() {
      if is_wgpu_initialised {
        channel
          .send(HardwareMessage::RequestRawWindowHandle)
          .unwrap();
      }

      while let Some(message) = channel.try_recv() {
        match message {
          HardwareMessage::RawWindowHandle(raw_window) => {
            new_wgpu = WgpuRenderer::new(raw_window).ok();
          }
          _ => {}
        }
      }
    }

    if new_wgpu.is_some() {
      self.wgpu = new_wgpu;
    }

    return TaskResult::Ok;
  }

  fn end(&mut self) -> anyhow::Result<()> {
    self.wgpu = None;
    Ok(())
  }
}

struct WgpuRenderer {
  surface: wgpu::Surface<'static>,
  device: wgpu::Device,
  queue: wgpu::Queue,
  config: wgpu::SurfaceConfiguration,
  is_surface_configured: bool,
  window: Arc<SyncRawWindow>,
}

pub fn async_facade<F, T>(future: F) -> T
where
  F: Future<Output = T>,
{
  async_std::task::block_on(future)
}

impl WgpuRenderer {
  pub fn new(window: SyncRawWindow) -> anyhow::Result<Self> {
    let window = Arc::new(window);

    // The instance is a handle to our GPU
    // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
      #[cfg(not(target_arch = "wasm32"))]
      backends: wgpu::Backends::PRIMARY,
      #[cfg(target_arch = "wasm32")]
      backends: wgpu::Backends::GL,
      ..Default::default()
    });

    let surface = unsafe {
      instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
        raw_window_handle: window.0,
        raw_display_handle: window.1,
      })
    }?;

    let adapter = async_facade(async {
      instance
        .request_adapter(&wgpu::RequestAdapterOptions {
          power_preference: wgpu::PowerPreference::default(),
          compatible_surface: Some(&surface),
          force_fallback_adapter: false,
        })
        .await
    })?;

    let (device, queue) = async_facade(async {
      adapter
        .request_device(&wgpu::DeviceDescriptor {
          label: None,
          required_features: wgpu::Features::empty(),
          // WebGL doesn't support all of wgpu's features, so if
          // we're building for the web we'll have to disable some.
          required_limits: if cfg!(target_arch = "wasm32") {
            wgpu::Limits::downlevel_webgl2_defaults()
          } else {
            wgpu::Limits::default()
          },
          memory_hints: Default::default(),
          trace: wgpu::Trace::Off,
        })
        .await
    })?;

    let surface_caps = surface.get_capabilities(&adapter);
    // Shader code in this tutorial assumes an sRGB surface texture. Using a different
    // one will result in all the colors coming out darker. If you want to support non
    // sRGB surfaces, you'll need to account for that when drawing to the frame.
    let surface_format = surface_caps
      .formats
      .iter()
      .find(|f| f.is_srgb())
      .copied()
      .unwrap_or(surface_caps.formats[0]);

    let config = wgpu::SurfaceConfiguration {
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
      format: surface_format,
      width: 800,  // TODO: MAKE THIS ACTUALLY MATCH THE SURFACE.
      height: 600,  // TODO: MAKE THIS ACTUALLY MATCH THE SURFACE.
      present_mode: surface_caps.present_modes[0],
      alpha_mode: surface_caps.alpha_modes[0],
      view_formats: vec![],
      desired_maximum_frame_latency: 2,
    };

    Ok(Self {
      surface,
      device,
      queue,
      config,
      is_surface_configured: true,
      window,
    })
  }
}
