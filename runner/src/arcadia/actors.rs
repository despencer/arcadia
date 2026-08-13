use std::io::{Result, Read, Write};
use byteorder::{ReadBytesExt, WriteBytesExt, LittleEndian};
use crate::arcadia::dispatcher::Dispatcher;
use crate::arcadia::control::Control;

#[derive(Default)]
pub enum ActorLifecycle
{
 #[default]
 Empty,
 Death { id: u64 },
 Make { parent: u64, credits: u32, home: u64, startup: Vec<u8>  }
}

#[derive(Default)]
pub enum ActorInside
{
 #[default]
 Empty,
 Make { credits: u32, startup: Vec<u8> }
}

#[derive(Default)]
pub struct Body
{
 pub id: u64,
 home: u64,
 credits: u32,
 reserve: u32,
 inside: Dispatcher<ActorInside>
}

impl Body
{
 pub fn get_credits(&self) -> u32
 {
   self.credits.saturating_sub(self.reserve)
 }

 pub fn take_credits(&mut self, amount: u32) -> u32
 {
  let ret = self.get_credits().saturating_sub(amount);
  self.reserve += ret;
  ret
 }

 pub fn birth(&mut self, credits: u32, startup: Vec<u8>)
 {
  log::debug!("Body birth request {} have {}", credits, self.credits);
  if credits < self.credits
   {
   self.credits -= credits;
   self.inside.put( ActorInside::Make { credits: credits, startup: startup } );
   }
 }

 pub fn tick(&mut self, outside: &mut Dispatcher<ActorLifecycle>)
 {
  while self.inside.len() > 0
     {
     match self.inside.get()
       {
         ActorInside::Make {credits, startup} => outside.put( ActorLifecycle::Make { parent: self.id, credits: credits, home: self.home, startup:startup } ),
         _ => {}
       }
     }

 }
}

#[derive(Default)]
pub struct Actor
{
 body: Body,
 control: Control
}

impl Actor
{
 pub fn get_id(&self) -> u64
 { self.body.id }

 pub fn tick(&mut self, dispatcher: &mut Dispatcher<ActorLifecycle>)
 {
  self.body.tick(dispatcher);
  self.control.tick(&mut self.body);
 }

 pub fn billing(&mut self, amount: u32, dispatcher: &mut Dispatcher<ActorLifecycle>)
 {
  if self.body.credits >= amount
    {  self.body.credits -= amount; }
  else
    { self.body.credits = 0; dispatcher.put( ActorLifecycle::Death {id : self.body.id} ); }
 }

 pub fn feed(&mut self, amount: u32)
 {
  self.body.credits = self.body.credits.saturating_add(amount);
 }

 pub fn new(id: u64, credits: u32, home: u64,  startup: Vec<u8>) -> Actor
 {
  let mut actor = Actor::default();
  actor.body.id = id;
  actor.body.home = home;
  actor.body.credits = credits;
  actor.control.new(startup);
  actor
 }

 pub fn load_1<R:Read>(source: &mut R) -> Result<Self>
 {
   let mut actor = Actor::default();
   actor.body.id = source.read_u64::<LittleEndian>()?;
   actor.body.home = source.read_u64::<LittleEndian>()?;
   log::debug!("Actor {} loading", actor.body.id);
   actor.body.credits = source.read_u32::<LittleEndian>()?;
   actor.body.reserve = source.read_u32::<LittleEndian>()?;
   actor.control.load_1(source)?;
   log::debug!("Actor {} loaded, {} credits", actor.body.id, actor.body.credits);
   Ok(actor)
 }

 pub fn save_1<W:Write>(&self, target: &mut W) -> Result<()>
 {
   log::debug!("Saving actor {}", self.body.id);
   target.write_u64::<LittleEndian>(self.body.id)?;
   target.write_u64::<LittleEndian>(self.body.home)?;
   target.write_u32::<LittleEndian>(self.body.credits)?;
   target.write_u32::<LittleEndian>(self.body.reserve)?;
   self.control.save_1(target)?;
   Ok(())
 }
}