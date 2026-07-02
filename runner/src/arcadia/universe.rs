use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use std::io::{Result, Read, Write, Error, ErrorKind};
use std::fs::File;
use std::collections::HashMap;
use byteorder::{ReadBytesExt, WriteBytesExt, LittleEndian};
use crate::arcadia::actors::Actor;
use crate::arcadia::places::{World, Container, Realm};
use crate::arcadia::depot::{Depot,DepotIndex};
use crate::arcadia::dispatcher::{Message,Dispatcher};

#[derive(Default)]
pub struct Storage
{
 pub actors: Depot<Actor>,
 pub worlds: Depot<World>,
 pub alookup: HashMap<u64, DepotIndex>
}

#[derive(Default)]
pub struct Universe
{
 timetick: u64,
 lastseqid: u64,
 storage: Storage,
 commune: Container,
 realm: Realm,
 dispatcher: Dispatcher
}

const UNIVERSE_VERSION:u16 = 1;

impl Universe
{
 pub fn load_1<R:Read>(&mut self, source: &mut R) -> Result<()>
 {
  self.timetick = source.read_u64::<LittleEndian>()?;
  self.lastseqid = source.read_u64::<LittleEndian>()?;
  self.commune.load_1(source)?;
  let counta = source.read_u32::<LittleEndian>()? as usize;
  for _i in 0..counta
     {
     let actor = Actor::load_1(source)?; let aid = actor.id;
     let iactor = self.storage.actors.insert(actor);
     self.commune.insert(iactor); self.storage.alookup.insert(aid, iactor);
     }
  let countw = source.read_u32::<LittleEndian>()? as usize;
  for _i in 0..countw
     {
     let world = World::load_1(source, &self.storage.alookup)?; let iworld = self.storage.worlds.insert(world);
     self.realm.insert(iworld);
     }

  Ok(())
 }

 pub fn save_1<W:Write>(&mut self, target: &mut W) -> Result<()>
 {
  target.write_u64::<LittleEndian>(self.timetick)?;
  target.write_u64::<LittleEndian>(self.lastseqid)?;
  self.commune.save_1(target)?;
  target.write_u32::<LittleEndian>(self.storage.actors.len() as u32)?;
  for actor in self.storage.actors.iterdata()
      { actor.save_1(target)?; }
  target.write_u32::<LittleEndian>(self.storage.worlds.len() as u32)?;
  for world in self.storage.worlds.iterdata()
      { world.save_1(target, &self.storage.actors)?; }
  Ok(())
 }

 pub fn load(&mut self, filename: &String) -> Result<()>
 {
   let mut source = File::open(filename)?;
   let version = source.read_u16::<LittleEndian>()?;
   match version
   {
     1 => { self.load_1(&mut source) }
     _ => { return Err(Error::new(ErrorKind::InvalidData, "Unknown version")); }
   }?;

   println!("Universe {:?} loaded, {:?} actors", filename, self.storage.actors.len());
   Ok(())
 }

 pub fn save(&mut self, filename: &String) -> Result<()>
 {
  let mut target = File::create(filename)?;
  target.write_u16::<LittleEndian>(UNIVERSE_VERSION)?;
  self.save_1(&mut target)?;
  println!("Universe {:?} saved", filename);
  Ok(())
 }
}

impl Universe
{
 pub fn tick(&mut self)
 {
  self.timetick += 1;
  self.commune.tick(&mut self.storage.actors, &mut self.dispatcher);
  self.realm.tick(&mut self.storage.worlds, &mut self.storage.actors);
  while self.dispatcher.len() > 0
     {
     match self.dispatcher.get()
       {
         Message::Death {id} => self.drop_actor(id),
       }
     }
 }

 pub fn lookup_actor(&self, id: u64) -> DepotIndex
 {
  *self.storage.alookup.get(&id).unwrap()
 }

 pub fn drop_actor(&mut self, id: u64)
 {
  let iactor = self.lookup_actor(id);
  self.commune.drop_actor(iactor);
  self.realm.drop_actor(&mut self.storage.worlds, iactor);
  self.storage.alookup.remove(&id);
  self.storage.actors.remove(iactor);
 }

 pub fn run(filename: String, cancel_ticket:Arc<AtomicBool>)
 {
   let mut uni = Universe::default();
   uni.load(&filename).expect("Could not load an Universe");
   let mut start = Instant::now();

   println!("Universe starts at {:?}", uni.timetick);
   while !cancel_ticket.load(Ordering::Relaxed)
   {
     for _i in 0..100
        { uni.tick() }
     if start.elapsed().as_millis() > 750
        {
        println!("Universe at {:?}", uni.timetick);
        start = Instant::now();
        }
   }
   println!("Universe finishes at {:?}", uni.timetick);
   uni.save(&filename).expect("Could not save an Universe");
 }

}