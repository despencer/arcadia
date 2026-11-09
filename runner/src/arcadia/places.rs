use std::io::{Result};
use crate::arcadia::common::Id;
use crate::arcadia::storage::{Reader, Writer, Factory, Stored};
use crate::arcadia::depot::{Depot, DepotIndex};
use crate::arcadia::actors::Actor;
use crate::arcadia::interface::{OldInterface, Hub};
use crate::arcadia::interface;
use crate::arcadia::control::Sampler;

/*#[derive(Default)]
pub struct Container
{
 billing: u32,
 oldactors: Vec<DepotIndex>,
}

impl Container
{
 pub fn insert(&mut self, actor: DepotIndex)
 {
  self.oldactors.push(actor);
 }

 pub fn drop_actor(&mut self, actor: DepotIndex)
 {
  if let Some(index) = self.oldactors.iter().position(|x| *x == actor)
    { self.oldactors.remove(index); }
 }

 pub fn load(&mut self, reader: &mut Reader) -> Result<()>
 {
  self.billing = reader.u32()?;
  Ok(())
 }

 pub fn save(&self, writer: &mut Writer) -> Result<()>
 {
  writer.u32(self.billing)?;
  Ok(())
 }

}*/

#[derive(Default)]
pub struct World
{
 id: u64,
 production: u32,
 feeder: Sampler,
 actors: Vec<Id<Actor>>,
}

impl World
{
 pub fn get_id(&self) -> u64
 { self.id }

 pub fn tick(&mut self, hub: &mut Hub)
 {
/*  if self.oldactors.len() > 0
    {
    for a in &self.oldactors
       { actors.get_mut(*a).unwrap().feed(self.feeder.sample()); }
    }*/
  for a in &self.actors
    { hub.send(a, interface::Universe::Actor { actor: interface::Actor::Feed { id: a.target, amount:self.feeder.sample() } } ); }
 }

 pub fn drop_actor(&mut self, actor: Id<Actor>)
 {
  log::info!("World {} drop actor {:?}", self.id, actor);
  if let Some(index) = self.actors.iter().position(|x| *x == actor)
    { self.actors.remove(index); }
  self.adjust_feeder();
  log::info!("World {} drop actor, {} actors left", self.id, self.oldactors.len());
 }

 pub fn add_actor(&mut self, actor: Id<Actor>)
 {
  log::info!("World {} add actor {:?}", self.id, actor);
  self.actors.push(actor);
  self.adjust_feeder();
 }

 pub fn adjust_feeder(&mut self)
 {
  if self.actors.len() > 0
    {
    self.feeder.set( self.production / (self.actors.len() as u32) );
    }
  else
    {
    self.feeder.set(0);
    }
 }

 pub fn load(source: &mut Reader) -> Result<Self>
 {
   log::debug!("World loading");
   let mut world = World::default();
   world.id = source.u64()?;
   world.production = source.u32()?;
   source.vec(&mut world.actors);
   world.adjust_feeder();
   log::debug!("World {} loaded", world.id);
   Ok(world)
 }

 pub fn save(&self, target: &mut Writer, actors: &Depot<Actor>) -> Result<()>
 {
   target.u64(self.id)?;
   target.u32(self.production)?;
   log::debug!("World {} saving, {} actors", self.id, self.oldactors.len());
   target.save(&self.actors)?;
   Ok(())
 }
}

#[derive(Default)]
pub struct Realm
{
 worlds: Vec<DepotIndex>
}

impl Realm
{
 pub fn tick(&mut self, worlds: &mut Depot<World>, hub: &mut Hub)
 {
  for w in &self.worlds
    { worlds.get_mut(*w).unwrap().tick(hub); }
 }

 pub fn insert(&mut self, world: DepotIndex)
 {
  self.worlds.push(world);
 }

 pub fn drop_actor(&mut self, worlds: &mut Depot<World>, actor: Id<Actor>)
 {
  for w in &self.worlds
    { worlds.get_mut(*w).unwrap().drop_actor(actor); }
 }
}