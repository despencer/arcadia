use std::io::{Result, Error, ErrorKind};
use std::collections::{VecDeque};
use crate::arcadia::storage::{Reader,Writer,Stored,Factory};
use crate::arcadia::control::{Compartment,UnitFactory};

pub struct Seed
{
 pub credits: u32,
 pub seed: Box<Compartment>
}

impl Seed
{
 pub fn new(credits: u32) -> Seed
 {
  Seed { credits: credits, seed: Box::new(Compartment::default()) }
 }
}

impl Factory for Seed
{
 fn load(source: &mut Reader) -> Result<Self>
 {
  let credits = source.u32()?;
  let factory = UnitFactory::default();
  let component = Compartment::load(source, &factory)?;
  Ok( Seed { credits: credits, seed: component } )
 }
}

impl Stored for Seed
{
 fn save(&self, target: &mut Writer) -> Result<()>
 {
  target.u32(self.credits)?;
  self.seed.save(target)
 }

}

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Port(pub usize);

pub enum ValueData
{
 FValue { value: f32 },
 BValue { value: bool },
 Seeds { value: VecDeque<Seed> }
}

impl From<f32> for ValueData
{
 fn from(value: f32) -> Self { Self::FValue { value: value } }
}

impl From<bool> for ValueData
{
 fn from(value: bool) -> Self { Self::BValue { value: value } }
}

impl ValueData
{
 pub fn new_seeds() -> Self { Self::Seeds { value: VecDeque::new() } }
}

pub struct Value
{
 name: String,
 value: ValueData
}

impl Value
{
 const FLOAT: u8 = 1;
 const BOOL: u8 = 2;
 const SEEDS: u8 = 3;
}

impl Stored for Value
{
 fn save(&self, writer: &mut Writer) -> Result<()>
 {
  writer.utf8(&self.name)?;
  match &self.value
    {
    ValueData::FValue {value} => { writer.u8(Self::FLOAT)?; writer.f32(*value) }
    ValueData::BValue {value} => { writer.u8(Self::BOOL)?; writer.bool(*value) }
    ValueData::Seeds {value} => { writer.u8(Self::SEEDS)?; writer.vecdeque(value) }
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
    Self::FLOAT => ValueData::FValue { value: reader.f32()? },
    Self::BOOL => ValueData::BValue { value: reader.bool()? },
    Self::SEEDS => { let mut seeds: VecDeque<Seed> = VecDeque::new(); reader.vecdeque(&mut seeds)?; ValueData::Seeds { value: seeds } },
    _ => return Err(Error::new(ErrorKind::InvalidData, "Unknown value"))
    };
  Ok( Value { name, value } )
 }
}

#[derive(Default)]
pub struct Values
{
 pub values: Vec<Value>,
}

impl Values
{
 fn version(&self) -> u8 { 1 }

 pub fn load(&mut self, reader: &mut Reader) -> Result<()>
 {
  if reader.u8()? > self.version()
         { return Err(Error::new(ErrorKind::InvalidData, "Unknown control version")); }
  reader.vec(&mut self.values)
 }

 pub fn save(&self, writer: &mut Writer) -> Result<()>
 {
  writer.u8(self.version())?;
  writer.vec(&self.values)
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

 pub fn get_mut(&mut self, index:Port) -> &mut ValueData
 {
  &mut self.values[index.0].value
 }

 pub fn set(&mut self, index:Port, value:ValueData)
 {
  self.values[index.0].value = value;
 }
}
