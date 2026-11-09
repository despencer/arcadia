use std::io::{Result};
use crate::arcadia::common::Id;
use crate::arcadia::storage::{Reader,Writer};
use crate::arcadia::depot::{Depot, DepotIndex};
use crate::arcadia::actors::Actor;
use crate::arcadia::interface::Hub;
use crate::arcadia::interface;
use crate::arcadia::values::Seed;

#[derive(Default)]
pub struct Commune
{
 id: Id<Commune>,
 billing: u32,
 actors: Depot<Actor>,
}

impl Commune
{
 pub fn tick(&mut self, hub: &mut Hub)
 {
  for a in &self.channels
   { hub.send(a.get_id(), interface::Universe::Actor { actor: interface::Actor::Bill { id: a.target, amount:self.billing } } ); }
  for a in self.actors.iterdata_mut()
      { a.tick(hub); }
 }

 pub fn load(&mut self, reader: &mut Reader) -> Result<()>
 {
  self.id = Id::Commune::load(reader)?;
  self.billing = reader.u32()?;
  self.actors.load(reader)?;
 }

 pub fn save(&self, writer: &mut Writer) -> Result<()>
 {
  self.id.save(writer)?;
  writer.u32(self.billing)?;
  self.actors.save(writer)?;
 }

 pub fn make_actor(&mut self, hub: &mut Hub, id: Id<Actor>, home: u64, seed: &mut Seed)
 {
  let mut actor = Actor::new(id, home, seed);
  hub.register(id);
  self.actors.insert(actor);
 }

 pub fn drop_actor(&mut self, hub: &mut Hub, id: Id<Actor>)
 {
   self.actors.remove( self.actors.iter().position(|x| x.id == id).unwrap() );
   hub.remove(id);
 }
}
