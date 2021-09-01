use smallvec::SmallVec;
use std::borrow::Borrow;
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

struct Branch<T> {
	item: T,
	child_id: usize
}

pub struct Internal<T> {
	parent: usize,
	first_child_id: usize,
	branches: SmallVec<[Branch<T>; M]>
}

impl<T> Default for Internal<T> {
	fn default() -> Self {
		Self {
			parent: usize::MAX,
			first_child_id: usize::MAX,
			branches: SmallVec::new()
		}
	}
}

impl<T> Internal<T> {
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
		self.branches.len()
	}

	fn item(&self, offset: Offset) -> Option<&T> {
		self.branches.get(offset.unwrap()).map(|b| &b.item)
	}

	fn child_id(&self, index: usize) -> Option<usize> {
		if index == 0 {
			Some(self.first_child_id)
		} else {
			self.branches.get(index - 1).map(|b| b.child_id)
		}
	}
	
	fn max_capacity(&self) -> usize {
		M
	}

	fn set_first_child(&mut self, id: usize) {
		self.first_child_id = id;
	}

	fn push_right(&mut self, item: T, child: usize) {
		self.branches.push(Branch {
			item,
			child_id: child
		})
	}
}

impl<'s, T, S: cc_traits::SlabMut<Node<T>>> btree::node::buffer::Internal<Storage<T, S>> for Internal<T> {
	fn parent(&self) -> Option<usize> {
		self.parent()
	}

	fn set_parent(&mut self, parent: Option<usize>) {
		self.set_parent(parent)
	}

	fn item_count(&self) -> usize {
		self.item_count()
	}

	fn item<'a>(&'a self, offset: Offset) -> Option<&'a T> where Storage<T, S>: 'a {
		self.item(offset)
	}

	fn child_id(&self, index: usize) -> Option<usize> {
		self.child_id(index)
	}
	
	fn max_capacity(&self) -> usize {
		self.max_capacity()
	}

	fn set_first_child(&mut self, id: usize) {
		self.set_first_child(id)
	}

	fn push_right(&mut self, item: T, child: usize) {
		self.push_right(item, child)
	}

	fn forget(self) {
		std::mem::forget(self.branches)
	}
}

impl<'s, T, S: 's + cc_traits::Slab<Node<T>>> btree::node::ItemAccess<Storage<T, S>> for &'s Internal<T> {
	/// Returns the current number of items stored in this node.
	fn item_count(&self) -> usize {
		(*self).item_count()
	}

	/// Returns a reference to the item with the given offset in the node.
	fn borrow_item(&self, offset: Offset) -> Option<&T> {
		(*self).item(offset)
	}
}

impl<'a, T, S: 'a + cc_traits::Slab<Node<T>>> btree::node::InternalRef<Storage<T, S>> for &'a Internal<T> {
	/// Returns the identifer of the parent node, if any.
	fn parent(&self) -> Option<usize> {
		(*self).parent()
	}

	/// Returns the id of the child with the given index, if any.
	/// 
	/// Note that in the case of leaf nodes, this always return `None`.
	fn child_id(&self, index: usize) -> Option<usize> {
		(*self).child_id(index)
	}

	/// Returns the maximum capacity of this node.
	/// 
	/// Must be at least 6 for internal nodes, and 7 for leaf nodes.
	/// 
	/// The node is considered overflowing if it contains `max_capacity` items.
	fn max_capacity(&self) -> usize {
		(*self).max_capacity()
	}
}

impl<'a, T, S: 'a + cc_traits::Slab<Node<T>>> btree::node::InternalConst<'a, Storage<T, S>> for &'a Internal<T> {
	fn item(&self, offset: Offset) -> Option<&'a T> {
		(*self).item(offset)
	}
}

impl<'a, T, S: 'a + cc_traits::Slab<Node<T>>> btree::node::ItemAccess<Storage<T, S>> for &'a mut Internal<T> {
	/// Returns the current number of items stored in this node.
	fn item_count(&self) -> usize {
		self.branches.len()
	}

	/// Returns a reference to the item with the given offset in the node.
	fn borrow_item(&self, offset: Offset) -> Option<&T> {
		(*self).item(offset)
	}
}

impl<'a, T, S: 'a + cc_traits::Slab<Node<T>>> btree::node::InternalRef<Storage<T, S>> for &'a mut Internal<T> {
	/// Returns the identifer of the parent node, if any.
	fn parent(&self) -> Option<usize> {
		Internal::<T>::parent(self)
	}

	/// Returns the id of the child with the given index, if any.
	/// 
	/// Note that in the case of leaf nodes, this always return `None`.
	fn child_id(&self, index: usize) -> Option<usize> {
		Internal::<T>::child_id(self, index)
	}

	/// Returns the maximum capacity of this node.
	/// 
	/// Must be at least 6 for internal nodes, and 7 for leaf nodes.
	/// 
	/// The node is considered overflowing if it contains `max_capacity` items.
	fn max_capacity(&self) -> usize {
		Internal::<T>::max_capacity(self)
	}
}

impl<'r, T, S: 'r + cc_traits::SlabMut<Node<T>>> btree::node::InternalMut<'r, Storage<T, S>> for &'r mut Internal<T> {
	fn set_parent(&mut self, parent: Option<usize>) {
		(*self).set_parent(parent)
	}

	fn set_first_child(&mut self, id: usize) {
		(*self).set_first_child(id)
	}

	/// Returns a mutable reference to the item with the given offset in the node.
	fn into_item_mut(self, offset: Offset) -> Option<&'r mut T> {
		self.branches.get_mut(offset.unwrap()).map(|branch| &mut branch.item)
	}

	fn insert(&mut self, offset: Offset, item: T, right_child_id: usize) {
		self.branches.insert(offset.unwrap(), Branch {
			item,
			child_id: right_child_id
		})
	}

	fn remove(&mut self, offset: Offset) -> (T, usize) {
		let b = self.branches.remove(offset.unwrap());
		(b.item, b.child_id)
	}

	fn replace(&mut self, offset: Offset, mut item: T) -> T {
		std::mem::swap(&mut self.branches.get_mut(offset.unwrap()).unwrap().item, &mut item);
		item
	}

	fn append(&mut self, separator: T, mut other: Internal<T>) -> Offset {
		let offset = self.branches.len().into();
		self.branches.push(Branch {
			item: separator,
			child_id: other.first_child_id
		});
		self.branches.append(&mut other.branches);
		offset
	}
}