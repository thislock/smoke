use flume::{Receiver, Sender};

#[derive(Clone)]
pub struct TaskChannel<T> {
  sender: Sender<T>,
  receiver: Receiver<T>,
}

impl<T: 'static + Send> TaskChannel<T> {
  pub fn new() -> Self {
    let (sender, receiver) = flume::unbounded();
    Self { sender, receiver }
  }

  pub fn send(&self, msg: T) -> Result<(), ()> {
    self.sender.send(msg).map_err(|error| {
      println!("error: {:?}", error);
      ()
    })
  }

  /// Blocking message recieve
  pub fn recv(&self) -> Option<T> {
    self.receiver.recv().ok()
  }

  pub fn try_recv(&self) -> Option<T> {
    self.receiver.try_recv().ok()
  }

  /// Blocking message recieve
  pub async fn recv_async(&self) -> Option<T> {
    self.receiver.recv_async().await.ok()
  }
}

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

type ChannelId = &'static str;

#[derive(Clone)]
pub struct ChannelRegistry<T> {
  inner: Arc<Mutex<HashMap<ChannelId, PendingChannel<T>>>>,
}

enum PendingChannel<T> {
  Waiting(TaskChannel<T>),
  Pending(TaskChannel<T>),
}

impl<T: Clone + Send + 'static> ChannelRegistry<T> {
  pub fn new() -> Self {
    Self {
      inner: Arc::new(Mutex::new(HashMap::new())),
    }
  }

  /// Request or create a channel with the given ID.
  /// - If no other task has requested this ID yet, store it.
  /// - If another task is waiting, link the channels and remove the entry.
  pub fn get_or_create(&self, id: &'static str) -> Option<TaskChannel<T>> {
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
        PendingChannel::Pending(matching_channel),
      );

      return Some(new_channel);
    } else {
      // First task to request this channel
      let channel = TaskChannel::new();
      map.insert(id, PendingChannel::Waiting(channel));

      return None;
    }
  }
}
