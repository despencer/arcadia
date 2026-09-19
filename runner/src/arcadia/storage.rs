use std::collections::{VecDeque};
use std::io::{Result, Read, Write, Error, ErrorKind};
use byteorder::{ReadBytesExt, WriteBytesExt, LittleEndian};
use crate::arcadia::values::Port;

pub trait Stored
{
 fn save(&self, writer: &mut Writer) -> Result<()>;
}

pub trait Factory where Self:Sized
{
 fn load(reader: &mut Reader) -> Result<Self>;
}

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
 pub fn bool(&mut self) -> Result<bool>
 {
  Ok( (self.u8()?) != 0 )
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
 pub fn u64(&mut self) -> Result<u64>
 {
  self.source.read_u64::<LittleEndian>()
 }
 pub fn f32(&mut self) -> Result<f32>
 {
  self.source.read_f32::<LittleEndian>()
 }
 pub fn port(&mut self) -> Result<Port>
 {
  Ok( Port(self.u8()? as usize) )
 }
 pub fn count(&mut self) -> Result<u32>
 {
  self.u32()
 }
 pub fn utf8(&mut self) -> Result<String>
 {
  let size = self.u16()? as usize;
  let mut buf = vec![0u8; size];
  self.source.read_exact(&mut buf)?;
  String::from_utf8(buf).map_err(|e| Error::new(ErrorKind::InvalidData, e))
 }
 pub fn vecdeque<T:Factory>(&mut self, value: &mut VecDeque<T>) -> Result<()>
 {
  value.clear();
  for _ in 0..self.count()?
    { value.push_back( T::load(self)? ); }
  Ok(())
 }
 pub fn vec<T:Factory>(&mut self, value: &mut Vec<T>) -> Result<()>
 {
  value.clear();
  for _ in 0..self.count()?
    { value.push( T::load(self)? ); }
  Ok(())
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
 pub fn bool(&mut self, value: bool) -> Result<()>
 {
  self.u8(value as u8)
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
 pub fn u64(&mut self, value: u64) -> Result<()>
 {
  self.target.write_u64::<LittleEndian>(value)
 }
 pub fn f32(&mut self, value: f32) -> Result<()>
 {
  self.target.write_f32::<LittleEndian>(value)
 }
 pub fn port(&mut self, value: Port) -> Result<()>
 {
  self.u8(value.0 as u8)
 }
 pub fn count(&mut self, value: usize) -> Result<()>
 {
  self.u32(value as u32)
 }
 pub fn utf8(&mut self, value: &String) -> Result<()>
 {
  let buf = value.as_bytes();
  self.u16( buf.len() as u16)?;
  self.target.write_all(buf)
 }
 pub fn vecdeque<T:Stored>(&mut self, value: &VecDeque<T>) -> Result<()>
 {
  self.count(value.len())?;
  for item in value.iter()
     { item.save(self)?; }
  Ok(())
 }
 pub fn vec<T:Stored>(&mut self, value: &Vec<T>) -> Result<()>
 {
  self.count(value.len())?;
  for item in value.iter()
     { item.save(self)?; }
  Ok(())
 }
}