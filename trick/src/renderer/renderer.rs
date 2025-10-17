use crate::{
  renderer::{registry::HardwareMessage},
  update_manager::{
    PostInit, Task, TaskResult,
    channel,
  },
};

pub const RENDERER_CHANNEL: &'static str = "IPEPIFSUIHDFIUHSIHGIHSFUIGHIYWHWRURUURURURURUUR"; // computers don't need clarity

pub struct RendererTask {
  wgpu: Option<WgpuRenderer>,
  channel_registry: Option<channel::ChannelRegistry<HardwareMessage>>,
  renderer_channel: Option<channel::TaskChannel<HardwareMessage>>,
}

pub enum RendererMessage {
  RequestRawWindowHandle,
}

impl RendererTask {
  fn sync_renderer_channel<'a>(&'a mut self) -> &'a mut Option<channel::TaskChannel<HardwareMessage>> {
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
  fn start(&mut self, channel_registry: channel::ChannelRegistry<HardwareMessage>) -> anyhow::Result<PostInit> {
    self.channel_registry = Some(channel_registry);

    Ok(PostInit {
      name: "renderer task",
      requests: &[],
    })
  }

  fn update(&mut self) -> TaskResult {
    let is_wgpu_initialised = self.wgpu.is_none();

    if let Some(channel) = self.sync_renderer_channel() {
      if is_wgpu_initialised {
        channel.send(HardwareMessage::RequestRawWindowHandle).unwrap();
      }

      while let Some(message) = channel.try_recv() {
        match message {
          HardwareMessage::RawWindowHandle(raw_window) => {
            println!("raw window handle gotten");
          }
          _ => {}
        }
      }
    }

    return TaskResult::Ok;
  }

  fn end(&mut self) -> anyhow::Result<()> {
    self.wgpu = None;
    Ok(())
  }
}

struct WgpuRenderer {}

impl WgpuRenderer {
  pub fn new() -> Self {
    Self {}
  }
}
