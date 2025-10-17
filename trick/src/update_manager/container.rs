use std::sync::{Arc, Mutex};
use crate::{renderer::registry::HardwareMessage, update_manager::{self, channel::ChannelRegistry}};

#[derive(Clone, PartialEq)]
pub enum TaskPermission {
  Root,
}

#[derive(Clone)]
pub struct TaskContainer {
  task: Arc<Mutex<dyn update_manager::Task>>,
  task_label: &'static str,
  task_permission: TaskPermission,
}

impl Drop for TaskContainer {
  fn drop(&mut self) {
    let mut error_msg = String::from("failed to drop: ");
    error_msg.push_str(&self.task_label);
    let error_msg: &str = &error_msg;

    // executes the tasks "destructor" after unlocking

    // ok this crashes things, so im going to leave it be

    // let mut task_lock = self.task.lock().expect(error_msg);
    // task_lock.end().expect(error_msg);
  }
}

impl TaskContainer {
  pub fn new<TaskT: update_manager::Task + 'static>(
    mut task: TaskT,
    permissions: TaskPermission,
    channel_registry: ChannelRegistry<HardwareMessage>,
  ) -> anyhow::Result<Self>
  where
    TaskT: Sized,
  {
    let mut label = "BLANK TASK LABEL";

    if let Ok(post_init) = task.start(channel_registry) {
      label = post_init.name;
    }

    Ok(Self {
      task: Arc::new(Mutex::new(task)),
      task_label: label,
      task_permission: permissions,
    })
  }

  pub fn get_permission(&self) -> &TaskPermission {
    return &self.task_permission;
  }

  pub fn get_label(&self) -> &str {
    return self.task_label;
  }

  pub fn reload_task(&self, channel_registry: ChannelRegistry<HardwareMessage>) -> anyhow::Result<()> {
    let mut task_lock = self.task.lock().unwrap();
    task_lock.end()?;
    task_lock.start(channel_registry)?;
    Ok(())
  }

  pub fn run(&self) -> update_manager::TaskResult {
    if let Ok(mut task_lock) = self.task.lock() {
      return task_lock.update();
    } else {
      return update_manager::TaskResult::ErrFatal("FAILED TO UNLOCK MUTEX");
    }
  }
}
