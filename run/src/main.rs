fn main() -> anyhow::Result<()> {

  let mut program = trick::update_manager::UpdateManager::new()?;

  program.add_task(trick::renderer::window::SdlTask::default())?;

  while true {
    program.update_tasks();
  }

  Ok(())
}
