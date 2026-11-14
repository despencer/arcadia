use std::io::{Result};
use crate::arcadia::common::Id;
use crate::arcadia::storage::{Reader, Writer, Factory, Stored};
use crate::arcadia::actors::Actor;
use crate::arcadia::interface::Hub;
use crate::arcadia::interface;
use crate::arcadia::control::Sampler;

#[derive(Default)]
pub struct World
{
 id: Id<World>,
 production: u32,
 feeder: Sampler,
 actors: Vec<Id<Actor>>,
}

impl World
{
 pub fn get_id(&self) -> Id<World>
 { self.id }

 pub fn tick(&mut self, hub: &mut Hub)
 {
  for event in hub.mail(self.id)
    {
    match event
      {
      interface::Universe::Actor {actor} =>
              match actor
                  {
                  interface::Actor::Register {id} =>
                      {
                      self.actors.push(id);
                      self.adjust_feeder();
                      log::info!("World {:?} add actor {:?}", self.id, id);
                      },
                  _ => {}
                  }
      }
    }
  for a in &self.actors
    { hub.send(*a, interface::Universe::Actor { actor: interface::Actor::Feed { id: *a, amount:self.feeder.sample() } } ); }
 }

 pub fn drop_actor(&mut self, actor: Id<Actor>)
 {
  log::info!("World {:?} drop actor {:?}", self.id, actor);
  if let Some(index) = self.actors.iter().position(|x| *x == actor)
    { self.actors.remove(index); }
  self.adjust_feeder();
  log::info!("World {:?} drop actor, {} actors left", self.id, self.actors.len());
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
}

impl Factory for World
{
 fn load(source: &mut Reader) -> Result<Self>
 {
   log::debug!("World loading");
   let mut world = World::default();
   world.id = Id::<World>::load(source)?;
   world.production = source.u32()?;
   source.vec(&mut world.actors);
   world.adjust_feeder();
   log::debug!("World {:?} loaded", world.id);
   Ok(world)
 }
}

impl Stored for World
{
 fn save(&self, target: &mut Writer) -> Result<()>
 {
   self.id.save(target)?;
   target.u32(self.production)?;
   log::debug!("World {:?} saving, {} actors", self.id, self.actors.len());
   target.vec(&self.actors)?;
   Ok(())
 }
}

#[derive(Default)]
pub struct Realm
{
 id: Id<Realm>,
 worlds: Vec<World>,
}

impl Realm
{
 pub fn tick(&mut self, hub: &mut Hub)
 {
  for event in hub.mail(self.id)
    {
    match event
      {
       _ => {}
      }
    }
  for w in self.worlds.iter_mut()
      { w.tick(hub); }
 }

 pub fn drop_actor(&mut self, id: Id<Actor>)
 {
  for w in self.worlds.iter_mut()
      { w.drop_actor(id); }
 }

 pub fn load(&mut self, reader: &mut Reader) -> Result<()>
 {
  self.id = Id::<Realm>::load(reader)?;
  reader.vec(&mut self.worlds)?;
  Ok( () )
 }

 pub fn save(&self, writer: &mut Writer) -> Result<()>
 {
  self.id.save(writer)?;
  writer.vec(&self.worlds)?;
  Ok( () )
 }

}