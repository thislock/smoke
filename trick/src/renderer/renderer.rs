use crate::update_manager::{channel, PostInit, Task, TaskResult};

pub const RENDERER_CHANNEL: &'static str = "IPEPIFSUIHDFIUHSIHGIHSFUIGHIYWHWRURUURURURURUUR"; // computers don't need clarity

pub struct RendererTask {
  wgpu: Option<WgpuRenderer>,
  channel_registry: Option<channel::ChannelRegistry>,
  renderer_channel: Option<channel::TaskChannel<channel::Message>>,
}

impl RendererTask {
  fn sync_renderer_channel<'a>(&'a mut self) -> &'a mut Option<channel::TaskChannel<channel::Message>> {
    
    if let Some(_renderer_channel) = &mut self.renderer_channel {
      return &mut self.renderer_channel;
    }

    if let Some(channel_registry) = self.channel_registry.clone() {
      let channel_request =channel_registry
        .get_or_create(RENDERER_CHANNEL);
      
      if let Some(channel_accepted) = channel_request {
        self.renderer_channel = Some(channel_accepted);
      }
    }

    return &mut self.renderer_channel;
  }
}

impl Default for RendererTask {
  fn default() -> Self {
    Self { wgpu: None, channel_registry: None, renderer_channel: None }
  }
}

impl Task for RendererTask {
  fn start(&mut self, channel_registry: channel::ChannelRegistry) -> anyhow::Result<PostInit> {

    self.channel_registry = Some(channel_registry);

    Ok(
      PostInit {
        name: "renderer task",
        requests: &[],
      }
    )
  }

  fn update(&mut self) -> TaskResult {
    
    if let Some(channel) = self.sync_renderer_channel() {
      println!("LEZ GOOOOOOOOOOOOOO!!!!!!!!!!!!!!");
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
