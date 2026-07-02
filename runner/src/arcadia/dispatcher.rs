use std::collections::VecDeque;

pub enum Message
{
 Death { id: u64}
}

#[derive(Default)]
pub struct Dispatcher
{
 messages: VecDeque<Message>
}

impl Dispatcher
{
 pub fn put(&mut self, message: Message)
 {
  self.messages.push_back(message);
 }

 pub fn len(&self) -> usize
 {
  self.messages.len()
 }

 pub fn get(&mut self) -> Message
 {
  self.messages.pop_front().unwrap()
 }
}

