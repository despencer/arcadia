use std::io::{Result, Read, Write};
use byteorder::{ReadBytesExt, WriteBytesExt, LittleEndian};
use crate::arcadia::dispatcher::Dispatcher;
use crate::arcadia::control::Control;

#[derive(Default)]
pub enum ActorLifecycle
{
 #[default]
 Empty,
 Death { id: u64}
}

#[derive(Default)]
pub struct Body
{
 credits: u32
}

impl Body
{
 pub fn get_credits(&self) -> u32
 { self.credits }
}

#[derive(Default)]
pub struct Actor
{
 pub id: u64,
 body: Body,
 control: Control
}

impl Actor
{
 pub fn tick(&mut self)
 {
  self.control.tick(&mut self.body);
 }

 pub fn billing(&mut self, amount: u32, dispatcher: &mut Dispatcher<ActorLifecycle>)
 {
  if self.body.credits >= amount
    {  self.body.credits -= amount; }
  else
    { self.body.credits = 0; dispatcher.put( ActorLifecycle::Death {id : self.id} ); }
 }

 pub fn feed(&mut self, amount: u32)
 {
  self.body.credits = self.body.credits.saturating_add(amount);
 }

 pub fn load_1<R:Read>(source: &mut R) -> Result<Self>
 {
   let mut actor = Actor::default();
   actor.id = source.read_u64::<LittleEndian>()?;
   log::debug!("Actor {} loading", actor.id);
   actor.body.credits = source.read_u32::<LittleEndian>()?;
   actor.control.load_1(source)?;
   log::debug!("Actor {} loaded, {} credits", actor.id, actor.body.credits);
   Ok(actor)
 }

 pub fn save_1<W:Write>(&self, target: &mut W) -> Result<()>
 {
   log::debug!("Saving actor {}", self.id);
   target.write_u64::<LittleEndian>(self.id)?;
   target.write_u32::<LittleEndian>(self.body.credits)?;
   self.control.save_1(target)?;
   Ok(())
 }
}