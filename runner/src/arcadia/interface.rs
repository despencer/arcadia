use std::io::{Result, Error, ErrorKind};
use std::collections::{VecDeque, BTreeMap};
use crate::arcadia::common::Id;
use crate::arcadia::values::Seed;
use crate::arcadia::storage::{Reader, Writer, Stored, Factory};
use crate::arcadia::actors;

#[derive(Default, Debug)]
pub struct OldInterface<T>
{
 messages: VecDeque<T>
}

impl<T> OldInterface<T>
{
 pub fn put(&mut self, message: T)
 {
  self.messages.push_back(message);
 }

 pub fn len(&self) -> usize
 {
  self.messages.len()
 }

 pub fn get(&mut self) -> Option<T>
 {
  self.messages.pop_front()
 }
}

impl<T:Factory> OldInterface<T>
{
 pub fn load(&mut self, reader: &mut Reader) -> Result<()>
 {
  reader.vecdeque(&mut self.messages)
 }
}

impl<T:Stored> OldInterface<T>
{
 pub fn save(&self, writer: &mut Writer) -> Result<()>
 {
  writer.vecdeque(&self.messages)
 }
}

/*pub enum Party
{
 World { id: Id<World> },
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

impl Channel
{
 pub fn send(&mut self, message: Universe)
  { self.messages.push_back(message); }
 pub fn recv(&mut self) -> Option<Universe>
  { self.messages.pop_front() }
 pub fn len(&self) -> usize
  { self.messages.len() }
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
 pub fn send(&mut self, channel: DepotIndex, message: Universe)
 {
  self.channels.get_mut(channel).unwrap().send(message);
 }
}

pub struct Output<T>
{
 pub target : Id<T>,
 channel: DepotIndex
}

impl <T> Output<T>
{
 pub fn send(&self, hub: &mut Hub, message: Universe)
 {
  hub.send(self.channel, message);
 }
}

impl<T> Factory for Output<T> where T:Factory
{
 fn load(reader: &mut Reader) -> Result<Self>
 {
   Ok( Output { target: T::load(reader)?, channel: DepotIndex::load(reader)? } )
 }
}

impl<T> Stored for Output<T> where T:Stored
{
 fn save(&self, writer: &mut Writer) -> Result<()>
  { self.target.save(writer)?; self.channel.save(writer) }
}

#[derive(Default)]
pub struct Input
{
 channels: Vec<DepotIndex>,
}

impl Input
{
 pub fn recv(&self, hub: &mut Hub) -> Option<Universe>
 {
  for c in &self.channels
    {
    let channel = hub.channels.get_mut(*c).unwrap();
    if channel.len() > 0
       { return channel.recv(); }
    }
  None
 }
}*/

#[derive(Default)]
pub enum PartyType
{
 #[default]
 Unknown = 0,
 Universe = 1,
 Commune = 2,
 World = 3,
 Actor = 4,
}

#[derive(Default)]
pub struct Receiver
{
 party_type: PartyType,
 id: u64,
}

#[derive(Default)]
pub struct Mailbox
{
 receiver: Receiver,
 messages: VecDeque<Universe>,
}

impl Mailbox
{
 pub fn recv(&mut self) -> Option<Universe>
 {
  self.messages.pop_front()
 }
}

impl Factory for Mailbox
{
 fn load(_reader: &mut Reader) -> Result<Self>
 {
   todo!()
 }
}

impl Stored for Mailbox
{
 fn save(&self, _writer: &mut Writer) -> Result<()>
 {
   todo!()
 }
}

#[derive(Default)]
pub struct Hub
{
 universe: Id<Universe>,
 lastseqid: u64,
 receivers: BTreeMap<u64, Receiver>,
}

impl Hub
{
 pub fn send<T>(&mut self, _receiver: Id<T>, _message: Universe)
   { todo!() }
 pub fn mail<T>(&mut self, _receiver: Id<T>) -> Vec<Universe>
   { todo!() }
 pub fn register<T>(&mut self, _receiver: Id<T>)
   { todo!() }
 pub fn remove<T>(&mut self, _receiver: Id<T>)
   { todo!() }
 pub fn gen_id(&mut self) -> u64
 {
  self.lastseqid += 1;
  self.lastseqid
 }
 pub fn load(&mut self, _reader: &mut Reader) -> Result<()>
  {
   todo!()
  }
 pub fn save(&self, _writer: &mut Writer) -> Result<()>
 {
   todo!()
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
 Drop { id: Id<actors::Actor> },
 Make { parent: Id<actors::Actor>, home: u64, seed: Seed  },
 Feed { id: Id<actors::Actor>, amount: u32 },
 Bill { id: Id<actors::Actor>, amount: u32 }
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
        Self::DROP => Self::Drop { id: Id::<actors::Actor>::load(reader)? },
        Self::MAKE => Self::Make { parent: Id::<actors::Actor>::load(reader)?, home : reader.u64()?, seed: Seed::load(reader)? },
        Self::FEED => Self::Feed { id: Id::<actors::Actor>::load(reader)?, amount: reader.u32()? },
        Self::BILL => Self::Bill { id: Id::<actors::Actor>::load(reader)?, amount: reader.u32()? },
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
    Self::Drop {id} => { writer.u8(Self::DROP)?; id.save(writer)? },
    Self::Make {parent, home, seed} =>
           { writer.u8(Self::MAKE)?; parent.save(writer)?;
             writer.u64(*home)?; seed.save(writer)? },
    Self::Feed {id, amount} => { writer.u8(Self::FEED)?; id.save(writer)?; writer.u32(*amount)? },
    Self::Bill {id, amount} => { writer.u8(Self::BILL)?; id.save(writer)?; writer.u32(*amount)? },
    }
  Ok( () )
 }
}
