
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
mod sdl3;

// thanks, chatgpt!
fn wrap_surface<T: RenderSurface + 'static>(res: anyhow::Result<T>) -> Option<Box<dyn RenderSurface>> {
  res.ok().map(|h| Box::new(h) as Box<dyn RenderSurface>)
}

pub fn try_surfaces() -> Option<Box<dyn RenderSurface>> {
  let surfaces= [
    wrap_surface(sdl3::window::SdlHandle::new()),
  ];

  let mut first_working_surface = None;
  for surface in surfaces {
    if let Some(ok_surface) = surface {
      first_working_surface = Some(ok_surface);
      break;
    }
  }

  return first_working_surface;
}

pub type DisplayResult<'a> = Result<raw_window_handle::DisplayHandle<'a>, raw_window_handle::HandleError>;
pub type WindowResult<'a> = Result<raw_window_handle::WindowHandle<'a>, raw_window_handle::HandleError>;

const DEFAULT_RESOLUTION: [u32;2] = [800, 600];
pub trait RenderSurface {
  fn get_display(&self) -> DisplayResult;
  fn get_window(&self) -> WindowResult;
  fn new() -> anyhow::Result<Self> where Self: Sized;
}

impl HasWindowHandle for dyn RenderSurface {
  fn window_handle(&self) -> Result<raw_window_handle::WindowHandle, raw_window_handle::HandleError> {
    self.get_window()
  }
}

impl HasDisplayHandle for dyn RenderSurface {
  fn display_handle(&self) -> Result<raw_window_handle::DisplayHandle<'_>, raw_window_handle::HandleError> {
    self.get_display()
  }
}