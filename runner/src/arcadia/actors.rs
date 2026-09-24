use std::io::{Result, Read, Write};
use std::mem;
use crate::arcadia::dispatcher::Dispatcher;
use crate::arcadia::control::Control;
use crate::arcadia::values::Seed;
use crate::arcadia::storage::{Reader,Writer};

#[derive(Default)]
pub enum ActorLifecycle
{
 #[default]
 Empty,
 Death { id: u64 },
 Make { parent: u64, home: u64, seed: Seed  }
}

#[derive(Default)]
pub enum ActorInside
{
 #[default]
 Empty,
 Make { seed: Seed }
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

 pub fn birth(&mut self, seed: Seed)
 {
  log::debug!("Body birth request {} have {}", seed.credits, self.credits);
  if seed.credits < self.credits
   {
   self.credits -= seed.credits;
   self.reserve -= seed.credits;
   self.inside.put( ActorInside::Make { seed: seed } );
   }
 }

 pub fn tick(&mut self, outside: &mut Dispatcher<ActorLifecycle>)
 {
  while self.inside.len() > 0
     {
     match self.inside.get()
       {
         ActorInside::Make {seed} => outside.put( ActorLifecycle::Make { parent: self.id, home: self.home, seed:seed } ),
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

 pub fn new(id: u64, home: u64,  seed: &mut Seed) -> Actor
 {
  let mut actor = Actor::default();
  actor.body.id = id;
  actor.body.home = home;
  actor.body.credits = seed.credits;
  std::mem::swap(&mut actor.control, &mut seed.seed);
  actor
 }

 pub fn load_1(source: &mut dyn Read) -> Result<Self>
 {
   let mut actor = Actor::default();
   let mut reader = Reader::new(source);
   actor.body.id = reader.u64()?;
   actor.body.home = reader.u64()?;
   log::debug!("Actor {} loading", actor.body.id);
   actor.body.credits = reader.u32()?;
   actor.body.reserve = reader.u32()?;
   actor.control.load(&mut reader)?;
   log::debug!("Actor {} loaded, {} credits", actor.body.id, actor.body.credits);
   Ok(actor)
 }

 pub fn save_1(&self, target: &mut dyn Write) -> Result<()>
 {
   log::debug!("Saving actor {}", self.body.id);
   let mut writer = Writer::new(target);
   writer.u64(self.body.id)?;
   writer.u64(self.body.home)?;
   writer.u32(self.body.credits)?;
   writer.u32(self.body.reserve)?;
   self.control.save(&mut writer)?;
   Ok(())
 }
}