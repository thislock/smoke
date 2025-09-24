use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

use crate::update_manager::{Task, TaskCommand, TaskResult};

// contains the unsafe impl as much as possible by putting it in this module

pub struct SdlTask {
  handle: Option<SdlHandle>,
}

impl Default for SdlTask {
  fn default() -> Self {
    Self { handle: None }
  }
}

impl Task for SdlTask {
  fn start(&mut self) -> anyhow::Result<&'static str> {
    self.handle = Some(SdlHandle::new()?);
    Ok("sdl3 task")
  }
  fn end(&mut self) -> anyhow::Result<()> {
    self.handle = None;
    Ok(())
  }

  fn update(&mut self, messages: &[TaskCommand]) -> TaskResult {
    for msg in messages {
      match msg {
        TaskCommand::Print => println!("WOWIE ZOWIE"),
      }
    }

    if let Some(sdl_handle) = &mut self.handle {
      for event in sdl_handle.event_pump.poll_iter() {
        match event {
          sdl3::event::Event::Quit { .. } => {
            return TaskResult::RequestShutdown;
          }
          _ => {}
        }
      }
    }

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
