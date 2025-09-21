pub trait Task {
  fn start(&mut self) -> anyhow::Result<()>;
  fn update(&self) -> anyhow::Result<()>;
  fn end(&mut self) -> anyhow::Result<()>;
}

struct TaskContainer {
  task: Box<dyn Task>,
}

pub struct UpdateManager {
  tasks: Vec<TaskContainer>,

}

impl UpdateManager {

  pub fn new() -> anyhow::Result<Self> {
    Ok(
      Self {
        tasks: vec![],
      }
    )
  }

}
