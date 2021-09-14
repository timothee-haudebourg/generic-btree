use std::marker::PhantomData;
use super::StorageMut;

/// Internal buffer node.
pub trait Internal<S: StorageMut>: Default {
	fn parent(&self) -> Option<usize>;

	fn set_parent(&mut self, parent: Option<usize>);

	fn item_count(&self) -> usize;

	fn child_count(&self) -> usize {
		self.item_count() + 1usize
	}

	fn child_id(&self, index: usize) -> Option<usize>;

	fn first_child_id(&self) -> usize {
		self.child_id(0).unwrap()
	}

	fn set_first_child_id(&mut self, id: usize);

	fn children(&self) -> Children<S, Self> {
		Children {
			node: self,
			index: 0,
			storage: PhantomData
		}
	}
	
	/// Maximum capacity of the internal node.
	/// 
	/// This corresponds to the Knuth order of the tree.
	fn max_capacity(&self) -> usize;

	fn push_right(&mut self, item: S::Item, child: usize);

	/// Drop this internal node without dropping the items.
	/// 
	/// Used without care, this may lead to memory leaks.
	fn forget(self);
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