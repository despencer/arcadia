use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use std::io::{Result, Read, Write, Error, ErrorKind};
use std::fs::File;
use byteorder::{ReadBytesExt, WriteBytesExt, LittleEndian};
mod actor;
mod places;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct SlabIndex(usize);

pub struct Slab<T: Sized>
{
    data: Vec<Option<T>>,
    removed_indexes: Vec<SlabIndex>,
}

impl<T: Sized> Slab<T>
{
    pub fn new() -> Self
        { Self { data: Vec::new(), removed_indexes: Default::default(), } }

    pub fn insert(&mut self, item: T) -> SlabIndex
        {
          if let Some(index) = self.removed_indexes.pop()
             {  self.data[index.0] = Some(item); index }
          else
             { let index = SlabIndex(self.data.len()); self.data.push(Some(item)); index }
        }

    pub fn get_mut(&mut self, index: SlabIndex) -> Option<&mut T>
        {
        if let Some(item) = self.data.get_mut(index.0)
            { return item.as_mut(); }
        None
        }

    pub fn get(&self, index: SlabIndex) -> Option<&T>
        {
        if let Some(item) = self.data.get(index.0)
            { return item.as_ref(); }
        None
        }

    pub fn iter(&self) -> Iter<'_, T>
        {
           Iter { iter: self.data.iter().enumerate(), }
        }

    pub fn all(&self) -> IterData<'_, T>
        {
           IterData { iter: self.data.iter().enumerate(), }
        }

    pub fn len(&self) -> usize
        {  self.data.len() - self.removed_indexes.len() }
}

pub struct Iter<'a, T>
{
  iter: std::iter::Enumerate<std::slice::Iter<'a, Option<T>>>,
}

impl<'a, T> Iterator for Iter<'a, T>
{
  type Item = SlabIndex;
  fn next(&mut self) -> Option<Self::Item>
      {
      loop
         {
          let (i, item) = self.iter.next()?;
          let si = SlabIndex(i);

          if item.is_none()
              { continue; }
          return Some(si);
         }
      }
}

pub struct IterData<'a, T>
{
  iter: std::iter::Enumerate<std::slice::Iter<'a, Option<T>>>,
}

impl<'a, T> Iterator for IterData<'a, T>
{
  type Item = &'a T;
  fn next(&mut self) -> Option<Self::Item>
      {
      loop
         {
          let (_, item) = self.iter.next()?;
          if item.is_none()
              { continue; }
          return Some(item.as_ref().unwrap());
         }
      }
}

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
 actors: Slab<actor::Actor>,
 defplace: places::Container
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
  Universe::save_vector_1::<W, actor::Actor>(&self.actors, target)?;
  Ok(())
 }

 pub fn load_1<R>(source: &mut R) -> Result<Self> where R:Read
 {
   let tick = source.read_u64::<LittleEndian>()?;
   let seqid = source.read_u64::<LittleEndian>()?;
   let actors = Universe::load_vector_1::<R, actor::Actor>(source)?;
   let defplace = places::Container::new();
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