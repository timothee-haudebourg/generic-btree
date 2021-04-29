use std::{
	marker::PhantomData,
	borrow::Borrow,
	ops::Deref
};
use crate::util::binary_search_min;
use super::{
	Storage,
	StorageMut,
	Offset,
	StorageItem,
	ItemAccess,
	ItemRef,
	ItemMut
};

/// Internal node reference.
pub trait InternalRef<'a, S: 'a + Storage>: ItemAccess<'a, S> + Sized {
	/// Returns the identifer of the parent node, if any.
	fn parent(&self) -> Option<usize>;

	/// Find the offset of the item matching the given key.
	///
	/// If the key matches no item in this node,
	/// this funtion returns the index and id of the child that may match the key.
	#[inline]
	fn offset_of<Q: ?Sized>(&self, key: &Q) -> Result<Offset, (usize, usize)> where S::Key: Borrow<Q>, Q: Ord {
		match binary_search_min(self, key) {
			Some(i) => {
				let item = self.item(i).unwrap();
				if item.key().deref().borrow() == key {
					Ok(i)
				} else {
					let child_index = 1usize + i.unwrap();
					let id = self.child_id(child_index).unwrap();
					Err((child_index, id))
				}
			},
			None => Err((0, self.child_id(0).unwrap()))
		}
	}

	#[inline]
	fn get<Q: ?Sized>(&self, key: &Q) -> Result<S::ValueRef<'a>, usize> where S::Key: Borrow<Q>, Q: Ord {
		match binary_search_min(self, key) {
			Some(i) => {
				let item = self.item(i).unwrap();
				if item.key().deref().borrow() == key {
					Ok(item.value())
				} else {
					Err(self.child_id(1usize + i.unwrap()).unwrap())
				}
			},
			_ => Err(self.child_id(0).unwrap())
		}
	}

	/// Returns the id of the child with the given index, if any.
	fn child_id(&self, index: usize) -> Option<usize>;

	#[inline]
	fn child_count(&self) -> usize {
		self.item_count() + 1usize
	}

	/// Returns the index of the child with the given id, if any.
	#[inline]
	fn child_index(&self, id: usize) -> Option<usize> {
		let child_count = self.item_count() + 1usize;
		for i in 0..child_count {
			if self.child_id(i).unwrap() == id {
				return Some(i)
			}
		}

		None
	}

	fn children(&'a self) -> Children<'a, S, Self> {
		Children {
			node: self,
			index: 0,
			storage: PhantomData
		}
	}

	/// Returns the maximum capacity of this node.
	/// 
	/// Must be at least 6 for internal nodes, and 7 for leaf nodes.
	/// 
	/// The node is considered overflowing if it contains `max_capacity` items.
	fn max_capacity(&self) -> usize;

	/// Returns the minimum capacity of this node.
	/// 
	/// The node is considered underflowing if it contains less items than this value.
	#[inline]
	fn min_capacity(&self) -> usize {
		self.max_capacity() / 2 - 1
	}

	/// Checks if the node is overflowing.
	/// 
	/// For an internal node, this is when it contains `max_capacity` items.
	/// For a leaf node, this is when it contains `max_capacity + 1` items.
	#[inline]
	fn is_overflowing(&self) -> bool {
		self.item_count() >= self.max_capacity()
	}

	/// Checks if the node is underflowing.
	#[inline]
	fn is_underflowing(&self) -> bool {
		self.item_count() < self.min_capacity()
	}
}

impl<'a, T, S: 'a + Storage> InternalRef<'a, S> for &'a mut T where &'a T: InternalRef<'a, S> {
	fn parent(&self) -> Option<usize> {
		self.parent()
	}

	fn child_id(&self, index: usize) -> Option<usize> {
		self.child_id(index)
	}

	fn max_capacity(&self) -> usize {
		self.max_capacity()
	}
}

pub trait InternalMut<'a, S: 'a + StorageMut>: InternalRef<'a, S> {
	fn set_parent(&mut self, parent: Option<usize>);

	fn set_first_child(&mut self, id: usize);

	/// Returns a mutable reference to the item with the given offset in the node.
	fn into_item_mut(self, offset: Offset) -> Option<S::ItemMut<'a>>;

	fn insert(&mut self, offset: Offset, item: StorageItem<S>, right_child_id: usize);

	fn remove(&mut self, offset: Offset) -> (StorageItem<S>, usize);

	fn replace(&mut self, offset: Offset, item: StorageItem<S>) -> StorageItem<S>;

	fn append(&mut self, separator: StorageItem<S>, other: S::InternalNode) -> Offset;

	#[inline]
	fn get_mut<Q: ?Sized>(self, key: &Q) -> Result<S::ValueMut<'a>, usize> where <S as Storage>::Key: Borrow<Q>, Q: Ord {
		match binary_search_min(&self, key) {
			Some(i) => {
				let child_id = self.child_id(1usize + i.unwrap());
				let item = self.into_item_mut(i).unwrap();
				if item.key().deref().borrow() == key {
					Ok(item.into_value_mut())
				} else {
					Err(child_id.unwrap())
				}
			},
			_ => Err(self.child_id(0).unwrap())
		}
	}

	#[inline]
	fn split(&mut self) -> (usize, StorageItem<S>, S::InternalNode) {
		use crate::btree::node::buffer::Internal;
		assert!(self.is_overflowing()); // implies self.other_children.len() >= 4

		// Index of the median-key item in `other_children`.
		let median_i = (self.item_count() - 1) / 2; // Since the knuth-order is at least 3, `median_i` is at least 1.

		// Put all the branches on the right of the median pivot in `right_branches`.
		let right_len = self.item_count() - median_i - 1;
		let mut right_branches = Vec::new(); // Note: branches are stored in reverse order.
		for i in 0..right_len {
			let offset: Offset = (median_i + right_len - i).into();
			let (item, right_child_id) = self.remove(offset);
			right_branches.push((item, right_child_id));
		}

		let mut right_node = S::InternalNode::default();

		// Remove the median pivot.
		let (median_item, median_right_child) = self.remove(median_i.into());
		right_node.set_first_child(median_right_child);

		// Move the right branches to the other node.
		for (item, child_id) in right_branches.into_iter().rev() {
			right_node.push_right(item, child_id);
		}

		assert!(!self.is_underflowing());
		// assert!(!right_node.is_underflowing());

		(self.item_count(), median_item, right_node)
	}
}

pub struct Children<'a, S: 'a + Storage, R: InternalRef<'a, S>> where S::Key: 'a, S::Value: 'a {
	node: &'a R,
	index: usize,
	storage: PhantomData<S>
}

impl<'a, S: 'a + Storage, R: InternalRef<'a, S>> Iterator for Children<'a, S, R> {
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