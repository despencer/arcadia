use crate::arcadia::actors::Actor;
use crate::arcadia::depot::{Depot, DepotIndex};
use crate::arcadia::universe::{Universe, Load1, Save1};

pub struct World
{
 production: u32,
 actors: Vec<DepotIndex>
}

impl World
{
 pub fn new() -> World
 {
  let actors = Vec::<DepotIndex>::new();
  World { production:0, actors: actors }
 }

 pub fn put(&mut self, actor: DepotIndex) -> ()
 {
  self.actors.push(actor);
 }

 pub fn step(&mut self, actors: &mut Depot<Actor>) -> ()
 {
  for iactor in self.actors.iter()
     { actors.get_mut(*iactor).unwrap().add_credits(self.production); };
 }

}