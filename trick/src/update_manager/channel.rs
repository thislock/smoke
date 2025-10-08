use flume::{Receiver, Sender};

pub struct TaskChannel<T> {
  sender: Sender<T>,
  receiver: Receiver<T>,
}

impl<T> TaskChannel<T> {
  /// Create a new unbounded message channel
  pub fn new() -> Self {
    let (sender, receiver) = flume::unbounded();
    Self { sender, receiver }
  }

  /// Send a message (thread-safe)
  pub fn send(&self, msg: T) -> Result<(), ()> {
    self.sender.send(msg).map_err(|_| ())
  }

  /// Receive a message (blocking)
  pub fn recv(&self) -> Option<T> {
    self.receiver.recv().ok()
  }

  /// Non-blocking try receive
  pub fn try_recv(&self) -> Option<T> {
    self.receiver.try_recv().ok()
  }

  /// Async receive (if using async runtimes)
  pub async fn recv_async(&self) -> Option<T> {
    self.receiver.recv_async().await.ok()
  }

}
