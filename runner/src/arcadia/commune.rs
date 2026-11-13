use std::io::{Result};
use crate::arcadia::common::Id;
use crate::arcadia::storage::{Reader, Writer, Factory, Stored};
use crate::arcadia::actors::Actor;
use crate::arcadia::interface::Hub;
use crate::arcadia::interface;

#[derive(Default, Debug)]
pub struct Commune
{
 id: Id<Commune>,
 billing: u32,
 actors: Vec<Actor>,
}

impl Commune
{
 pub fn tick(&mut self, hub: &mut Hub)
 {
  for event in hub.mail(self.id)
    {
    match event
      {
      interface::Universe::Actor {actor} =>
              match actor
                  {
                  interface::Actor::Make {home, mut seed, ..} =>
                      {
                      let id = Id::<Actor>::new(hub.gen_id());
                      let actor = Actor::new(id, home, &mut seed);
                      self.actors.push(actor);
                      hub.send(home, interface::Universe::Actor { actor: interface::Actor::Register { id: id } } );
                      },
                  interface::Actor::Drop {id} =>
                      {
                      let index = self.actors.iter().position(|x| x.get_id() == id).unwrap();
                      self.actors.remove(index);
                      hub.remove(id);
                      hub.send(hub.universe, interface::Universe::Actor { actor: interface::Actor::Drop { id: id } } );
                      },
                  _ => {}
                  }
      }
    }
  for a in &self.actors
   { hub.send(a.get_id(), interface::Universe::Actor { actor: interface::Actor::Bill { id: a.get_id(), amount:self.billing } } ); }
  for a in self.actors.iter_mut()
      { a.tick(hub); }
 }

 pub fn load(&mut self, reader: &mut Reader) -> Result<()>
 {
  self.id = Id::<Commune>::load(reader)?;
  self.billing = reader.u32()?;
  reader.vec(&mut self.actors)?;
  Ok( () )
 }

 pub fn save(&self, writer: &mut Writer) -> Result<()>
 {
  self.id.save(writer)?;
  writer.u32(self.billing)?;
  writer.vec(&self.actors)?;
  Ok( () )
 }

}
