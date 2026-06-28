use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use std::io::{Result, Read, Write, Error, ErrorKind};
use std::fs::File;
use byteorder::{ReadBytesExt, WriteBytesExt, LittleEndian};
use crate::arcadia::actors::Actor;
use crate::arcadia::places::Container;
use crate::arcadia::depot::Slab;

pub trait Load1<R> where R:Read, Self:Sized
{
 fn load_1(source: &mut R) -> Result<Self>;
}

pub trait Save1<W> where W:Write, Self:Sized
{
 fn save_1(&self, target: &mut W) -> Result<()>;
}

pub struct Universe
{
 timetick: u64,
 lastseqid: u64,
 actors: Slab<Actor>,
 defplace: Container
}

const UNIVERSE_VERSION:u16 = 1;

impl Universe
{
 pub fn load_vector_1<R, T>(source: &mut R) -> Result<Slab<T>> where R:Read, T:Load1<R>
 {
  let ucount = source.read_u32::<LittleEndian>()?;
  let count = ucount as usize;
  let mut result = Slab::<T>::new();
  for _i in 0..count
     {
     let item = T::load_1(source)?;
     result.insert(item);
     }
  Ok(result)
 }
 pub fn save_vector_1<W, T>(data: &Slab<T>, target: &mut W) -> Result<()> where W:Write, T:Save1<W>
 {
  let count = data.len() as u32;
  target.write_u32::<LittleEndian>(count)?;

  for (_, item) in data.all().enumerate()
      { item.save_1(target)?; }
  Ok(())
 }
}

impl Universe
{
 pub fn move_actors(uni: &mut Universe) -> ()
 {
  for (_, actor) in uni.actors.iter().enumerate()
      { uni.defplace.push(actor); };
 }
}

impl Universe
{
 pub fn save_1<W>(&self, target: &mut W) -> Result<()> where W:Write
 {
  target.write_u64::<LittleEndian>(self.timetick)?;
  target.write_u64::<LittleEndian>(self.lastseqid)?;
  Universe::save_vector_1::<W, Actor>(&self.actors, target)?;
  Ok(())
 }

 pub fn load_1<R>(source: &mut R) -> Result<Self> where R:Read
 {
   let tick = source.read_u64::<LittleEndian>()?;
   let seqid = source.read_u64::<LittleEndian>()?;
   let actors = Universe::load_vector_1::<R, Actor>(source)?;
   let defplace = Container::new();
   let mut universe = Universe { timetick: tick, lastseqid: seqid, actors: actors, defplace:defplace };
   { let uni = &mut universe; Universe::move_actors(uni); }
   Ok(universe)
 }

 pub fn load(filename: &String) -> Result<Self>
 {
   let mut source = File::open(filename)?;
   let version = source.read_u16::<LittleEndian>()?;
   let universe = match version
   {
     1 => { Universe::load_1(&mut source) }
     _ => { return Err(Error::new(ErrorKind::InvalidData, "Unknown version")); }
   }?;

   println!("Universe {:?} loaded, {:?} actors", filename, universe.actors.len());
   Ok(universe)
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
 pub fn run(filename: String, cancel_ticket:Arc<AtomicBool>)
 {
   let mut uni = Universe::load(&filename).expect("Could not load an Universe");
   let mut start = Instant::now();

   println!("Universe starts at {:?}", uni.timetick);
   while !cancel_ticket.load(Ordering::Relaxed)
   {
     for _i in 0..100
        { uni.step() }
     if start.elapsed().as_millis() > 750
        {
        println!("Universe at {:?}", uni.timetick);
        start = Instant::now();
        }
   }
   println!("Universe finishes at {:?}", uni.timetick);
   uni.save(&filename).expect("Could not save an Universe");
 }

 pub fn step(&mut self)
 {
   self.timetick += 1;
   self.defplace.add_credits(&mut self.actors);
 }

}