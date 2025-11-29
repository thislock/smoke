// is owned by a given task, with each task defining the specific mannorism of a given routine.
pub trait TaskRoutine {
  // static functions defined by the implementer
  type RoutineFn: Fn(Self::RoutineInput) -> Self::RoutineOutput;
  type RoutineOutput;
  type RoutineInput;
  // for users who know what is implementing TaskRoutine
  fn add_routine(&mut self, routine: Self::RoutineFn);
  // for users who just want to run the implementation of TaskRoutine
  /// if data needs to be modified, changes will need to be sent through the 
  /// RoutineOutput channel, and merged with the main data.
  fn run_routines(&mut self);
}
