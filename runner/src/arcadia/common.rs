use std::marker::PhantomData;
use std::io::{Result};
use crate::arcadia::storage::{Reader, Writer, Factory, Stored};

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Id<T>
{
 value: u64,
 value_type: PhantomData<T>,
}

impl<T> Id<T>
{
 pub fn new(id: u64) -> Self
  { Id::<T>{value: id, value_type: PhantomData} }
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
