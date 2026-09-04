use std::io::{Result, Read, Write};
use byteorder::{ReadBytesExt, WriteBytesExt, LittleEndian};

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

pub struct Writer<'a>
{
 target: &'a mut dyn Write
}

impl<'a> Writer<'a>
{
 pub fn new(target: &'a mut dyn Write) -> Self
 {
  Writer { target }
 }

 pub fn u8(&mut self, value: u8) -> Result<()>
 {
  self.target.write_u8(value)
 }
 pub fn u16(&mut self, value: u16) -> Result<()>
 {
  self.target.write_u16::<LittleEndian>(value)
 }
 pub fn u32(&mut self, value: u32) -> Result<()>
 {
  self.target.write_u32::<LittleEndian>(value)
 }
 pub fn f32(&mut self, value: f32) -> Result<()>
 {
  self.target.write_f32::<LittleEndian>(value)
 }
 pub fn count(&mut self, value: usize) -> Result<()>
 {
  self.u32(value as u32)
 }
}