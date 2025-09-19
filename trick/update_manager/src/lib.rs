
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


pub fn add(left: u64, right: u64) -> u64 {
  left + right
}



#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn it_works() {
    let result = add(2, 2);
    assert_eq!(result, 4);
  }
}
