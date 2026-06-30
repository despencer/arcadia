use std::io::{Result, Read, Write, Error, ErrorKind};
use std::collections::HashMap;
use byteorder::{ReadBytesExt, WriteBytesExt, LittleEndian};
use crate::arcadia::depot::{Depot,DepotIndex};
use crate::arcadia::actors::Actor;

#[derive(Default)]
pub struct Container
{
 billing: u32,
 actors: Vec<DepotIndex>
}

#[derive(Default)]
pub struct World
{
 id: u64,
 production: u32,
 actors: Vec<DepotIndex>
}

#[derive(Default)]
pub struct Realm
{
 worlds: Vec<DepotIndex>
}

impl Container
{
 pub fn tick(&mut self, actors: &mut Depot<Actor>)
 {
  for a in &self.actors
    {
    actors.get_mut(*a).unwrap().billing(self.billing);
    actors.get_mut(*a).unwrap().tick();
    }
 }

 pub fn insert(&mut self, actor: DepotIndex)
 {
  self.actors.push(actor);
 }

 pub fn load_1<R:Read>(&mut self, source: &mut R) -> Result<()>
 {
  self.billing = source.read_u32::<LittleEndian>()?;
  Ok(())
 }

 pub fn save_1<W:Write>(&mut self, target: &mut W) -> Result<()>
 {
  target.write_u32::<LittleEndian>(self.billing)?;
  Ok(())
 }

}

impl World
{
 pub fn tick(&mut self, actors: &mut Depot<Actor>)
 {
  for a in &self.actors
    { actors.get_mut(*a).unwrap().feed(self.production); }
 }

 pub fn load_1<R:Read>(source: &mut R, alookup: &HashMap::<u64, DepotIndex>) -> Result<Self>
 {
   let mut world = World::default();
   world.id = source.read_u64::<LittleEndian>()?;
   world.production = source.read_u32::<LittleEndian>()?;
   let counta = source.read_u32::<LittleEndian>()? as usize;
   for _i in 0..counta
     {
     let aid = source.read_u64::<LittleEndian>()?;
     if let Some(iactor) = alookup.get(&aid)
         { world.actors.push(*iactor); }
     else
         { return Err(Error::new(ErrorKind::InvalidData, "Actor not found")); }
     }
   Ok(world)
 }

 pub fn save_1<W:Write>(&self, target: &mut W, actors: &Depot<Actor>) -> Result<()>
 {
   target.write_u64::<LittleEndian>(self.id)?;
   target.write_u32::<LittleEndian>(self.production)?;
   target.write_u32::<LittleEndian>(self.actors.len() as u32)?;
   for iactor in self.actors.iter()
      { target.write_u64::<LittleEndian>(actors.get(*iactor).unwrap().id)?; }
   Ok(())
 }
}

impl Realm
{
 pub fn tick(&mut self, worlds: &mut Depot<World>, actors: &mut Depot<Actor>)
 {
  for w in &self.worlds
    { worlds.get_mut(*w).unwrap().tick(actors); }
 }

 pub fn insert(&mut self, world: DepotIndex)
 {
  self.worlds.push(world);
 }

}