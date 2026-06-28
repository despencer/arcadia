use crate::arcadia::actors::Actor;
use crate::arcadia::depot::{Depot, DepotIndex};

pub struct Container
{
 pub members: Vec<DepotIndex>
}

impl Container
{
 pub fn new() -> Container
 {
  let actors = Vec::<DepotIndex>::new();
  Container { members: actors }
 }

 pub fn push(&mut self, actor: DepotIndex) -> ()
 {
  self.members.push(actor);
 }

 pub fn add_credits(&self, sa: &mut Depot::<Actor>) -> ()
 {
  for imember in self.members.iter()
      { sa.get_mut(*imember).unwrap().add_credits(10); };
 }
}