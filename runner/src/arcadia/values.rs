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

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Port(pub usize);

pub enum ValueData
{
 FValue { value: f32 },
 BValue { value: bool }
}

impl From<f32> for ValueData
{
 fn from(value: f32) -> Self { Self::FValue { value: value } }
}


pub struct Value
{
 name: String,
 value: ValueData
}

impl Value
{
 const FVALUE: u8 = 1;
 const BVALUE: u8 = 2;
}

impl Stored for Value
{
 fn save(&self, writer: &mut Writer) -> Result<()>
 {
  writer.utf8(&self.name)?;
  match self.value
    {
    ValueData::FValue {value} => { writer.u8(Self::FVALUE)?; writer.f32(value) }
    ValueData::BValue {value} => { writer.u8(Self::FVALUE)?; writer.bool(value) }
    }
 }
}

impl Factory for Value
{
 fn load(reader: &mut Reader) -> Result<Self>
 {
  let name = reader.utf8()?;
  let value = match reader.u8()?
    {
    Self::FVALUE => ValueData::FValue { value: reader.f32()? },
    Self::BVALUE => ValueData::BValue { value: reader.bool()? },
    _ => return Err(Error::new(ErrorKind::InvalidData, "Unknown value"))
    };
  Ok( Value { name, value } )
 }
}

#[derive(Default)]
pub struct Values
{
 pub values: Vec<Value>,
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
  self.birth = (reader.u8()?) != 0;
  reader.vecdeque(&mut self.birthcredits)?;
  reader.vecdeque(&mut self.seeds)
 }

 pub fn save(&self, writer: &mut Writer) -> Result<()>
 {
  writer.u8(self.version())?;
  writer.vec(&self.values)?;
  writer.u8(self.birth as u8)?;
  writer.vecdeque(&self.birthcredits)?;
  writer.vecdeque(&self.seeds)
 }

 pub fn make(&mut self, name: &String, value: ValueData) -> Port
 {
  for i in 0..self.values.len()
    {
    if self.values[i].name == *name
       { return Port(i); }
    }
  self.values.push( Value { name: name.clone(), value } );
  Port(self.values.len()-1)
 }

 pub fn get(&self, index:Port) -> &ValueData
 {
  &self.values[index.0].value
 }

 pub fn set(&mut self, index:Port, value:ValueData)
 {
  self.values[index.0].value = value;
 }
}
