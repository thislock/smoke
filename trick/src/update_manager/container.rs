use std::sync::{Arc, Mutex};
use crate::update_manager::{self, TaskTag, channel::ChannelRegistry};

#[derive(Clone, PartialEq)]
pub enum TaskPermission {
  Root,
}

#[derive(Clone)]
pub struct TaskContainer<M: Clone + Send + 'static> {
  task: Arc<Mutex<dyn update_manager::Task<M>>>,
  task_label: &'static str,
  tags: &'static [TaskTag],
  task_permission: TaskPermission,
}

// impl Drop for TaskContainer {
//   fn drop(&mut self) {
//     let mut error_msg = String::from("failed to drop: ");
//     error_msg.push_str(&self.task_label);
//     let error_msg: &str = &error_msg;

//     // executes the tasks "destructor" after unlocking
//     // while let Ok(mut unlocked_task) = self.task.lock() {
//     //   unlocked_task.end().expect(error_msg);
//     // }
//   }
// }

impl<M: Clone + Send + 'static> TaskContainer<M> {
  pub fn new<TaskT: update_manager::Task<M> + 'static>(
    mut task: TaskT,
    permissions: TaskPermission,
    channel_registry: ChannelRegistry<M>,
  ) -> anyhow::Result<Self>
  where
    TaskT: Sized,
  {
    let mut label = "BLANK TASK LABEL";
    let mut tags: &'static [TaskTag] = &[];

    if let Ok(post_init) = task.start(channel_registry) {
      label = post_init.name;
      tags = post_init.tags;
    }

    Ok(Self {
      task: Arc::new(Mutex::new(task)),
      task_label: label,
      tags,
      task_permission: permissions,
    })
  }

  pub fn get_permission(&self) -> &TaskPermission {
    return &self.task_permission;
  }

  pub fn get_label(&self) -> &str {
    return self.task_label;
  }

  pub fn get_tag(&self) -> &'static [TaskTag] {
    self.tags
  }

  pub fn reload_task(&self, channel_registry: ChannelRegistry<M>) -> anyhow::Result<()> {
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
