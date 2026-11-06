use std::io::{Result};
use crate::arcadia::storage::{Reader,Writer};
use crate::arcadia::depot::{Depot, DepotIndex};
use crate::arcadia::actors::{ActorId, Actor};
use crate::arcadia::interface::{Output, Hub, OldInterface};
use crate::arcadia::interface;

#[derive(Default)]
pub struct Commune
{
 billing: u32,
 actors: Depot<Actor>,
 channels: Vec<Output<ActorId>>,
}

impl Commune
{
 pub fn tick(&mut self, hub: &mut Hub)
 {
  for a in &self.channels
   { a.send(hub, interface::Universe::Actor { actor: interface::Actor::Bill { id: a.target, amount:self.billing } } ); }
  for a in self.actors.iterdata_mut()
      { a.tick(hub); }
 }

 pub fn load(&mut self, reader: &mut Reader) -> Result<()>
 {
  self.billing = reader.u32()?;
  self.actors.load(reader)?;
  reader.vec(&mut self.channels)
 }

 pub fn save(&self, writer: &mut Writer) -> Result<()>
 {
  writer.u32(self.billing)?;
  self.actors.save(writer)?;
  writer.vec(&self.channels)
 }
}
