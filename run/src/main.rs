

fn main() -> anyhow::Result<()> {
  let mut program = trick::update_manager::UpdateManager::new()?;
  
  use trick::*;
  program.add_task(
    renderer::window::SdlTask::default(), 
    update_manager::container::TaskPermission::Root
  )?;

  loop {
    let update_result = program.update_tasks();
    use trick::update_manager::*;
    match update_result {
      UpdateReturn::Ok => {}
      UpdateReturn::Shutdown => {
        return Ok(());
      }
    }
  }

}
