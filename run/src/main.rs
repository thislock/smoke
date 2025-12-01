
use trick::task_routine_prelude::*;
fn test_routine(input: RenderRoutineInput) -> RenderRoutineOutput {
  println!("IM ALIVE!!!!!!!");
  RenderRoutineOutput::Good
}


fn main() -> anyhow::Result<()> {
  use trick::renderer::registry::HardwareMessage;
  let mut program = trick::update_manager::UpdateManager::<HardwareMessage>::new()?;

  let sdl_task = renderer::window::SdlTask::default();
  let mut renderer_task = renderer::renderer::RendererTask::default();

  renderer_task.add_routine(test_routine);

  use trick::*;
  program.add_task(sdl_task, update_manager::container::TaskPermission::Root)?;
  program.add_task(
    renderer_task,
    update_manager::container::TaskPermission::Root,
  )?;

  'main: loop {
    let update_result = program.update_tasks();
    use trick::update_manager::UpdateReturn;
    match update_result {
      UpdateReturn::Ok => {}
      UpdateReturn::Shutdown => {
        break 'main;
      }
    }
  }

  return Ok(());
}
