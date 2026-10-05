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
 fn make(&mut self, bp: &BluePrint, root: &BluePrint, values: &mut Values) -> Result<()>;
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

/* fn make(&self, units: &mut Units, values: &mut Values)
 {
  let mut asensor = CreditSensor::default();
  asensor.variator.set( self.variator.precision );
  asensor.credits = values.make(&"credits".to_string(), ValueData::from(0.0));
  units.append(Box::new(asensor));
 }

 fn blueprints(&self) -> BluePrint
 {
  BluePrint::new(Units::CREDIT_SENSOR, vec![ BluePrint::from(self.variator.precision) ])
 }*/

 fn make(&mut self, bp: &BluePrint, _root: &BluePrint, values: &mut Values) -> Result<()>
 {
  self.variator.set( bp.get_u32(0)? );
  self.credits = values.make(&"credits".to_string(), ValueData::from(0.0));
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

/* fn make(&self, units: &mut Units, values: &mut Values)
 {
  let mut asignal = BirthSignal::default();
  asignal.scale = self.scale;
  asignal.threshold = self.threshold;
  asignal.variation = self.variation;
  asignal.selector = Normal::new(0.0, (asignal.variation as f32)/1000.0).unwrap();
  asignal.credits = values.make(&"credits".to_string(), ValueData::from(0.0));
  asignal.birth = values.make(&"birth".to_string(), ValueData::from(false));
  units.append(Box::new(asignal));
 }

 fn blueprints(&self) -> BluePrint
 {
  BluePrint::new(Units::BIRTH_SIGNAL, vec![ BluePrint::from(self.scale), BluePrint::from(self.threshold), BluePrint::from(self.variation) ] )
 }*/

 fn make(&mut self, bp: &BluePrint, _root: &BluePrint, values: &mut Values) -> Result<()>
 {
  self.scale = bp.get_f32(0)?;
  self.threshold = bp.get_f32(1)?;
  self.variation = bp.get_u32(2)?;
  self.selector = Normal::new(0.0, (self.variation as f32)/1000.0).unwrap();
  self.credits = values.make(&"credits".to_string(), ValueData::from(0.0));
  self.birth = values.make(&"birth".to_string(), ValueData::from(false));
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

/* fn make(&self, units: &mut Units, values: &mut Values)
 {
  let mut acredit = BirthCredit::default();
  acredit.giveaway.set( self.giveaway.nominal );
  acredit.birth = values.make(&"birth".to_string(), ValueData::from(false));
  acredit.birthcredits = values.make(&"birthcredits".to_string(), ValueData::new_seeds() );
  units.append(Box::new(acredit));
 }

 fn blueprints(&self) -> BluePrint
 {
  BluePrint::new(Units::BIRTH_CREDIT, vec![ BluePrint::from(self.giveaway.nominal) ])
 }*/
 fn make(&mut self, bp: &BluePrint, _root: &BluePrint, values: &mut Values) -> Result<()>
 {
  self.giveaway.set( bp.get_u32(0)? );
  self.birth = values.make(&"birth".to_string(), ValueData::from(false));
  self.birthcredits = values.make(&"birthcredits".to_string(), ValueData::new_seeds() );
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

/* fn make(&self, units: &mut Units, values: &mut Values)
 {
  let mut amaker = ChildMaker::default();
  amaker.variator.set(self.variator.precision);
  amaker.birthcredits = values.make(&"birthcredits".to_string(), ValueData::new_seeds() );
  amaker.seeds = values.make(&"seeds".to_string(), ValueData::new_seeds() );
  units.append(Box::new(amaker));
 }

 fn blueprints(&self) -> BluePrint
 {
  BluePrint::new(Units::CHILD_MAKER, vec![ BluePrint::from(self.variator.precision) ])
 }*/

 fn make(&mut self, bp: &BluePrint, root: &BluePrint, values: &mut Values) -> Result<()>
 {
  self.variator.set( bp.get_u32(0)? );
  self.blueprints = bp.get_reference(1, root)?.clone();
  self.birthcredits = values.make(&"birthcredits".to_string(), ValueData::new_seeds() );
  self.seeds = values.make(&"seeds".to_string(), ValueData::new_seeds() );
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
/* fn make(&self, units: &mut Units, values: &mut Values)
 {
  let mut aspawner = Spawner::default();
  aspawner.seeds = values.make(&"seeds".to_string(), ValueData::new_seeds() );
  units.append(Box::new(aspawner));
 }
 fn blueprints(&self) -> BluePrint
 {
  BluePrint::new(Units::SPAWNER, vec![ ])
 }*/
 fn make(&mut self, _bp: &BluePrint, _root: &BluePrint, values: &mut Values) -> Result<()>
 {
  self.seeds = values.make(&"seeds".to_string(), ValueData::new_seeds() );
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
//  factory.insert(Self::COMPOUND, || Box::new(Compound::default()) );
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
// const COMPOUND :u16 = 1;
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

/* pub fn new(&mut self, bp: &BluePrint, values: &mut Values) -> Result<()>
 {
  for ubp in bp.get_collection()?
      {
      let mut aunit = self.factory.get(&ubp.get_unit()?).expect("Unknown unit")();
      aunit.make_bp(ubp, values)?;
      self.units.push(aunit);
      }

  Ok(())
 }

 pub fn make(&self, units: &mut Units, values: &mut Values)
 {
  for u in self.units.iter()
     { u.make(units, values); }
 }

 pub fn append(&mut self, unit: Box<dyn Unit>)
 {
  self.units.push(unit);
 }*/

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

  for ubp in instructions
    { compartment.make_instructions(&ubp, root, factory)?; }

  Ok( Box::new(compartment) )
 }

 pub fn make_instructions(&mut self, bp: &BluePrint, root: &BluePrint, factory: &UnitFactory) -> Result<()>
 {
  match bp
    {
    BluePrint::Instruction {unit, instructions: _} =>
          {
          let mut aunit = factory.get(*unit);
          aunit.make(bp, root, &mut self.values)?;
          self.units.push(aunit);
          },
    BluePrint::Compound { node: _, instructions} =>
          {
          for ubp in instructions
              { self.make_instructions(&ubp, root, factory)?; }
          },
     _ => return Err(Error::new(ErrorKind::InvalidData, "Invalid instructions for a unit"))
    }
 Ok(())
 }

/* fn blueprints(&self) -> BluePrint
 {
  let mut value : Vec<BluePrint> = Vec::new();
  for unit in self.units.iter()
     { value.push( unit.blueprints() ); }
  BluePrint::Collection { unit: Self::COMPOUND, value: value }
 }*/

}

/*#[derive(Default)]
pub struct Compound
{
 units: Units,
 values: Values
}

impl Unit for Compound
{
 fn utype(&self) -> u16 { Units::COMPOUND }
 fn version(&self) -> u8 { 1 }
 fn load(&mut self, _version: u8, reader: &mut Reader, factory: &UnitFactory) -> Result<()>
 {
  self.units.load(reader, factory)?;
  self.values.load(reader)
 }
 fn save(&self, writer: &mut Writer) -> Result<()>
 {
  self.units.save(writer)?;
  self.values.save(writer)
 }
 fn tick(&mut self, _values: &mut Values, body: &mut Body)
 {
  self.units.tick(&mut self.values, body);
 }
 fn make(&self, units: &mut Units, _values: &mut Values)
 {
  let mut acomp = Self::default();
  self.units.make(&mut acomp.units, &mut acomp.values);
  units.append(Box::new(acomp));
 }
 fn blueprints(&self) -> BluePrint
 {
  self.units.blueprints()
 }
 fn make(&mut self, bp: &BluePrint, root: &BluePrint, _values: &mut Values) -> Result<()>
 {
  self.units.new(bp, root, &mut self.values)
 }
}

impl Compound
{
 fn tick_top(&self, body: &mut Body)
 {
  self.units.tick(&mut self.values, body);
 }
}*/

/*#[derive(Default)]
pub struct Control
{
 units: Units,
 values: Values,
}

impl Control
{
 fn version(&self) -> u8 { 1 }

 pub fn tick(&mut self, body: &mut Body)
 {
  self.units.tick(&mut self.values, body);
 }

 pub fn make_from(&mut self, source: &Units)
 {
  source.make(&mut self.units, &mut self.values);
 }

 pub fn new(&mut self, bp: &BluePrint)
 {
  self.units.new(bp, &mut self.values).unwrap();
 }

 pub fn load(&mut self, reader: &mut Reader) -> Result<()>
 {
   if reader.u8()? > self.version()
         { return Err(Error::new(ErrorKind::InvalidData, "Unknown control version")); }
   self.units.load(reader)?;
   self.values.load(reader)?;
   Ok(())
 }

 pub fn save(&self, writer: &mut Writer) -> Result<()>
 {
   writer.u8(self.version())?;
   self.units.save(writer)?;
   self.values.save(writer)?;
   Ok(())
 }
}*/