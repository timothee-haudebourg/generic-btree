use std::marker::PhantomData;
use super::{
	StorageMut,
	Offset,
	Item
};

/// Internal node buffer.
pub trait Internal<S: StorageMut>: Default {
	fn parent(&self) -> Option<usize>;

	fn set_parent(&mut self, parent: Option<usize>);

	fn item_count(&self) -> usize;

	fn item(&self, offset: Offset) -> Option<&Item<S::Key, S::Value>>;

	fn child_count(&self) -> usize {
		self.item_count() + 1usize
	}

	fn child_id(&self, index: usize) -> Option<usize>;

	fn children(&self) -> Children<S, Self> {
		Children {
			node: self,
			index: 0,
			storage: PhantomData
		}
	}
	
	fn max_capacity(&self) -> usize;

	fn set_first_child(&mut self, id: usize);

	fn push_right(&mut self, item: Item<S::Key, S::Value>, child: usize);
}

pub struct Children<'a, S: StorageMut, R> {
	node: &'a R,
	index: usize,
	storage: PhantomData<S>
}

impl<'a, S: StorageMut, R: Internal<S>> Iterator for Children<'a, S, R> {
	type Item = usize;

	fn next(&mut self) -> Option<usize> {
		if self.index < self.node.child_count() {
			let i = self.index;
			self.index += 1;
			self.node.child_id(i)
		} else {
			None
		}
	}
}