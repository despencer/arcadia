use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use std::io::{Result, Error, ErrorKind};
use std::fs::File;
use std::path::Path;
use crate::arcadia::storage::{Reader, Writer, Factory, Stored};
use crate::arcadia::common::Id;
use crate::arcadia::commune::Commune;
use crate::arcadia::places::Realm;
use crate::arcadia::interface;
use crate::arcadia::interface::Hub;
use crate::arcadia::telemetry;

pub struct Universe
{
 id: Id<Universe>,
 timetick: u64,
 commune: Commune,
 realm: Realm,
 communications: Hub,
}

impl Universe
{
 fn version(&self) -> u16 { 1 }

 pub fn load_1(&mut self, reader: &mut Reader) -> Result<()>
 {
  log::info!("Universe loading started");
  self.id = Id::load(reader)?;
  self.timetick = reader.u64()?;
  self.commune.load(reader)?;
  self.realm.load(reader)?;
  self.communications.load(reader)?;
  self.communications.universe = self.id;
  self.communications.timetick = self.timetick;
  log::info!("Universe loading finished, timetick={}", self.timetick);
  Ok(())
 }

 pub fn save(&self, writer: &mut Writer) -> Result<()>
 {
  self.id.save(writer)?;
  writer.u64(self.timetick)?;
  self.commune.save( writer)?;
  self.realm.save( writer)?;
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
  self.communications.timetick = self.timetick;
  self.commune.tick(&mut self.communications);
  self.realm.tick(&mut self.communications);

  for event in self.communications.mail(self.id)
    {
    match event
      {
      interface::Universe::Actor {actor} =>
              match actor
                  {
                  interface::Actor::Drop {id} => { self.realm.drop_actor(id); },
                  _ => {}
                  }
      }
    }
 }

 pub fn run(filename: String, cancel_ticket:Arc<AtomicBool>)
 {
   let mut uni = Universe { id: Id::<Universe>::new(0),
                            timetick: 0, commune: Commune::default(),
                            communications: Hub::new( telemetry::Writer::new(Path::new(&filename).with_extension("history").to_str().unwrap().to_owned()).unwrap()),
                            realm: Realm::default(), };
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