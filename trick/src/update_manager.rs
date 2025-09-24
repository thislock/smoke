use std::sync::{Arc, Mutex};

pub enum TaskCommand {
  Print,
}

pub enum OutgoingTaskMessage {
  Yell(TaskCommand), // send a command to every other task
}

pub enum TaskResult {
  ErrFatal(&'static str), // system error: the entire program needs to go down.
  ErrReload,              // logic error: just restart the task.
  Ok,                     // success: nothing to send
  OkMessage(OutgoingTaskMessage), // success: send a message to another task
  RequestShutdown,
}

pub trait Task {
  /// returns the label of the newly constructed task.
  fn start(&mut self) -> anyhow::Result<&'static str>;

  fn update(&mut self, messages: &[TaskCommand]) -> TaskResult;

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
  pub fn new<T: Task + 'static>(mut task: T) -> anyhow::Result<Self>
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
    })
  }

  pub fn reload_task(&self) -> anyhow::Result<()> {
    let mut task_lock = self.task.lock().unwrap();
    task_lock.end()?;
    task_lock.start()?;
    Ok(())
  }

  pub fn run(&self, messages: &[TaskCommand]) -> TaskResult {
    if let Ok(mut task_lock) = self.task.lock() {
      return task_lock.update(messages);
    } else {
      return TaskResult::ErrFatal("FAILED TO UNLOCK MUTEX");
    }

    TaskResult::Ok
  }
}

pub struct UpdateManager {
  tasks: Vec<TaskContainer>,
}

pub enum UpdateReturn {
  Ok,
  Shutdown,
}

impl UpdateManager {
  pub fn new() -> anyhow::Result<Self> {
    Ok(Self { tasks: Vec::new() })
  }

  pub fn add_task<GenericTask: Task + 'static>(&mut self, task: GenericTask) -> anyhow::Result<()> {
    let task = TaskContainer::new(task)?;

    self.tasks.push(task);

    Ok(())
  }

  pub fn update_tasks(&self) -> UpdateReturn {
    for task in &self.tasks {
      let task_result = task.run(&[TaskCommand::Print]);
      match task_result {
        TaskResult::ErrFatal(msg) => println!(
          "task {} returned with fatal error: {}",
          task.task_label, msg
        ),
        TaskResult::ErrReload => task.reload_task().unwrap(),
        TaskResult::Ok => {}
        TaskResult::OkMessage(out_msg) => {
          // TODO: work on outgoing messages.
        }

        TaskResult::RequestShutdown => {
          return UpdateReturn::Shutdown;
        }
      }
    }

    return UpdateReturn::Ok;
  }
}
