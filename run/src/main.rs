use std::arch::x86_64::_MM_MANTISSA_NORM_ENUM;

use trick::update_manager::UpdateReturn;

fn main() -> anyhow::Result<()> {
  let mut program = trick::update_manager::UpdateManager::new()?;

  program.add_task(trick::renderer::window::SdlTask::default())?;

  while true {
    let update_result = program.update_tasks();
    use trick::update_manager::*;
    match update_result {
      UpdateReturn::Ok => {}
      UpdateReturn::Shutdown => {
        return Ok(());
      }
      _ => {}
    }
  }

  Ok(())
}
