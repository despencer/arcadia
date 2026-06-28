#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct SlabIndex(usize);

pub struct Slab<T: Sized>
{
    data: Vec<Option<T>>,
    removed_indexes: Vec<SlabIndex>,
}

impl<T: Sized> Slab<T>
{
    pub fn new() -> Self
        { Self { data: Vec::new(), removed_indexes: Default::default(), } }

    pub fn insert(&mut self, item: T) -> SlabIndex
        {
          if let Some(index) = self.removed_indexes.pop()
             {  self.data[index.0] = Some(item); index }
          else
             { let index = SlabIndex(self.data.len()); self.data.push(Some(item)); index }
        }

    pub fn get_mut(&mut self, index: SlabIndex) -> Option<&mut T>
        {
        if let Some(item) = self.data.get_mut(index.0)
            { return item.as_mut(); }
        None
        }

    pub fn get(&self, index: SlabIndex) -> Option<&T>
        {
        if let Some(item) = self.data.get(index.0)
            { return item.as_ref(); }
        None
        }

    pub fn iter(&self) -> Iter<'_, T>
        {
           Iter { iter: self.data.iter().enumerate(), }
        }

    pub fn all(&self) -> IterData<'_, T>
        {
           IterData { iter: self.data.iter().enumerate(), }
        }

    pub fn len(&self) -> usize
        {  self.data.len() - self.removed_indexes.len() }
}

pub struct Iter<'a, T>
{
  iter: std::iter::Enumerate<std::slice::Iter<'a, Option<T>>>,
}

impl<'a, T> Iterator for Iter<'a, T>
{
  type Item = SlabIndex;
  fn next(&mut self) -> Option<Self::Item>
      {
      loop
         {
          let (i, item) = self.iter.next()?;
          let si = SlabIndex(i);

          if item.is_none()
              { continue; }
          return Some(si);
         }
      }
}

pub struct IterData<'a, T>
{
  iter: std::iter::Enumerate<std::slice::Iter<'a, Option<T>>>,
}

impl<'a, T> Iterator for IterData<'a, T>
{
  type Item = &'a T;
  fn next(&mut self) -> Option<Self::Item>
      {
      loop
         {
          let (_, item) = self.iter.next()?;
          if item.is_none()
              { continue; }
          return Some(item.as_ref().unwrap());
         }
      }
}
