use std::io::{Result, Error, ErrorKind};
use std::collections::{VecDeque, BTreeMap};
use crate::arcadia::common::Id;
use crate::arcadia::values::Seed;
use crate::arcadia::storage::{Reader, Writer, Stored, Factory};
use crate::arcadia::actors;
use crate::arcadia::places::World;
use crate::arcadia::commune::Commune;
use crate::arcadia::telemetry;
use crate::arcadia::universe;

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

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
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

impl From<Id<universe::Universe>> for Receiver { fn from(id: Id<universe::Universe>) -> Self { Receiver { party_type: PartyType::Universe, id: id.get_value() } } }
impl From<Id<Commune>> for Receiver { fn from(id: Id<Commune>) -> Self { Receiver { party_type: PartyType::Commune, id: id.get_value() } } }
impl From<Id<World>> for Receiver { fn from(id: Id<World>) -> Self { Receiver { party_type: PartyType::World, id: id.get_value() } } }
impl From<Id<Actor>> for Receiver { fn from(id: Id<Actor>) -> Self { Receiver { party_type: PartyType::Actor, id: id.get_value() } } }

impl Receiver
{
 const UNIVERSE:u8 = PartyType::Universe as u8;
 const COMMUNE:u8 = PartyType::Commune as u8;
 const WORLD:u8 = PartyType::World as u8;
 const ACTOR:u8 = PartyType::Actor as u8;

 pub fn load(&mut self, reader: &mut Reader) -> Result<()>
 {
  self.party_type = match reader.u8()?
     {
     Self::UNIVERSE => PartyType::Universe,
     Self::COMMUNE => PartyType::Commune,
     Self::WORLD => PartyType::World,
     Self::ACTOR => PartyType::Actor,
     _ => return Err(Error::new(ErrorKind::InvalidData, "Unknown party type"))
     };
  Ok(())
 }
 fn save(&self, writer: &mut Writer) -> Result<()>
 {
  writer.u8(self.party_type as u8)?;
  writer.u64(self.id)
 }
}

#[derive(Default)]
pub struct Mailbox
{
 receiver: Receiver,
 messages: VecDeque<Universe>,
}

impl Mailbox
{
 pub fn new<T>(receiver: Id<T>) -> Self where Receiver:From<Id<T>>
 {
  Mailbox { receiver: Receiver::from(receiver), messages: VecDeque::new() }
 }
}

impl Factory for Mailbox
{
 fn load(reader: &mut Reader) -> Result<Self>
 {
   let mut abox = Self::default();
   abox.receiver.load(reader)?;
   reader.vecdeque(&mut abox.messages)?;
   Ok(abox)
 }
}

impl Stored for Mailbox
{
 fn save(&self, writer: &mut Writer) -> Result<()>
 {
   self.receiver.save(writer)?;
   writer.vecdeque(&self.messages)
 }
}

pub struct Hub
{
 pub universe: Id<universe::Universe>,
 pub timetick: u64,
 lastseqid: u64,
 receivers: BTreeMap<u64, Mailbox>,
 telemetry: telemetry::Writer
}

impl Hub
{
 pub fn new(telemetry :telemetry::Writer) -> Self
 {
   Hub
    {
    universe: Id::new(0),
    lastseqid: 0,
    timetick: 0,
    receivers: BTreeMap::default(),
    telemetry: telemetry,
    }
 }

 pub fn send<T>(&mut self, receiver: Id<T>, message: Universe)
 {
  self.telemetry.write(self.timetick, &message).unwrap();
  if let Some(mailbox) = self.receivers.get_mut( &receiver.get_value() )
     {
     mailbox.messages.push_back(message);
     }
 }

 pub fn mail<T>(&mut self, receiver: Id<T>) -> Vec<Universe>
 {
  let mut ret = vec![];
  if let Some(mailbox) = self.receivers.get_mut( &receiver.get_value() )
     {
     while let Some(message) = mailbox.messages.pop_front()
         { ret.push(message); }
     }
  ret
 }

 pub fn register<T>(&mut self, receiver: Id<T>) where Receiver:From<Id<T>>
 {
  self.receivers.entry(receiver.get_value()).or_insert_with(|| Mailbox::new(receiver));
 }

 pub fn remove<T>(&mut self, receiver: Id<T>)
 {
  self.receivers.remove(&receiver.get_value());
 }

 pub fn gen_id(&mut self) -> u64
 {
  self.lastseqid += 1;
  self.lastseqid
 }

 pub fn load(&mut self, reader: &mut Reader) -> Result<()>
  {
   self.lastseqid = reader.u64()?;
   reader.btreemap(&mut self.receivers)
  }

 pub fn save(&self, writer: &mut Writer) -> Result<()>
 {
   writer.u64(self.lastseqid)?;
   writer.btreemap(&self.receivers)
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
 Make { parent: Id<actors::Actor>, home: Id<World>, seed: Seed  },
 Feed { id: Id<actors::Actor>, amount: u32 },
 Bill { id: Id<actors::Actor>, amount: u32 },
 Register { id: Id<actors::Actor> },
}

impl Actor
{
 const EMPTY: u8 = 1;
 const DROP: u8 = 2;
 const MAKE: u8 = 3;
 const FEED: u8 = 4;
 const BILL: u8 = 5;
 const REGISTER: u8 = 6;

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
        Self::MAKE => Self::Make { parent: Id::<actors::Actor>::load(reader)?, home : Id::<World>::load(reader)?, seed: Seed::load(reader)? },
        Self::FEED => Self::Feed { id: Id::<actors::Actor>::load(reader)?, amount: reader.u32()? },
        Self::BILL => Self::Bill { id: Id::<actors::Actor>::load(reader)?, amount: reader.u32()? },
        Self::REGISTER => Self::Register { id: Id::<actors::Actor>::load(reader)? },
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
             home.save(writer)?; seed.save(writer)? },
    Self::Feed {id, amount} => { writer.u8(Self::FEED)?; id.save(writer)?; writer.u32(*amount)? },
    Self::Bill {id, amount} => { writer.u8(Self::BILL)?; id.save(writer)?; writer.u32(*amount)? },
    Self::Register {id} => { writer.u8(Self::REGISTER)?; id.save(writer)? },
    }
  Ok( () )
 }
}
