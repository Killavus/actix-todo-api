#[derive(Debug)]
pub struct TodoList {
  id: Option<u32>,
  name: String,
}

impl TodoList {
  pub fn id(&self) -> Option<u32> {
    self.id.clone()
  }
}
