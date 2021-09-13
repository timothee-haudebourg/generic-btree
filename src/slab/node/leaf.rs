use smallvec::SmallVec;
use crate::{
	btree::{
		self,
		node::Offset
	},
	slab::{
		M,
		Node,
		Storage
	}
};

pub struct Leaf<T> {
	parent: usize,
	items: SmallVec<[T; M+1]>
}

impl<T> Default for Leaf<T> {
	fn default() -> Self {
		Self {
			parent: usize::MAX,
			items: SmallVec::new()
		}
	}
}

impl<T, S: cc_traits::SlabMut<Node<T>>> btree::node::buffer::Leaf<Storage<T, S>> for Leaf<T> {
	fn parent(&self) -> Option<usize> {
		if self.parent == usize::MAX {
			None
		} else {
			Some(self.parent)
		}
	}

	fn set_parent(&mut self, parent: Option<usize>) {
		self.parent = parent.unwrap_or(usize::MAX)
	}

	fn item_count(&self) -> usize {
		self.items.len()
	}

	// fn item<'a>(&'a self, offset: Offset) -> Option<&'a T> where Storage<T, S>: 'a {
	// 	self.items.get(offset.unwrap())
	// }

	fn max_capacity(&self) -> usize {
		M+1
	}

	fn push_right(&mut self, item: T) {
		self.items.push(item)
	}

	fn forget(self) {
		std::mem::forget(self.items)
	}
}

impl<'a, T, S: 'a + cc_traits::Slab<Node<T>>> btree::node::ItemAccess<Storage<T, S>> for &'a Leaf<T> {
	/// Returns the current number of items stored in this node.
	fn item_count(&self) -> usize {
		self.items.len()
	}

	/// Returns a reference to the item with the given offset in the node.
	fn borrow_item(&self, offset: Offset) -> Option<&T> {
		self.items.get(offset.unwrap())
	}
}

impl<'a, T, S: 'a + cc_traits::Slab<Node<T>>> btree::node::LeafRef<Storage<T, S>> for &'a Leaf<T> {
	fn parent(&self) -> Option<usize> {
		if self.parent == usize::MAX {
			None
		} else {
			Some(self.parent)
		}
	}

	fn max_capacity(&self) -> usize {
		M+1
	}
}

impl<'a, T, S: 'a + cc_traits::Slab<Node<T>>> btree::node::LeafConst<'a, Storage<T, S>> for &'a Leaf<T> {
	fn item(&self, offset: Offset) -> Option<&'a T> {
		self.items.get(offset.unwrap())
	}
}

impl<'a, T, S: 'a + cc_traits::Slab<Node<T>>> btree::node::ItemAccess<Storage<T, S>> for &'a mut Leaf<T> {
	/// Returns the current number of items stored in this node.
	fn item_count(&self) -> usize {
		self.items.len()
	}

	/// Returns a reference to the item with the given offset in the node.
	fn borrow_item(&self, offset: Offset) -> Option<&T> {
		self.items.get(offset.unwrap())
	}
}

impl<'a, T, S: 'a + cc_traits::Slab<Node<T>>> btree::node::LeafRef<Storage<T, S>> for &'a mut Leaf<T> {
	fn parent(&self) -> Option<usize> {
		if self.parent == usize::MAX {
			None
		} else {
			Some(self.parent)
		}
	}

	fn max_capacity(&self) -> usize {
		M+1
	}
}

impl<'r, T, S: 'r + cc_traits::SlabMut<Node<T>>> btree::node::LeafMut<'r, Storage<T, S>> for &'r mut Leaf<T> {
	fn set_parent(&mut self, parent: Option<usize>) {
		self.parent = parent.unwrap_or(usize::MAX)
	}

	fn item_mut(&mut self, offset: Offset) -> Option<&mut T> {
		self.items.get_mut(offset.unwrap())
	}

	fn into_item_mut(self, offset: Offset) -> Option<&'r mut T> {
		self.items.get_mut(offset.unwrap())
	}

	fn insert(&mut self, offset: Offset, item: T) {
		self.items.insert(offset.unwrap(), item)
	}

	fn remove(&mut self, offset: Offset) -> T {
		self.items.remove(offset.unwrap())
	}

	fn append(&mut self, separator: T, mut other: Leaf<T>) -> Offset {
		let offset = self.items.len().into();
		self.items.push(separator);
		self.items.append(&mut other.items);
		offset
	}
}