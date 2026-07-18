use std::collections::VecDeque;

#[derive(Default)]
pub enum Message
{
 #[default]
 Empty,
 Death { id: u64}
}

#[derive(Default)]
pub struct Dispatcher<T>
{
 messages: VecDeque<T>
}

impl<T> Dispatcher<T>
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

