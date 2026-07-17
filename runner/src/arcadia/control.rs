use std::io::{Result, Read, Write};
use byteorder::{ReadBytesExt, WriteBytesExt, LittleEndian};

#[derive(Default)]
pub struct Control
{
 threshold: u32
}

impl Control
{
 pub fn load_1<R:Read>(&mut self, source: &mut R) -> Result<()>
 {
   self.threshold = source.read_u32::<LittleEndian>()?;
   Ok(())
 }

 pub fn save_1<W:Write>(&self, target: &mut W) -> Result<()>
 {
   target.write_u32::<LittleEndian>(self.threshold)?;
   Ok(())
 }
}