use std::io::{Result, Error, ErrorKind};
use crate::arcadia::common::Id;
use crate::arcadia::commune::Commune;
use crate::arcadia::interface::{OldInterface, Hub};
use crate::arcadia::interface;
use crate::arcadia::control::{Compartment, UnitFactory};
use crate::arcadia::values::Seed;
use crate::arcadia::storage::{Reader, Writer, Factory, Stored};
use crate::arcadia::places::World;

/*#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct ActorId(u64);

impl ActorId
{
 pub fn new(id: u64) -> Self
  { ActorId(id) }
}

impl Factory for ActorId
{
 fn load(reader: &mut Reader) -> Result<Self>
  { Ok(ActorId(reader.u64()?)) }
}

impl Stored for ActorId
{
 fn save(&self, writer: &mut Writer) -> Result<()>
  { writer.u64(self.0) }
}*/

#[derive(Default, Debug)]
pub enum ActorInside
{
 #[default]
 Empty,
 Make { seed: Seed }
}

#[derive(Default, Debug)]
pub struct Body
{
 pub id: Id<Actor>,
 home: Id<World>,
 credits: u32,
 reserve: u32,
 inside: OldInterface<ActorInside>,
 commune: Id<Commune>,
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

 pub fn billing(&mut self, amount: u32, hub: &mut Hub)
 {
  if self.credits >= amount
    {  self.credits -= amount; }
  else
    {
    self.credits = 0;
    hub.send(self.commune, interface::Universe::Actor { actor: interface::Actor::Drop { id: self.id } } );
    }
 }

 pub fn feed(&mut self, amount: u32)
 {
  self.credits = self.credits.saturating_add(amount);
 }

 pub fn tick(&mut self, hub: &mut Hub)
 {
  while let Some(event) = self.inside.get()
    {
     match event
       {
         ActorInside::Make {seed} => hub.send(self.commune, interface::Universe::Actor { actor: interface::Actor::Make { parent: self.id, home: self.home, seed:seed } } ),
         _ => {}
       }
    }
  for event in hub.mail(self.id)
    {
    match event
      {
      interface::Universe::Actor {actor} =>
              match actor
                  {
                  interface::Actor::Feed {amount, ..} => self.feed(amount),
                  interface::Actor::Bill {amount, ..} => self.billing(amount, hub),
                  _ => {}
                  }
      }
    }
 }
}

#[derive(Default, Debug)]
pub struct Actor
{
 body: Body,
 control: Box<Compartment>
}

impl Actor
{
 fn version(&self) -> u8 { 1 }

 pub fn get_id(&self) -> Id<Actor>
 { self.body.id }

 pub fn tick(&mut self, hub: &mut Hub)
 {
  self.body.tick(hub);
  self.control.tick(&mut self.body);
 }

 pub fn new(id: Id<Actor>, home: Id<World>,  seed: &mut Seed) -> Actor
 {
  let mut actor = Actor::default();
  actor.body.id = id;
  actor.body.home = home;
  actor.body.credits = seed.credits;
  std::mem::swap(&mut actor.control, &mut seed.seed);
  actor
 }
}

impl Factory for Actor
{
 fn load(reader: &mut Reader) -> Result<Self>
 {
   let mut actor = Actor::default();
   if reader.u8()? > actor.version()
         { return Err(Error::new(ErrorKind::InvalidData, "Unknown actor version")); }
   actor.body.id = Id::load(reader)?;
   actor.body.home = Id::load(reader)?;
   actor.body.credits = reader.u32()?;
   actor.body.reserve = reader.u32()?;
   let factory = UnitFactory::default();
   actor.control = Compartment::load(reader, &factory)?;
   log::debug!("Actor {:?} loaded, {} credits", actor.body.id, actor.body.credits);
   Ok(actor)
 }
}

impl Stored for Actor
{
 fn save(&self, writer: &mut Writer) -> Result<()>
 {
   log::debug!("Saving actor {:?}", self.body.id);
   writer.u8(self.version())?;
   self.body.id.save(writer)?;
   self.body.home.save(writer)?;
   writer.u32(self.body.credits)?;
   writer.u32(self.body.reserve)?;
   self.control.save(writer)?;
   Ok(())
 }
}