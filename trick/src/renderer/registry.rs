use crate::update_manager::channel::TaskReceiver;

#[derive(Clone)]
pub enum HardwareMessage {
  RequestRawWindowHandle,
  RenderSyncro(SyncRawWindow),
  End,
}

#[derive(Clone, Copy)]
pub struct SurfaceResolution {
  pub width: u32,
  pub height: u32,
}

#[derive(Clone, Copy)]
pub enum SurfaceChanges {
  UpdateResolution(SurfaceResolution),
}

#[derive(Clone)]
pub struct SyncRawWindow(
  pub raw_window_handle::RawWindowHandle,
  pub raw_window_handle::RawDisplayHandle,
  pub TaskReceiver<SurfaceChanges>,
);
unsafe impl Send for SyncRawWindow {}
unsafe impl Sync for SyncRawWindow {}
