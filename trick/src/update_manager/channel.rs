use flume::{Receiver, Sender};

#[derive(Clone)]
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

use std::collections::HashMap;
use std::any::Any;
use std::sync::{Arc, Mutex};

type ChannelId = &'static str;
pub type Message = Box<dyn Any + Send + 'static>;

#[derive(Clone)]
pub struct ChannelRegistry {
  inner: Arc<Mutex<HashMap<ChannelId, PendingChannel>>>,
}

enum PendingChannel {
  Waiting(TaskChannel<Message>),
  Pending(TaskChannel<Message>),
}

impl ChannelRegistry {
  pub fn new() -> Self {
    Self {
      inner: Arc::new(Mutex::new(HashMap::new())),
    }
  }

  /// Request or create a channel with the given ID.
  /// - If no other task has requested this ID yet, store it.
  /// - If another task is waiting, link the channels and remove the entry.
  pub fn get_or_create(&self, id: &'static str) -> Option<TaskChannel<Message>> {
    
    let mut map = self.inner.lock().ok()?;

    // if the channel was accepted already, stop and return it.
    if let Some(PendingChannel::Pending(_)) = map.get(id) {
      // nested statement so it doesnt remove any Waiting state items.
      if let Some(PendingChannel::Pending(accepted_channel)) = map.remove(id) {
        return Some(accepted_channel);
      }
    }

    // we don't need to check if the statement is Pending here, because it would've returned if it was.

    if let Some(PendingChannel::Waiting(mut matching_channel)) = map.remove(id) {
      // we got the channel verified, now create a new one, and link them up.
      let mut new_channel = TaskChannel::new();

      // swap around the recievers so they get messages from one another
      let bucket = new_channel.receiver;
      new_channel.receiver = matching_channel.receiver;
      matching_channel.receiver = bucket;

      // return everything back

      // add the other channel to the accepted queue 
      // (side tangent, the spelling for "queue" is total bullshit, i had to google it for this stupid comment)
      map.insert(
        id,
        // set it to pending
        PendingChannel::Pending(
          matching_channel
        ),
      )?;

      return Some(new_channel);
    
    } else {
      // First task to request this channel
      let channel = TaskChannel::new();
      map.insert(
        id,
        PendingChannel::Waiting(
          channel,
        ),
      )?;
      
      return None;
    }
  }
}
