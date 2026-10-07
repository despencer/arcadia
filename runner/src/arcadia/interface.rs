use std::collections::VecDeque;
use crate::arcadia::values::Seed;

#[derive(Default)]
pub struct Interface<T>
{
 messages: VecDeque<T>
}

impl<T> Interface<T>
{
 pub fn put(&mut self, message: T)
 {
  self.messages.push_back(message);
 }

 pub fn len(&self) -> usize
 {
  self.messages.len()
 }

 pub fn get(&mut self) -> T
 {
  self.messages.pop_front().unwrap()
 }
}

#[derive(Default)]
pub enum ActorLifecycle
{
 #[default]
 Empty,
 Death { id: u64 },
 Make { parent: u64, home: u64, seed: Seed  }
}
