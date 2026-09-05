use std::io::{Result};
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
 pub fn load(&mut self, source: &mut Reader) -> Result<()>
 {
  self.credits = source.f32()?;
  self.birth = (source.u8()?) != 0;
  let countbc = source.count()?;
  for _ in 0..countbc
    { self.birthcredits.push_back( Seed::load(source)? ); }
  let counts = source.count()? as usize;
  for _ in 0..counts
    { self.seeds.push_back( Seed::load(source)? ); }
  Ok( () )
 }

 pub fn save(&self, target: &mut Writer) -> Result<()>
 {
  target.f32(self.credits)?;
  target.u8(self.birth as u8)?;
  target.count(self.birthcredits.len())?;
  for seed in self.birthcredits.iter()
      { seed.save(target)?; }
  target.count(self.seeds.len())?;
  for seed in self.seeds.iter()
      { seed.save(target)?; }
  Ok(())
 }
}
