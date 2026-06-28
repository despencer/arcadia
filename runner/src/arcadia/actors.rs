use std::io::{Result, Read, Write};
use byteorder::{ReadBytesExt, WriteBytesExt, LittleEndian};
use crate::arcadia::universe::{Load1,Save1};

pub struct Actor
{
 id: u64,
 credits: u32,
}

impl<R> Load1<R> for Actor where R:Read, Self:Sized
{
 fn load_1(source: &mut R) -> Result<Self>
 {
   let id = source.read_u64::<LittleEndian>()?;
   let actor = Actor { id: id, credits: 0 };
   Ok(actor)
 }
}

impl<W> Save1<W> for Actor where W:Write, Self:Sized
{
 fn save_1(&self, target: &mut W) -> Result<()>
 {
  target.write_u64::<LittleEndian>(self.id)?;
  Ok(())
 }

}

impl Actor
{
 pub fn add_credits(&mut self, amount: u32) -> ()
 {
  self.credits += amount;
 }
}