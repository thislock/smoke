#[derive(Clone)]
pub enum HardwareMessage {
  RequestRawWindowHandle,
  RawWindowHandle(SyncRawWindow),
}

#[derive(Clone, Copy)]
pub struct SyncRawWindow(
  pub raw_window_handle::RawWindowHandle,
  pub raw_window_handle::RawDisplayHandle,
);
unsafe impl Send for SyncRawWindow {}
unsafe impl Sync for SyncRawWindow {}
