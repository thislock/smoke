use crate::update_manager::container::TaskPermission;


pub mod channel;
pub mod container;

// TODO: figure out how channels are sent and recieved upon task initialisation 
pub enum TaskCommand {
  Report,
  LinkChannel(&'static str),
}

pub enum OutgoingTaskMessage {
  /// send a command to every other task
  Yell(TaskCommand),
}

pub enum TaskResult {
  /// system error: the entire program or task needs to go down.
  ErrFatal(&'static str),
  /// logic error: just restart the task.
  ErrReload,              
  /// success: nothing of note to send
  Ok,
  /// success: send a message to another task
  OkMessage(OutgoingTaskMessage), 
  /// request shutdown of the entire program
  RequestShutdown,
}

pub trait Task {
  /// returns the label of the newly constructed task.
  fn start(&mut self) -> anyhow::Result<&'static str>;

  fn update(&mut self, messages: &[TaskCommand]) -> TaskResult;

  fn end(&mut self) -> anyhow::Result<()>;
}

pub enum UpdateReturn {
  Ok,
  Shutdown,
}

pub struct UpdateManager {
  tasks: Vec<container::TaskContainer>,
}


impl UpdateManager {
  pub fn new() -> anyhow::Result<Self> {
    Ok(Self { tasks: Vec::new() })
  }

  pub fn add_task<GenericTask: Task + 'static>(&mut self, task: GenericTask, perms: container::TaskPermission) -> anyhow::Result<()> {
    let task = container::TaskContainer::new(task, perms)?;
    self.tasks.push(task);
    Ok(())
  }

  pub fn update_tasks(&self) -> UpdateReturn {
    for task in &self.tasks {
      let task_result = task.run(&[TaskCommand::Report]);
      match task_result {
        TaskResult::ErrFatal(msg) => println!(
          "task {} returned with fatal error: {}",
          task.get_label(), msg
        ),
        TaskResult::ErrReload => task.reload_task().unwrap(),
        TaskResult::Ok => {}
        TaskResult::OkMessage(_out_msg) => {
          // TODO: work on outgoing messages.
        }

        TaskResult::RequestShutdown => {
          if *task.get_permission() == TaskPermission::Root {
            return UpdateReturn::Shutdown;
          } else {
            // do somethin idk
          }
        }
      }
    }

    return UpdateReturn::Ok;
  }
}
