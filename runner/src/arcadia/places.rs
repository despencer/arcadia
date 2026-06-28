use crate::arcadia::actors::Actor;
use crate::arcadia::depot::{Slab, SlabIndex};

pub struct Container
{
 pub members: Vec<SlabIndex>
}

impl Container
{
 pub fn new() -> Container
 {
  let actors = Vec::<SlabIndex>::new();
  Container { members: actors }
 }

 pub fn push(&mut self, actor: SlabIndex) -> ()
 {
  self.members.push(actor);
 }

 pub fn add_credits(&self, sa: &mut Slab::<Actor>) -> ()
 {
  for imember in self.members.iter()
      { sa.get_mut(*imember).unwrap().add_credits(10); };
 }
}