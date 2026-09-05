use std::io::{Result, Error, ErrorKind};
use std::collections::{VecDeque};
use crate::arcadia::storage::{Reader,Writer};
use crate::arcadia::control::{BluePrint};

pub struct Seed
{
 pub credits: u32,
 pub blueprints: BluePrint
}

impl Seed
{
 pub fn new(credits: u32) -> Seed
 {
  Seed { credits: credits, blueprints: BluePrint::default() }
 }

 pub fn load(source: &mut Reader) -> Result<Self>
 {
  let credits = source.u32()?;
  let bp = BluePrint::load(source)?;
  Ok( Seed { credits: credits, blueprints:bp } )
 }

 pub fn save(&self, target: &mut Writer) -> Result<()>
 {
  target.u32(self.credits)?;
  self.blueprints.save(target)?;
  Ok( () )
 }

}

pub enum Value
{
 FValue { value: f32 }
}

#[derive(Default)]
pub struct Values
{
 pub credits: f32,
 pub birth: bool,
 pub birthcredits: VecDeque<Seed>,
 pub seeds: VecDeque<Seed>,
 pub values: Vec<Value>
}

impl Values
{
 fn version(&self) -> u8 { 1 }

 pub fn load(&mut self, reader: &mut Reader) -> Result<()>
 {
  if reader.u8()? > self.version()
         { return Err(Error::new(ErrorKind::InvalidData, "Unknown control version")); }
  self.credits = reader.f32()?;
  self.birth = (reader.u8()?) != 0;
  let countbc = reader.count()?;
  for _ in 0..countbc
    { self.birthcredits.push_back( Seed::load(reader)? ); }
  let counts = reader.count()? as usize;
  for _ in 0..counts
    { self.seeds.push_back( Seed::load(reader)? ); }
  Ok( () )
 }

 pub fn save(&self, writer: &mut Writer) -> Result<()>
 {
  writer.u8(self.version())?;
  writer.f32(self.credits)?;
  writer.u8(self.birth as u8)?;
  writer.count(self.birthcredits.len())?;
  for seed in self.birthcredits.iter()
      { seed.save(writer)?; }
  writer.count(self.seeds.len())?;
  for seed in self.seeds.iter()
      { seed.save(writer)?; }
  Ok(())
 }
}
