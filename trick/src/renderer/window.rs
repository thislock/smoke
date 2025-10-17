use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

use crate::update_manager::{self, channel, Task, TaskResult};

// contains the unsafe impl as much as possible by putting it in this module

pub struct SdlTask {
  handle: Option<SdlHandle>,
  channel_reg: Option<channel::ChannelRegistry>,
  renderer_channel: Option<channel::TaskChannel>,
}

impl Default for SdlTask {
  fn default() -> Self {
    Self { handle: None, channel_reg: None, renderer_channel: None }
  }
}

impl SdlTask {
  fn sync_renderer_channel<'a>(&'a mut self) -> &'a mut Option<channel::TaskChannel> {
    
    if let Some(_renderer_channel) = &mut self.renderer_channel {
      return &mut self.renderer_channel;
    }

    if let Some(channel_registry) = self.channel_reg.clone() {
      let channel_request =channel_registry
        .get_or_create(crate::renderer::renderer::RENDERER_CHANNEL);
      
      if let Some(channel_accepted) = channel_request {
        self.renderer_channel = Some(channel_accepted);
      }
    }

    return &mut self.renderer_channel;
  }
}

impl Task for SdlTask {
  fn start(&mut self, channel_registry: channel::ChannelRegistry) -> anyhow::Result<update_manager::PostInit> {
    self.handle = Some(SdlHandle::new()?);
    self.channel_reg = Some(channel_registry);
    let _ = self.sync_renderer_channel();
    return Ok(update_manager::PostInit {
      name: "sdl3 desktop task",
      requests: &[],
    });
  }

  fn end(&mut self) -> anyhow::Result<()> {
    // set to none, dropping everything
    self.handle = None;
    Ok(())
  }

  fn update(&mut self) -> TaskResult {

    // recieve updates from the renderer channel
    if let Some(renderer_channel) = self.sync_renderer_channel() {
      
    }

    // sdl handle mutex scope START
    {
      // change this to unwrap_unchecked later, once that becomes a possible optimization
      // but with only a few tasks, for now it's better to have the error handled properly
      // let mut sdl_handle = unsafe { self.handle.as_mut().unwrap_unchecked() };
      let sdl_handle = { self.handle.as_mut().unwrap() };

      for event in sdl_handle.event_pump.poll_iter() {
        match event {
          sdl3::event::Event::Quit { .. } => {
            return TaskResult::RequestShutdown;
          }
          _ => {}
        }
      }
    }
    // sdl handle mutex scope END

    TaskResult::Ok
  }
}

pub struct SdlHandle {
  pub sdl_context: sdl3::Sdl,
  pub sdl_window: sdl3::video::Window,
  pub event_pump: sdl3::EventPump,
}

unsafe impl Send for SdlHandle {}
unsafe impl Sync for SdlHandle {}

const DEFAULT_RESOLUTION: [u32; 2] = [600, 800];

impl SdlHandle {
  fn get_display(&self) -> anyhow::Result<raw_window_handle::DisplayHandle<'_>> {
    let display_handle = self.sdl_window.display_handle()?;
    return Ok(display_handle);
  }

  fn get_window(&self) -> anyhow::Result<raw_window_handle::WindowHandle<'_>> {
    let window_handle = self.sdl_window.window_handle()?;
    return Ok(window_handle);
  }

  fn new() -> anyhow::Result<Self> {
    let sdl_context = sdl3::init()?;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
      .window("sdl window", DEFAULT_RESOLUTION[0], DEFAULT_RESOLUTION[1])
      .position_centered()
      .resizable()
      .metal_view()
      .build()?;

    let event_pump = sdl_context.event_pump()?;

    Ok(Self {
      sdl_context,
      sdl_window: window,
      event_pump,
    })
  }
}
