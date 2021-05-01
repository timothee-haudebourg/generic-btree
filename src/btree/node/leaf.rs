use std::{
	borrow::Borrow,
	ops::Deref,
	marker::PhantomData
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

/// Leaf node reference.
pub trait LeafRef<'a, S: 'a + Storage>: ItemAccess<'a, S> {
	/// Returns the identifer of the parent node, if any.
	fn parent(&self) -> Option<usize>;

	/// Find the offset of the item matching the given key.
	#[inline]
	fn offset_of<Q: ?Sized>(&self, key: &Q) -> Result<Offset, Offset> where S::Key: Borrow<Q>, Q: Ord {
		match binary_search_min(self, key) {
			Some(i) => {
				let item = self.borrow_item(i).unwrap();
				if item.key().deref().borrow() == key {
					Ok(i.into())
				} else {
					Err((i+1).into())
				}
			},
			None => Err(0.into())
		}
	}

	fn items(&self) -> Items<S, Self> {
		Items {
			node: self,
			offset: 0.into(),
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
		(self.max_capacity() - 1) / 2 - 1
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

/// Leaf node reference.
pub trait LeafConst<'a, S: 'a + Storage>: LeafRef<'a, S> {
	fn item(&self, offset: Offset) -> Option<S::ItemRef<'a>>;

	#[inline]
	fn get<Q: ?Sized>(&self, key: &Q) -> Option<S::ValueRef<'a>> where S::Key: Borrow<Q>, Q: Ord {
		match binary_search_min(self, key) {
			Some(i) => {
				let item = self.item(i).unwrap();
				if item.key().deref().borrow() == key {
					Some(item.value())
				} else {
					None
				}
			},
			_ => None
		}
	}
}

pub trait LeafMut<'a, S: 'a + StorageMut>: Sized + LeafRef<'a, S> {
	fn set_parent(&mut self, parent: Option<usize>);

	/// Returns a mutable reference to the item with the given offset in the node.
	fn item_mut(&mut self, offset: Offset) -> Option<S::ItemMut<'_>>;

	fn into_item_mut(self, offset: Offset) -> Option<S::ItemMut<'a>>;

	fn insert(&mut self, offset: Offset, item: StorageItem<S>);

	fn remove(&mut self, offset: Offset) -> StorageItem<S>;

	#[inline]
	fn remove_last(&mut self) -> StorageItem<S> {
		let offset = (self.item_count() - 1).into();
		self.remove(offset)
	}

	fn replace(&mut self, offset: Offset, item: StorageItem<S>) -> StorageItem<S> {
		self.item_mut(offset).unwrap().replace(item)
	}

	fn append(&mut self, separator: StorageItem<S>, other: S::LeafNode) -> Offset;

	#[inline]
	fn get_mut<Q: ?Sized>(self, key: &Q) -> Option<S::ValueMut<'a>> where S::Key: Borrow<Q>, Q: Ord {
		use crate::btree::node::item::Mut;
		match binary_search_min(&self, key) {
			Some(i) => {
				let item = self.into_item_mut(i).unwrap();
				if item.key().deref().borrow() == key {
					Some(item.into_value_mut())
				} else {
					None
				}
			},
			_ => None
		}
	}

	#[inline]
	fn split(&mut self) -> (usize, StorageItem<S>, S::LeafNode) {
		use crate::btree::node::buffer::Leaf;
		assert!(self.is_overflowing());

		// Index of the median-key item in `other_children`.
		let median_i = (self.item_count() - 1) / 2; // Since the knuth-order is at least 3, `median_i` is at least 1.

		// Put all the branches on the right of the median pivot in `right_branches`.
		let right_len = self.item_count() - median_i - 1;
		let mut right_branches = Vec::new(); // Note: branches are stored in reverse order.
		for i in 0..right_len {
			let offset: Offset = (median_i + right_len - i).into();
			let item = self.remove(offset);
			right_branches.push(item);
		}

		let mut right_node = S::LeafNode::default();
		right_node.set_parent(self.parent());

		// Remove the median pivot.
		let median_item = self.remove(median_i.into());

		// Move the right branches to the other node.
		for item in right_branches.into_iter().rev() {
			right_node.push_right(item);
		}

		assert!(!self.is_underflowing());
		// assert!(!right_node.is_underflowing());

		(self.item_count(), median_item, right_node)
	}
}

pub struct Items<'b, S, R: ?Sized> {
	node: &'b R,
	offset: Offset,
	storage: PhantomData<S>
}

impl<'a, 'b, S: 'a + Storage, R: LeafRef<'a, S>> Iterator for Items<'b, S, R> where 'a: 'b {
	type Item = S::ItemRef<'b>;

	fn next(&mut self) -> Option<Self::Item> {
		if self.offset < self.node.item_count() {
			let offset = self.offset;
			self.offset = offset + 1;
			self.node.borrow_item(offset)
		} else {
			None
		}
	}
}