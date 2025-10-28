use crate::{renderer::registry::HardwareMessage, update_manager::container::TaskPermission};

pub mod channel;
pub mod container;

pub enum TaskResult {
  /// system error: the entire program or task needs to go down.
  ErrFatal(&'static str),
  /// logic error: just restart the task.
  ErrReload,
  /// success: nothing of note to send
  Ok,
  /// request shutdown of the entire program
  RequestShutdown,
}

pub enum TaskRequest {
  /// link to channel with ID
  LinkChannel(&'static str),
}

#[derive(PartialEq)]
pub enum TaskTag {
  DropLast,
}

pub struct PostInit {
  pub name: &'static str,
  pub tags: &'static [TaskTag],
  pub requests: &'static [TaskRequest],
}

pub enum ManagerMessage {
  TaskChannel(),
}

pub trait Task {
  fn start(
    &mut self,
    channel_registry: channel::ChannelRegistry<HardwareMessage>,
  ) -> anyhow::Result<PostInit>;
  fn update(&mut self) -> TaskResult;
  fn end(&mut self) -> anyhow::Result<()>;
}

pub enum UpdateReturn {
  Ok,
  Shutdown,
}

pub struct UpdateManager {
  tasks: Vec<container::TaskContainer>,
  hardware_registry: channel::ChannelRegistry<HardwareMessage>,
}

impl Drop for UpdateManager {
  fn drop(&mut self) {
    // remove everything that doesn't have the "DropLast" tag
    self.tasks.retain(|x| x.get_tag().contains(&TaskTag::DropLast));
  }
}

impl UpdateManager {
  pub fn new() -> anyhow::Result<Self> {
    Ok(Self {
      tasks: Vec::new(),
      hardware_registry: channel::ChannelRegistry::new(),
    })
  }

  pub fn add_task<GenericTask: Task + 'static>(
    &mut self,
    task: GenericTask,
    perms: container::TaskPermission,
  ) -> anyhow::Result<()> {
    let task = container::TaskContainer::new(task, perms, self.hardware_registry.clone())?;
    self.tasks.push(task);
    Ok(())
  }

  pub fn update_tasks(&self) -> UpdateReturn {
    for task in &self.tasks {
      let task_result = task.run();
      match task_result {
        TaskResult::ErrFatal(msg) => println!(
          "task {} returned with fatal error: {}",
          task.get_label(),
          msg
        ),
        TaskResult::ErrReload => {
          // attempt to reload, TODO: handle this error properly
          if let Err(error) = task.reload_task(self.hardware_registry.clone()) {
            println!("ERROR: {:?}", error);
            return UpdateReturn::Shutdown;
          }
        }
        TaskResult::Ok => {}
        TaskResult::RequestShutdown => {
          if *task.get_permission() == TaskPermission::Root {
            return UpdateReturn::Shutdown;
          } else {
            // TODO: handle this case
          }
        }
      }
    }

    return UpdateReturn::Ok;
  }
}
