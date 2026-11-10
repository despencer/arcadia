use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use std::io::{Result, Error, ErrorKind};
use std::fs::File;
use std::collections::HashMap;
use std::path::Path;
use crate::arcadia::storage::{Reader, Writer};
use crate::arcadia::common::Id;
use crate::arcadia::actors::Actor;
use crate::arcadia::commune::Commune;
use crate::arcadia::places::{World, Realm};
use crate::arcadia::depot::{Depot, DepotIndex};
use crate::arcadia::interface;
use crate::arcadia::interface::Hub;
use crate::arcadia::values::Seed;
use crate::arcadia::telemetry;

#[derive(Default)]
pub struct Storage
{
 pub worlds: Depot<World>,
 pub wlookup: HashMap<u64, DepotIndex>
}

pub struct Universe
{
 id: Id<Universe>,
 timetick: u64,
 storage: Storage,
 commune: Commune,
 realm: Realm,
 communications: Hub,
 telemetry: telemetry::Writer
}

impl Universe
{
 fn version(&self) -> u16 { 1 }

 pub fn load_1(&mut self, reader: &mut Reader) -> Result<()>
 {
  log::info!("Universe loading started");
  self.timetick = reader.u64()?;
  log::info!("Universe loading finished, timetick={}, seqid={}", self.timetick, self.lastseqid);
  self.commune.load(reader)?;
/*  let counta = reader.count()?;
  log::info!("Universe reading {} actors", counta);
  for _i in 0..counta
     {
     let actor = Actor::load(reader)?; let aid = actor.get_id();
     let iactor = self.storage.actors.insert(actor);
     self.commune.insert(iactor); self.storage.alookup.insert(aid, iactor);
     }*/
  let countw = reader.count()?;
  log::info!("Universe reading {} worlds", countw);
  for _i in 0..countw
     {
     let world = World::load(reader)?; let wid = world.get_id();
     let iworld = self.storage.worlds.insert(world);
     self.realm.insert(iworld); self.storage.wlookup.insert(wid, iworld);
     }
  log::info!("Universe reading comms");
  self.communications.load(reader)?;
  log::info!("Universe loading finished");
  Ok(())
 }

 pub fn save(&self, writer: &mut Writer) -> Result<()>
 {
  writer.u64(self.timetick)?;
  self.commune.save( writer)?;
/*  log::info!("Universe saving {} actors", self.storage.actors.len());
  writer.count(self.storage.actors.len())?;
  for actor in self.storage.actors.iterdata()
      { actor.save(writer)?; }*/
  writer.count(self.storage.worlds.len())?;
  for world in self.storage.worlds.iterdata()
      { world.save(writer)?; }
  self.communications.save(writer)?;
  Ok(())
 }

 pub fn load(&mut self, filename: &String) -> Result<()>
 {
   let mut source = File::open(filename)?;
   let mut reader = Reader::new(&mut source);
   let version = reader.u16()?;
   match version
   {
     1 => { self.load_1(&mut reader) }
     _ => { return Err(Error::new(ErrorKind::InvalidData, "Unknown universe version")); }
   }?;
   println!("Universe {:?} loaded", filename);
   Ok(())
 }

 pub fn savefile(&mut self, filename: &String) -> Result<()>
 {
  let mut target = File::create(filename)?;
  let mut writer = Writer::new(&mut target);
  writer.u16(self.version())?;
  self.save(&mut writer)?;
  println!("Universe {:?} saved", filename);
  Ok(())
 }
}

impl Universe
{
 pub fn tick(&mut self)
 {
  self.timetick += 1;
  self.commune.tick(&mut self.communications);
  self.realm.tick(&mut self.storage.worlds, &mut self.communications);

  for event in self.communications.mail(self.id)
    {
    match event
      {
        _ => {}
      }
    }
 }

/* pub fn lookup_actor(&self, id: ActorId) -> DepotIndex
 {
  *self.storage.alookup.get(&id).unwrap()
 }*/

 pub fn lookup_world(&self, id: u64) -> DepotIndex
 {
  *self.storage.wlookup.get(&id).unwrap()
 }

/* pub fn make_actor(&mut self, parent: Id<Actor>, home: u64, seed: &mut Seed)
 {
  log::info!("New actor request from {:?} with {} credits", parent, seed.credits);
  let iactor : Id<Actor> = Id::new(self.gen_id());
  self.commune.make_actor(&mut self.communications, iactor, home, seed);
  self.storage.worlds.get_mut(self.lookup_world(home)).unwrap().add_actor(iactor);
  self.telemetry.write(self.timetick, telemetry::Message::Birth { id: iactor, parent, home } ).unwrap();
 }

 pub fn drop_actor(&mut self, id: Id<Actor>)
 {
  log::info!("Drop actor {:?} request", id);
  self.commune.drop_actor(&mut self.communications, id);
  self.realm.drop_actor(&mut self.storage.worlds, id);
  self.telemetry.write(self.timetick, telemetry::Message::Death { id } ).unwrap();
 }*/

 pub fn run(filename: String, cancel_ticket:Arc<AtomicBool>)
 {
   let mut uni = Universe { id: Id::<Universe>::new(0),
                            timetick: 0, lastseqid: 0, storage: Storage::default(), commune: Commune::default(),
                            communications: Hub::default(),
                            realm: Realm::default(),
                            telemetry: telemetry::Writer::new(Path::new(&filename).with_extension("history").to_str().unwrap().to_owned()).unwrap()  };
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
   uni.savefile(&filename).expect("Could not save an Universe");
 }

}