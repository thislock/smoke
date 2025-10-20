use crate::{
  renderer::registry::{HardwareMessage, SurfaceChanges, SurfaceResolution, SyncRawWindow},
  update_manager::{self, channel::{self, TaskChannel, TaskSender}, Task, TaskResult},
};

// contains the unsafe impl as much as possible by putting it in this module

pub struct SdlTask {
  handle: Option<SdlHandle>,
  channel_reg: Option<channel::ChannelRegistry<HardwareMessage>>,
  renderer_channel: Option<channel::TaskChannel<HardwareMessage>>,
}

impl Default for SdlTask {
  fn default() -> Self {
    Self {
      handle: None,
      channel_reg: None,
      renderer_channel: None,
    }
  }
}

impl SdlTask {
  fn sync_renderer_channel<'a>(
    &'a mut self,
  ) -> &'a mut Option<channel::TaskChannel<HardwareMessage>> {
    if let Some(_renderer_channel) = &mut self.renderer_channel {
      return &mut self.renderer_channel;
    }

    if let Some(channel_registry) = self.channel_reg.clone() {
      let channel_request =
        channel_registry.get_or_create(crate::renderer::renderer::RENDERER_CHANNEL);

      if let Some(channel_accepted) = channel_request {
        self.renderer_channel = Some(channel_accepted);
      }
    }

    return &mut self.renderer_channel;
  }
}

impl Task for SdlTask {
  fn start(
    &mut self,
    channel_registry: channel::ChannelRegistry<HardwareMessage>,
  ) -> anyhow::Result<update_manager::PostInit> {
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
    // i know this is terrible, but i just want my stupid code to work, and daddy borrow checker says no.
    let mut raw_window = None;
    if let Some(sdl_handle) = &mut self.handle {
      raw_window = Some(
        sdl_handle
          .get_handles()
          .expect("failed to get raw window handle"),
      );
    }

    // recieve updates from the renderer channel
    if let Some(renderer_channel) = self.sync_renderer_channel() {
      while let Some(message) = renderer_channel.try_recv() {
        match message {
          HardwareMessage::RequestRawWindowHandle => {
            if let Some(ref window_handle) = raw_window {
              renderer_channel
                .send(HardwareMessage::RenderSyncro(window_handle.clone()))
                .expect("FAILED TO SEND MESSAGE");
            }
          }
          _ => {}
        }
      }
    }

    // sdl handle mutex scope START
    {
      let sdl_handle = { self.handle.as_mut().unwrap() };

      for event in sdl_handle.event_pump.poll_iter() {
        match event {
          sdl3::event::Event::Quit { .. } => {
            return TaskResult::RequestShutdown;
          },
          sdl3::event::Event::Window { win_event, .. } => {
            match win_event {
              sdl3::event::WindowEvent::Resized( .. ) => {
                if let Some(sender) = &sdl_handle.renderer_channel {

                  let window_resolution = {
                    let size = sdl_handle.sdl_window.size();
                    SurfaceResolution {
                      width: size.0,
                      height: size.1,
                    }
                  };
                  sender.send(SurfaceChanges::UpdateResolution(window_resolution)).unwrap();

                }
              }
              _=> {},
            }
          },
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

  pub renderer_channel: Option<TaskSender<SurfaceChanges>>,
}

unsafe impl Send for SdlHandle {}
unsafe impl Sync for SdlHandle {}

const DEFAULT_RESOLUTION: [u32; 2] = [800, 600];

use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

impl SdlHandle {

  fn send_renderer_changes(&self) {
    if let Some(sender) = &self.renderer_channel {
      let window_resolution = self.get_surface_resolution();
      sender.send(SurfaceChanges::UpdateResolution(window_resolution)).unwrap();
    }
  }

  fn get_surface_resolution(&self) -> SurfaceResolution {
    let size = self.sdl_window.size();
    SurfaceResolution {
      width: size.0,
      height: size.1,
    }
  }

  fn get_handles(&mut self) -> anyhow::Result<SyncRawWindow> {
    let display_handle = self.sdl_window.display_handle()?.as_raw();
    let window_handle = self.sdl_window.window_handle()?.as_raw();

    // create renderer channel
    let (send, reciever) = TaskChannel::new().split();
    self.renderer_channel = Some(send);
    self.send_renderer_changes();

    return Ok(SyncRawWindow(
      window_handle,
      display_handle,
      reciever,
    ));
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

      renderer_channel: None,
    })
  }
}
