use std::io::{Result, Error, ErrorKind};
use std::collections::VecDeque;
use crate::arcadia::depot::Depot;
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

pub enum Party
{
 World { id: u64 },
}

impl Party
{
 const WORLD:u8 = 1;
}

impl Factory for Party
{
 fn load(reader: &mut Reader) -> Result<Self>
 {
  let party = match reader.u8()?
        {
        Self::WORLD => Self::World { id: reader.u64()? },
        _ => return Err(Error::new(ErrorKind::InvalidData, "Unknown party"))
        };
  Ok(party)
 }
}

impl Stored for Party
{
 fn save(&self, writer: &mut Writer) -> Result<()>
 {
  match self
    {
    Self::World {id} => { writer.u8(Self::WORLD)?; writer.u64(*id)? },
    }
  Ok( () )
 }
}

pub struct Channel
{
 sender: Party,
 receiver: Party,
 messages: VecDeque<Universe>,
}

impl Factory for Channel
{
 fn load(reader: &mut Reader) -> Result<Self>
 {
  Ok( Channel { sender: Party::load(reader)?, receiver: Party::load(reader)?, messages: reader.load_vecdeque()? } )
 }
}

impl Stored for Channel
{
 fn save(&self, writer: &mut Writer) -> Result<()>
 {
  self.sender.save(writer)?;
  self.receiver.save(writer)?;
  writer.vecdeque(&self.messages)?;
  Ok( () )
 }
}

#[derive(Default)]
pub struct Hub
{
 channels: Depot<Channel>,
}

impl Hub
{
 pub fn load(&mut self, reader: &mut Reader) -> Result<()>
  {
   self.channels.load(reader)
  }
 pub fn save(&self, writer: &mut Writer) -> Result<()>
 {
  self.channels.save(writer)
 }
}

pub enum Universe
{
 Actor { actor: Actor },
}

impl Universe
{
 const ACTOR: u8 = 1;

 fn version() -> u8 { 1 }
}

impl Factory for Universe
{
 fn load(reader: &mut Reader) -> Result<Self>
 {
  if reader.u8()? > Self::version()
         { return Err(Error::new(ErrorKind::InvalidData, "Unknown universe interface version")); }

  let utype = reader.u8()?;
  let uni = match utype
        {
        Self::ACTOR => Self::Actor { actor: Actor::load(reader)? },
        _ => return Err(Error::new(ErrorKind::InvalidData, "Unknown universe interface"))
        };
  Ok(uni)
 }
}

impl Stored for Universe
{
 fn save(&self, writer: &mut Writer) -> Result<()>
 {
  writer.u8(Self::version())?;
  match self
    {
    Self::Actor {actor} => { writer.u8(Self::ACTOR)?; actor.save(writer)? },
    }
  Ok( () )
 }
}

#[derive(Default)]
pub enum Actor
{
 #[default]
 Empty,
 Drop { id: u64 },
 Make { parent: u64, home: u64, seed: Seed  },
 Feed { id: u64, amount: u32 },
 Bill { id: u64, amount: u32 }
}

impl Actor
{
 const EMPTY: u8 = 1;
 const DROP: u8 = 2;
 const MAKE: u8 = 3;
 const FEED: u8 = 4;
 const BILL: u8 = 5;

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
        Self::FEED => Self::Feed { id: reader.u64()?, amount: reader.u32()? },
        Self::BILL => Self::Feed { id: reader.u64()?, amount: reader.u32()? },
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
             writer.u64(*home)?; seed.save(writer)? },
    Self::Feed {id, amount} => { writer.u8(Self::FEED)?; writer.u64(*id)?; writer.u32(*amount)? },
    Self::Bill {id, amount} => { writer.u8(Self::BILL)?; writer.u64(*id)?; writer.u32(*amount)? },
    }
  Ok( () )
 }
}
