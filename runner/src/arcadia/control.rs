use std::io::{Result, Error, ErrorKind};
use std::collections::{HashMap, VecDeque};
use rand_distr::{Normal, Distribution};
use crate::arcadia::actors::Body;
use crate::arcadia::values::{Seed, Values, ValueData, Port};
use crate::arcadia::storage::{Reader,Writer,Factory,Stored};
use crate::arcadia::blueprints::BluePrint;

pub struct Sampler
{
 nominal: u32,
 selector: Normal<f32>
}

impl Default for Sampler
{
 fn default() -> Self
 {
  Sampler { nominal: 0, selector: Normal::new(0.0, 1.0).unwrap() }
 }
}

impl Sampler
{
 pub fn set(&mut self, value: u32)
 {
  self.nominal = value;
  self.selector = Normal::new(value as f32, (value as f32)/10.0).unwrap();
 }

 pub fn sample(&self) -> u32
 {
  let mut rng = rand::thread_rng();
  let r = self.selector.sample(&mut rng);
  if r <= 0.0
     { return 0; }
  r as u32
 }
}

pub struct Variator
{
 precision: u32,
 selector: Normal<f32>
}

impl Default for Variator
{
 fn default() -> Self
 {
  Variator { precision: 0, selector: Normal::new(0.0, 1.0).unwrap() }
 }
}

impl Variator
{
 pub fn set(&mut self, value: u32)
 {
  self.precision = value;
  self.selector = Normal::new(1.0, (self.precision as f32)/1000.0).unwrap();
 }

 pub fn variate(&self, value: f32) -> f32
 {
  let mut rng = rand::thread_rng();
  self.selector.sample(&mut rng) * value
 }

 pub fn variate_u32(&self, value: u32) -> u32
 {
  let r = self.variate(value as f32);
  if r <= 0.0
     { return 0; }
  if r >= 2_000_000_000.0
     { return 2_000_000_000; }
  r as u32
 }
}

pub trait Unit
{
 fn utype(&self) -> u16;
 fn version(&self) -> u8;
 fn load(&mut self, version: u8, reader: &mut Reader) -> Result<()>;
 fn save(&self, target: &mut Writer) -> Result<()>;
 fn tick(&mut self, values: &mut Values, body: &mut Body);
 fn make(&mut self, bp: &BluePrint, root: &BluePrint, values: &mut Values, node: &String) -> Result<()>;
}

#[derive(Default)]
pub struct CreditSensor
{
 variator: Variator,
 credits: Port
}

impl Unit for CreditSensor
{
 fn utype(&self) -> u16 { UnitFactory::CREDIT_SENSOR }
 fn version(&self) -> u8 { 1 }

 fn load(&mut self, _version: u8, reader: &mut Reader) -> Result<()>
 {
  self.variator.set( reader.u32()? );
  self.credits = reader.port()?;
  Ok( () )
 }

 fn save(&self, target: &mut Writer) -> Result<()>
 {
  target.u32(self.variator.precision)?;
  target.port(self.credits)?;
  Ok(())
 }

 fn tick(&mut self, values: &mut Values, body: &mut Body)
 {
  values.set(self.credits, ValueData::from( self.variator.variate(body.get_credits() as f32) ));
 }

 fn make(&mut self, bp: &BluePrint, _root: &BluePrint, values: &mut Values, node: &String) -> Result<()>
 {
  self.variator.set( bp.get_u32(0)? );
  self.credits = values.make(node, &"credits".to_string(), ValueData::from(0.0));
  Ok(())
 }
}

pub struct BirthSignal
{
 scale: f32,
 threshold: f32,
 variation: u32,
 selector: Normal<f32>,
 credits: Port,
 birth: Port
}

impl Default for BirthSignal
{
 fn default() -> Self
 {
  BirthSignal { scale: 0.0, threshold: 0.0, variation:0, selector: Normal::new(0.0, 1.0).unwrap(), credits:Port(0), birth:Port(0) }
 }
}

impl Unit for BirthSignal
{
 fn utype(&self) -> u16 { UnitFactory::BIRTH_SIGNAL }
 fn version(&self) -> u8 { 1 }

 fn load(&mut self, _version: u8, reader: &mut Reader) -> Result<()>
 {
  self.scale = reader.f32()?;
  self.threshold = reader.f32()?;
  self.variation = reader.u32()?;
  self.selector = Normal::new(0.0, (self.variation as f32)/1000.0).unwrap();
  self.credits = reader.port()?;
  self.birth = reader.port()?;
  Ok( () )
 }

 fn save(&self, target: &mut Writer) -> Result<()>
 {
  target.f32(self.scale)?;
  target.f32(self.threshold)?;
  target.u32(self.variation)?;
  target.port(self.credits)?;
  target.port(self.birth)?;
  Ok(())
 }

 fn tick(&mut self, values: &mut Values, _body: &mut Body)
 {
  let mut rng = rand::thread_rng();
  let ValueData::FValue{ value: credits} = values.get(self.credits) else { panic!("Wrong data type for credits") };
  let value = (credits / self.scale) + self.threshold;
  values.set(self.birth, ValueData::from( self.selector.sample(&mut rng) < value ));
 }

 fn make(&mut self, bp: &BluePrint, _root: &BluePrint, values: &mut Values, node: &String) -> Result<()>
 {
  self.scale = bp.get_f32(0)?;
  self.threshold = bp.get_f32(1)?;
  self.variation = bp.get_u32(2)?;
  self.selector = Normal::new(0.0, (self.variation as f32)/1000.0).unwrap();
  self.credits = values.make(node, &"credits".to_string(), ValueData::from(0.0));
  self.birth = values.make(node, &"birth".to_string(), ValueData::from(false));
  Ok( () )
 }
}

#[derive(Default)]
pub struct BirthCredit
{
 giveaway: Sampler,
 birth: Port,
 birthcredits: Port
}

impl Unit for BirthCredit
{
 fn utype(&self) -> u16 { UnitFactory::BIRTH_CREDIT }
 fn version(&self) -> u8 { 1 }

 fn load(&mut self, _version: u8, reader: &mut Reader) -> Result<()>
 {
  self.giveaway.set( reader.u32()? );
  self.birth = reader.port()?;
  self.birthcredits = reader.port()?;
  Ok(())
 }
 fn save(&self, target: &mut Writer) -> Result<()>
 {
  target.u32(self.giveaway.nominal)?;
  target.port(self.birth)?;
  target.port(self.birthcredits)?;
  Ok(())
 }
 fn tick(&mut self, values: &mut Values, body: &mut Body)
 {
  let ValueData::BValue{ value: birth} = values.get(self.birth) else { panic!("Wrong data type for birth") };
  if *birth
     {
     values.set(self.birth, ValueData::from( false ));
     let mut giveaway = self.giveaway.sample();
     giveaway = body.take_credits(giveaway);
     if giveaway > 0
        {
        let &mut ValueData::Seeds { value: ref mut birthcredits} = values.get_mut(self.birthcredits) else { panic!("Wrong data type for birthcredits") };
        birthcredits.push_back( Seed::new(giveaway) );
        }
     }
 }

 fn make(&mut self, bp: &BluePrint, _root: &BluePrint, values: &mut Values, node: &String) -> Result<()>
 {
  self.giveaway.set( bp.get_u32(0)? );
  self.birth = values.make(node, &"birth".to_string(), ValueData::from(false));
  self.birthcredits = values.make(node, &"birthcredits".to_string(), ValueData::new_seeds() );
  Ok( () )
 }
}

#[derive(Default)]
pub struct ChildMaker
{
 variator: Variator,
 blueprints: BluePrint,
 birthcredits: Port,
 seeds: Port
}

impl Unit for ChildMaker
{
 fn utype(&self) -> u16 { UnitFactory::CHILD_MAKER }
 fn version(&self) -> u8 { 1 }

 fn load(&mut self, _version: u8, reader: &mut Reader) -> Result<()>
 {
  self.variator.set( reader.u32()? );
  self.blueprints = BluePrint::load(reader)?;
  self.birthcredits = reader.port()?;
  self.seeds = reader.port()?;
  Ok(()) 
 }
 fn save(&self, target: &mut Writer) -> Result<()>
 {
  target.u32(self.variator.precision)?;
  self.blueprints.save(target)?;
  target.port(self.birthcredits)?;
  target.port(self.seeds)?;
  Ok(())
 }
 fn tick(&mut self, values: &mut Values, _body: &mut Body)
 {
  let &mut ValueData::Seeds { value: ref mut birthcredits} = values.get_mut(self.birthcredits) else { panic!("Wrong data type for birthcredits") };
  let mut buf : VecDeque<Seed> = VecDeque::new();

  while birthcredits.len() > 0
     { buf.push_back( birthcredits.pop_front().unwrap() ); }

  let &mut ValueData::Seeds { value: ref mut seeds} = values.get_mut(self.seeds) else { panic!("Wrong data type for seeds") };

  while buf.len() > 0
     {
     let mut seed = buf.pop_front().unwrap();
     let bp = self.blueprints.variate(&self.variator);
     seed.seed = bp.make();
     seeds.push_back(seed);
     }
 }

 fn make(&mut self, bp: &BluePrint, root: &BluePrint, values: &mut Values, node: &String) -> Result<()>
 {
  self.variator.set( bp.get_u32(0)? );
  self.blueprints = bp.get_reference(1, root)?.clone();
  self.birthcredits = values.make(node, &"birthcredits".to_string(), ValueData::new_seeds() );
  self.seeds = values.make(node, &"seeds".to_string(), ValueData::new_seeds() );
  Ok( () )
 }

}

#[derive(Default)]
pub struct Spawner
{
 seeds: Port
}

impl Unit for Spawner
{
 fn utype(&self) -> u16 { UnitFactory::SPAWNER }
 fn version(&self) -> u8 { 1 }

 fn load(&mut self, _version: u8, reader: &mut Reader) -> Result<()>
 {
  self.seeds = reader.port()?;
  Ok(()) 
 }
 fn save(&self, target: &mut Writer) -> Result<()>
 {
  target.port(self.seeds)?;
  Ok(())
 }
 fn tick(&mut self, values: &mut Values, body: &mut Body)
 {
  let &mut ValueData::Seeds { value: ref mut seeds} = values.get_mut(self.seeds) else { panic!("Wrong data type for seeds") };

  while seeds.len() > 0
     {
     let seed = seeds.pop_front().unwrap();
     body.birth(seed);
     }
 }

 fn make(&mut self, _bp: &BluePrint, _root: &BluePrint, values: &mut Values, node: &String) -> Result<()>
 {
  self.seeds = values.make(node, &"seeds".to_string(), ValueData::new_seeds() );
  Ok(())
 }
}

type UnitCreator = fn() -> Box<dyn Unit>;

pub struct UnitFactory
{
 factory: HashMap<u16, UnitCreator>
}

impl Default for UnitFactory
{
 fn default() -> Self
  {
  let mut factory: HashMap<u16, UnitCreator> = HashMap::new();
  factory.insert(Self::CREDIT_SENSOR, || Box::new(CreditSensor::default()) );
  factory.insert(Self::BIRTH_SIGNAL, || Box::new(BirthSignal::default()) );
  factory.insert(Self::BIRTH_CREDIT, || Box::new(BirthCredit::default()) );
  factory.insert(Self::CHILD_MAKER, || Box::new(ChildMaker::default()) );
  factory.insert(Self::SPAWNER, || Box::new(Spawner::default()) );
  UnitFactory { factory }
  }
}

impl UnitFactory
{
 const CREDIT_SENSOR :u16 = 2;
 const BIRTH_SIGNAL :u16 = 3;
 const BIRTH_CREDIT :u16 = 4;
 const CHILD_MAKER :u16 = 5;
 const SPAWNER :u16 = 6;

 pub fn get (&self, unit: u16) -> Box<dyn Unit>
 {
  (self.factory.get(&unit).unwrap()) ()
 }
}

#[derive(Default)]
pub struct Compartment
{
 components: Vec<Box<Compartment>>,
 units: Vec<Box<dyn Unit>>,
 values: Values,
}

impl Compartment
{
 fn version(&self) -> u8 { 1 }

 pub fn tick(&mut self, body: &mut Body)
 {
  for u in &mut self.units
     { u.tick(&mut self.values, body); }
  for c in &mut self.components
     { c.tick(body); }
 }

 pub fn load(reader: &mut Reader, factory: &UnitFactory) -> Result<Box<Compartment>>
 {
  let mut compartment = Self::default();
  if reader.u8()? > compartment.version()
         { return Err(Error::new(ErrorKind::InvalidData, "Unknown compartment version")); }

  for _ in 0..reader.count()?
     { compartment.components.push( Compartment::load(reader, factory)? ); }

  for _ in 0..reader.count()?
     {
     let mut unit = factory.get(reader.u16()?);
     let version = reader.u8()?;
     if version > unit.version()
         { return Err(Error::new(ErrorKind::InvalidData, "Unknown unit version")); }
     unit.load(version, reader)?;
     compartment.units.push(unit);
     }
   compartment.values.load(reader)?;
   Ok( Box::new(compartment) )
 }

 pub fn save(&self, writer: &mut Writer) -> Result<()>
 {
  writer.u8(self.version())?;

  writer.count(self.components.len())?;
  for comp in &self.components
      { comp.save(writer)?;  }

  writer.count(self.units.len())?;
  for unit in &self.units
      {
      writer.u16(unit.utype())?;
      writer.u8(unit.version())?;
      unit.save(writer)?;
      }
   self.values.save(writer)?;
   Ok(())
 }

 pub fn make(bp: &BluePrint, root: &BluePrint, factory: &UnitFactory) -> Result<Box<Compartment>>
 {
  let BluePrint::Architecture {components, instructions} = bp else { panic!("Wrong blueprint for compartment") };
  let mut compartment = Self::default();

  for ubp in components
      { compartment.components.push( Compartment::make(ubp, root, factory)? ); }

  let node = "/".to_string();
  compartment.make_instructions(instructions, root, factory, &node)?;

  Ok( Box::new(compartment) )
 }

 pub fn make_instructions(&mut self, bp: &BluePrint, root: &BluePrint, factory: &UnitFactory, basenode: &String) -> Result<()>
 {
  match bp
    {
    BluePrint::Instruction {unit, instructions: _} =>
          {
          let mut aunit = factory.get(*unit);
          aunit.make(bp, root, &mut self.values, basenode)?;
          self.units.push(aunit);
          },
    BluePrint::Compound { node, instructions} =>
          {
          let cnode = Values::subnode(basenode, &node);
          for ubp in instructions
              { self.make_instructions(&ubp, root, factory, &cnode)?; }
          },
     _ => return Err(Error::new(ErrorKind::InvalidData, "Invalid instructions for a unit"))
    }
 Ok(())
 }

}
