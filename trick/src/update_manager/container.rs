
use std::sync::{Arc, Mutex};
use crate::update_manager;

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
    let mut task_lock = self.task.lock().expect(error_msg);
    task_lock.end().expect(error_msg);
  }
}

impl TaskContainer {
  pub fn new<T: update_manager::Task + 'static>(mut task: T, permissions: TaskPermission) -> anyhow::Result<Self>
  where
    T: Sized,
  {
    let mut label = "BLANK TASK LABEL";

    if let Ok(task_label) = task.start() {
      label = task_label;
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

  pub fn reload_task(&self) -> anyhow::Result<()> {
    let mut task_lock = self.task.lock().unwrap();
    task_lock.end()?;
    task_lock.start()?;
    Ok(())
  }

  pub fn run(&self, messages: &[update_manager::TaskCommand]) -> update_manager::TaskResult {
    if let Ok(mut task_lock) = self.task.lock() {
      return task_lock.update(messages);
    } else {
      return update_manager::TaskResult::ErrFatal("FAILED TO UNLOCK MUTEX");
    }
  }
}
