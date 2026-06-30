use std::io::{Result, Read, Write};
use byteorder::{ReadBytesExt, WriteBytesExt, LittleEndian};

#[derive(Default)]
pub struct Actor
{
 pub id: u64,
 credits: u32,
}

impl Actor
{
 pub fn tick(&mut self)
 {
 }

 pub fn billing(&mut self, amount: u32)
 {
  self.credits -= amount;
 }

 pub fn feed(&mut self, amount: u32)
 {
  self.credits += amount;
 }

 pub fn load_1<R:Read>(source: &mut R) -> Result<Self>
 {
   let mut actor = Actor::default();
   actor.id = source.read_u64::<LittleEndian>()?;
   actor.credits = source.read_u32::<LittleEndian>()?;
   Ok(actor)
 }

 pub fn save_1<W:Write>(&self, target: &mut W) -> Result<()>
 {
   target.write_u64::<LittleEndian>(self.id)?;
   target.write_u32::<LittleEndian>(self.credits)?;
   Ok(())
 }
}