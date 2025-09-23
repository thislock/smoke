use std::sync::{Arc, Mutex};

pub enum TaskMessage {
  Print
}

pub trait Task {
  /// returns the label of the newly constructed task.
  fn start(&mut self) -> anyhow::Result<&'static str>;

  fn update(&self, messages: &[TaskMessage]) -> anyhow::Result<()>;

  fn end(&mut self) -> anyhow::Result<()>;
}

#[derive(Clone)]
pub struct TaskContainer {
  task: Arc<Mutex<dyn Task>>,
  task_label: &'static str,
}

impl Drop for TaskContainer {
  fn drop(&mut self) {
    let mut error_msg = String::from("failed to drop: ");
    error_msg.push_str(&self.task_label);
    
    let mut task_lock = self.task.lock().expect(&error_msg);
    task_lock.end().expect(&error_msg);
  }
}

impl TaskContainer {
  
  pub fn new<T: Task + 'static>(mut task: T) -> anyhow::Result<Self> where T: Sized  {
    
    let mut label = "BLANK TASK LABEL";

    if let Ok(task_label) = task.start() {
      label = task_label;
    }
    
    Ok(Self { task: Arc::new(Mutex::new(task)), task_label: label })
  }

  pub fn run(&self, messages: &[TaskMessage]) -> anyhow::Result<()> {
    
    if let Ok(task_lock) = self.task.lock() {
      task_lock.update(messages);
    } else {
      // do something idk
    }
    
    Ok(())
  }
  

}

pub struct UpdateManager {
  tasks: Vec<TaskContainer>,
}

impl UpdateManager {

  pub fn new() -> anyhow::Result<Self> {
    Ok(
      Self {
        tasks: Vec::new(),
      }
    )
  }

  pub fn add_task<GenericTask: Task + 'static>(&mut self, task: GenericTask) -> anyhow::Result<()> {
    
    let task = TaskContainer::new(task)?;
    
    self.tasks.push(task);
    
    Ok(())
  }

  pub fn update_tasks(&self) {
    for task in &self.tasks {
      task.run(&[TaskMessage::Print]);
    }
  }

}