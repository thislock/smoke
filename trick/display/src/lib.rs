
pub fn something() {
  println!("hello from display!")
}

trait DisplayHandle {
  fn get_raw_handle(&self);
}