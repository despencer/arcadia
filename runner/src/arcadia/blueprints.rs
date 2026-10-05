use std::io::{Result, Error, ErrorKind};
use crate::arcadia::storage::{Reader,Writer,Stored,Factory};
use crate::arcadia::control::{Variator, Compartment, UnitFactory};

pub enum BluePrint
{
 FValue { value: f32},
 UValue { value: u32},
 Instruction { unit: u16, instructions: Vec<BluePrint> },
 Compound { node: String, instructions: Vec<BluePrint> },
 Architecture { components: Vec<BluePrint>, instructions: Vec<BluePrint> },
 Reference {}
}

impl Default for BluePrint
{
 fn default() -> Self
   { Self::Architecture { components: vec![], instructions: vec![] } }
}

impl BluePrint
{
 const FLOAT: u8 = 1;
 const INTEGER: u8 = 2;
 const INSTRUCTION: u8 = 3;
 const COMPOUND: u8 = 4;
 const ARCHITECTURE: u8 = 5;
 const REFERENCE: u8 = 6;
}

impl Factory for BluePrint
{
 fn load(reader: &mut Reader) -> Result<Self>
 {
  let utype = reader.u8()?;
  let bp = match utype
        {
        Self::FLOAT => Self::FValue { value: reader.f32()? },
        Self::INTEGER => Self::UValue { value: reader.u32()? },
        Self::INSTRUCTION =>
              {
              let unit = reader.u16()?;
              let mut instructions : Vec<BluePrint> = Vec::new();
              reader.vec(&mut instructions)?;
              Self::Instruction { unit: unit, instructions: instructions }
              },
        Self::COMPOUND =>
              {
              let node = reader.utf8()?;
              let mut instructions : Vec<BluePrint> = Vec::new();
              reader.vec(&mut instructions)?;
              Self::Compound { node, instructions }
              },
        Self::ARCHITECTURE =>
              {
              let mut components : Vec<BluePrint> = Vec::new();
              reader.vec(&mut components)?;
              let mut instructions : Vec<BluePrint> = Vec::new();
              reader.vec(&mut instructions)?;
              Self::Architecture { components, instructions }
              }
        Self::REFERENCE => Self::Reference {},
        _ => return Err(Error::new(ErrorKind::InvalidData, "Unknown blueprint"))
        };
  Ok(bp)
 }
}

impl Stored for BluePrint
{
 fn save(&self, target: &mut Writer) -> Result<()>
 {
  match self
    {
    Self::FValue {value} =>
           { target.u8(Self::FLOAT)?; target.f32(*value)? },
    Self::UValue {value} =>
           { target.u8(Self::INTEGER)?; target.u32(*value)? },
    Self::Instruction {unit, instructions} =>
           { target.u8(Self::INSTRUCTION)?;
             target.u16(*unit)?;
             target.vec(instructions)?; },
    Self::Compound {node, instructions} =>
           { target.u8(Self::COMPOUND)?;
             target.utf8(&node)?;
             target.vec(instructions)?; },
    Self::Architecture {components, instructions } =>
           { target.u8(Self::ARCHITECTURE)?;
             target.vec(components)?;
             target.vec(instructions)?; },
    Self::Reference {} =>
           { target.u8(Self::REFERENCE)?; }
    }
  Ok( () )
 }
}

impl From<u32> for BluePrint
{
 fn from(value: u32) -> Self { Self::UValue { value: value } }
}

impl From<f32> for BluePrint
{
 fn from(value: f32) -> Self { Self::FValue { value: value } }
}

impl Clone for BluePrint
{
 fn clone(&self) -> Self
 {
  match self
    {
    Self::FValue {value} => Self::FValue { value: *value },
    Self::UValue {value} => Self::UValue { value: *value },
    Self::Instruction {unit, instructions} =>
          Self::Instruction { unit: *unit, instructions: instructions.into_iter().map(|bp| bp.clone()).collect() },
    Self::Compound {node, instructions} =>
          Self::Compound { node: node.clone(), instructions: instructions.into_iter().map(|bp| bp.clone()).collect() },
    Self::Architecture {components, instructions } =>
          Self::Architecture { components: components.into_iter().map(|bp| bp.clone()).collect()
                   , instructions: instructions.into_iter().map(|bp| bp.clone()).collect() },
    Self::Reference {} => Self::Reference {}
    }
 }
}

impl BluePrint
{
 pub fn variate(&self, variator: &Variator) -> Self
 {
  match self
    {
    Self::FValue {value} => Self::FValue { value: variator.variate(*value) },
    Self::UValue {value} => Self::UValue { value: variator.variate_u32(*value) },
    Self::Instruction {unit, instructions} =>
          Self::Instruction { unit: *unit, instructions: instructions.into_iter().map(|bp| bp.variate(variator)).collect() },
    Self::Compound {node, instructions} =>
          Self::Compound { node: node.clone(), instructions: instructions.into_iter().map(|bp| bp.variate(variator)).collect() },
    Self::Architecture {components, instructions } =>
          Self::Architecture { components: components.into_iter().map(|bp| bp.variate(variator)).collect(),
                   instructions: instructions.into_iter().map(|bp| bp.variate(variator)).collect() },
    Self::Reference {} => Self::Reference {}
    }
 }

 pub fn make(&self) -> Box<Compartment>
 {
  let factory = UnitFactory::default();
  Compartment::make(self, self, &factory).unwrap()
 }

 pub fn get_instructions(&self) -> Result<&[BluePrint]>
 {
  match self
     {
     Self::Instruction {instructions, ..} => Ok(&instructions),
     _ => Err(Error::new(ErrorKind::InvalidData, "Instructions required"))
     }
 }

/* pub fn get_unit(&self) -> Result<u16>
 {
  match self
     {
     Self::Instruction {unit, ..} => Ok(*unit),
     _ => Err(Error::new(ErrorKind::InvalidData, "Instructions required"))
     }
 }*/

 pub fn get_f32(&self, index: usize) -> Result<f32>
 {
  let item = &(self.get_instructions()?)[index];
  match item
     {
     BluePrint::FValue { value } => Ok( *value ),
     _ => Err(Error::new(ErrorKind::InvalidData, "F32 required"))
     }
 }

 pub fn get_u32(&self, index: usize) -> Result<u32>
 {
  let item = &(self.get_instructions()?)[index];
  match item
     {
     BluePrint::UValue { value } => Ok( *value ),
     _ => Err(Error::new(ErrorKind::InvalidData, "U32 required"))
     }
 }

 pub fn get_reference<'a>(&self, index: usize, root: &'a BluePrint) -> Result<&'a BluePrint>
 {
  let item = &(self.get_instructions()?)[index];
  match item
     {
     BluePrint::Reference { } => Ok( root ),
     _ => Err(Error::new(ErrorKind::InvalidData, "Reference required"))
     }
 }

}
