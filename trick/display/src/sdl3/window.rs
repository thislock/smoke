
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use crate::RenderSurface;

// contains the unsafe impl as much as possible by putting it in this module

pub struct SdlHandle {
  pub sdl_context: sdl3::Sdl,
  pub sdl_window: sdl3::video::Window,
  pub event_pump: sdl3::EventPump,
}

// unsafe impl Send for SdlHandle {}
// unsafe impl Sync for SdlHandle {}

impl RenderSurface for SdlHandle {
  fn get_display(&self) -> crate::DisplayResult {
    self.sdl_window.display_handle()
  }
  fn get_window(&self) -> crate::WindowResult {
    self.sdl_window.window_handle()
  }
  fn new() -> anyhow::Result<Self> {

    let sdl_context = sdl3::init()?;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
      .window("sdl window", crate::DEFAULT_RESOLUTION[0],crate::DEFAULT_RESOLUTION[1])
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
