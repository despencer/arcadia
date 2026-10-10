use std::io::{Result, Error, ErrorKind};
use std::collections::VecDeque;
use crate::arcadia::values::Seed;
use crate::arcadia::storage::{Reader,Writer,Stored,Factory};

#[derive(Default)]
pub struct Interface<T>
{
 messages: VecDeque<T>
}

impl<T> Interface<T>
{
 pub fn put(&mut self, message: T)
 {
  self.messages.push_back(message);
 }

 pub fn len(&self) -> usize
 {
  self.messages.len()
 }

 pub fn get(&mut self) -> T
 {
  self.messages.pop_front().unwrap()
 }
}

impl<T:Factory> Interface<T>
{
 pub fn load(&mut self, reader: &mut Reader) -> Result<()>
 {
  reader.vecdeque(&mut self.messages)
 }
}

impl<T:Stored> Interface<T>
{
 pub fn save(&self, writer: &mut Writer) -> Result<()>
 {
  writer.vecdeque(&self.messages)
 }
}

#[derive(Default)]
pub enum Actor
{
 #[default]
 Empty,
 Drop { id: u64 },
 Make { parent: u64, home: u64, seed: Seed  }
}

impl Actor
{
 const EMPTY: u8 = 1;
 const DROP: u8 = 2;
 const MAKE: u8 = 3;

 fn version() -> u8 { 1 }
}

impl Factory for Actor
{
 fn load(reader: &mut Reader) -> Result<Self>
 {
  if reader.u8()? > Self::version()
         { return Err(Error::new(ErrorKind::InvalidData, "Unknown actor interface version")); }

  let utype = reader.u8()?;
  let act = match utype
        {
        Self::EMPTY => Self::Empty { },
        Self::DROP => Self::Drop { id: reader.u64()? },
        Self::MAKE => Self::Make { parent: reader.u64()?, home : reader.u64()?, seed: Seed::load(reader)? },
        _ => return Err(Error::new(ErrorKind::InvalidData, "Unknown actor interface"))
        };
  Ok(act)
 }
}

impl Stored for Actor
{
 fn save(&self, writer: &mut Writer) -> Result<()>
 {
  writer.u8(Self::version())?;
  match self
    {
    Self::Empty => { writer.u8(Self::EMPTY)? },
    Self::Drop {id} => { writer.u8(Self::DROP)?; writer.u64(*id)? },
    Self::Make {parent, home, seed} =>
           { writer.u8(Self::MAKE)?; writer.u64(*parent)?;
             writer.u64(*home)?; seed.save(writer)? }
    }
  Ok( () )
 }
}

