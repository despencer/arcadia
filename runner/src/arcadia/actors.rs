use std::io::{Result, Read, Write};
use byteorder::{ReadBytesExt, WriteBytesExt, LittleEndian};
use crate::arcadia::dispatcher::{Dispatcher, Message};
use crate::arcadia::control::Control;

#[derive(Default)]
pub struct Actor
{
 pub id: u64,
 credits: u32,
 control: Control
}

impl Actor
{
 pub fn tick(&mut self)
 {
 }

 pub fn billing(&mut self, amount: u32, dispatcher: &mut Dispatcher<Message>)
 {
  if self.credits >= amount
    {  self.credits -= amount; }
  else
    { self.credits = 0; dispatcher.put( Message::Death {id : self.id} ); }
 }

 pub fn feed(&mut self, amount: u32)
 {
  self.credits = self.credits.saturating_add(amount);
 }

 pub fn load_1<R:Read>(source: &mut R) -> Result<Self>
 {
   let mut actor = Actor::default();
   actor.id = source.read_u64::<LittleEndian>()?;
   actor.credits = source.read_u32::<LittleEndian>()?;
   actor.control.load_1(source)?;
   Ok(actor)
 }

 pub fn save_1<W:Write>(&self, target: &mut W) -> Result<()>
 {
   log::debug!("Saving actor {}", self.id);
   target.write_u64::<LittleEndian>(self.id)?;
   target.write_u32::<LittleEndian>(self.credits)?;
   self.control.save_1(target)?;
   Ok(())
 }
}