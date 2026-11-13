use std::marker::PhantomData;
use std::io::{Result};
use crate::arcadia::storage::{Reader, Writer, Factory, Stored};

pub struct Id<T>
{
 value: u64,
 value_type: PhantomData<T>,
}

impl<T> std::hash::Hash for Id<T>
{
 fn hash<H: std::hash::Hasher>(&self, state: &mut H)
   { self.value.hash(state); }
}

impl<T> PartialEq for Id<T>
{
 fn eq(&self, other: &Self) -> bool
    {  self.value == other.value }
}

impl<T> Clone for Id<T>
{
 fn clone(&self) -> Self { Id::<T> { value: self.value, value_type: PhantomData } }
}

impl<T> Copy for Id<T> { }
impl<T> Eq for Id<T> { }

impl<T> Default for Id<T>
{
 fn default() -> Self { Id::<T> { value: 0, value_type: PhantomData } }
}

impl<T> std::fmt::Debug for Id<T>
{
 fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { write!(f, "{}", &self.value)  }
}

impl<T> Id<T>
{
 pub fn new(id: u64) -> Self
  { Id::<T>{value: id, value_type: PhantomData} }
 pub fn get_value(&self) -> u64 { self.value }
}

impl<T> Factory for Id<T>
{
 fn load(reader: &mut Reader) -> Result<Self>
  { Ok( Id::<T> { value: reader.u64()?, value_type: PhantomData } ) }
}

impl<T> Stored for Id<T>
{
 fn save(&self, writer: &mut Writer) -> Result<()>
  { writer.u64(self.value) }
}
