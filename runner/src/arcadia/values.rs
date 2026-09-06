use std::io::{Result, Error, ErrorKind};
use std::collections::{VecDeque};
use crate::arcadia::storage::{Reader,Writer,Stored,Factory};
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
}

impl Factory for Seed
{
 fn load(source: &mut Reader) -> Result<Self>
 {
  let credits = source.u32()?;
  let bp = BluePrint::load(source)?;
  Ok( Seed { credits: credits, blueprints:bp } )
 }
}

impl Stored for Seed
{
 fn save(&self, target: &mut Writer) -> Result<()>
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

impl Value
{
 const FVALUE: u8 = 1;
}

impl Stored for Value
{
 fn save(&self, writer: &mut Writer) -> Result<()>
 {
  match self
    {
    Self::FValue {value} => { writer.u8(Self::FVALUE)?; writer.f32(*value) }
    }
 }
}

impl Factory for Value
{
 fn load(reader: &mut Reader) -> Result<Self>
 {
  let v = match reader.u8()?
    {
    Self::FVALUE => Self::FValue { value: reader.f32()? },
    _ => return Err(Error::new(ErrorKind::InvalidData, "Unknown value"))
    };
  Ok(v)
 }
}

#[derive(Default)]
pub struct Values
{
 pub values: Vec<Value>,
 pub credits: f32,
 pub birth: bool,
 pub birthcredits: VecDeque<Seed>,
 pub seeds: VecDeque<Seed>,
}

impl Values
{
 fn version(&self) -> u8 { 1 }

 pub fn load(&mut self, reader: &mut Reader) -> Result<()>
 {
  if reader.u8()? > self.version()
         { return Err(Error::new(ErrorKind::InvalidData, "Unknown control version")); }
  reader.vec(&mut self.values)?;
  self.credits = reader.f32()?;
  self.birth = (reader.u8()?) != 0;
  reader.vecdeque(&mut self.birthcredits)?;
  reader.vecdeque(&mut self.seeds)
 }

 pub fn save(&self, writer: &mut Writer) -> Result<()>
 {
  writer.u8(self.version())?;
  writer.vec(&self.values)?;
  writer.f32(self.credits)?;
  writer.u8(self.birth as u8)?;
  writer.vecdeque(&self.birthcredits)?;
  writer.vecdeque(&self.seeds)
 }
}
