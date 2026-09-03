use std::io::{Result, Read};
use byteorder::{ReadBytesExt, LittleEndian};

pub struct Reader<'a>
{
 source: &'a mut dyn Read
}

impl<'a> Reader<'a>
{
 pub fn new(source: &'a mut dyn Read) -> Self
 {
  Reader { source }
 }

 pub fn u8(&mut self) -> Result<u8>
 {
  self.source.read_u8()
 }
 pub fn u16(&mut self) -> Result<u16>
 {
  self.source.read_u16::<LittleEndian>()
 }
 pub fn u32(&mut self) -> Result<u32>
 {
  self.source.read_u32::<LittleEndian>()
 }

 pub fn f32(&mut self) -> Result<f32>
 {
  self.source.read_f32::<LittleEndian>()
 }

 pub fn count(&mut self) -> Result<u32>
 {
  self.u32()
 }
}