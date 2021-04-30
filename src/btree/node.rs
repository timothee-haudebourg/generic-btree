use std::{
	marker::PhantomData,
	ops::{
		Deref
	},
	borrow::Borrow
};
use super::{
	Storage,
	StorageMut
};

mod balance;
mod offset;
mod addr;
mod leaf;
mod internal;
pub mod item;
pub mod buffer;

pub use balance::Balance;
pub use offset::Offset;
pub use addr::Address;
pub use leaf::{
	LeafRef,
	LeafConst,
	LeafMut
};
pub use internal::{
	InternalRef,
	InternalConst,
	InternalMut
};
pub use item::{
	Item,
	ItemAccess,
	StorageItem,
	Ref as ItemRef,
	Mut as ItemMut
};
pub use buffer::Buffer;

/// Node type.
pub enum Type {
	/// Internal node, with child nodes between items.
	Internal,

	/// Leaf node, without children.
	Leaf
}

pub struct WouldUnderflow;

impl Type {
	pub fn is_internal(&self) -> bool {
		match self {
			Self::Internal => true,
			_ => false
		}
	}

	pub fn is_leaf(&self) -> bool {
		match self {
			Self::Leaf => true,
			_ => false
		}
	}
}

pub trait FromStorage {
	type Storage;
}

pub enum Desc<L, I> {
	Leaf(L),
	Internal(I)
}

pub struct Reference<S, L, I> {
	desc: Desc<L, I>,
	storage: PhantomData<S>
}

impl<'a, S: 'a + Storage, L: LeafRef<'a, S>, I: InternalRef<'a, S>> Reference<S, L, I> where S::Key: 'a, S::Value: 'a {
	#[inline]
	pub fn leaf(node: L) -> Self {
		Self {
			desc: Desc::Leaf(node),
			storage: PhantomData
		}
	}

	#[inline]
	pub fn internal(node: I) -> Self {
		Self {
			desc: Desc::Internal(node),
			storage: PhantomData
		}
	}

	#[inline]
	pub fn ty(&self) -> Type {
		match &self.desc {
			Desc::Leaf(_) => Type::Leaf,
			Desc::Internal(_) => Type::Internal
		}
	}

	#[inline]
	pub fn is_internal(&self) -> bool {
		self.ty().is_internal()
	}

	/// Returns the identifer of the parent node, if any.
	pub fn parent(&self) -> Option<usize> {
		match &self.desc {
			Desc::Internal(node) => node.parent(),
			Desc::Leaf(node) => node.parent()
		}
	}

	/// Returns the current number of items stored in this node.
	pub fn item_count(&self) -> usize {
		match &self.desc {
			Desc::Internal(node) => node.item_count(),
			Desc::Leaf(node) => node.item_count()
		}
	}

	/// Returns a reference to the item with the given offset in the node.
	pub fn borrow_item(&self, offset: Offset) -> Option<S::ItemRef<'_>> {
		match &self.desc {
			Desc::Internal(node) => node.borrow_item(offset),
			Desc::Leaf(node) => node.borrow_item(offset)
		}
	}

	#[inline]
	pub fn borrow_first_item(&self) -> Option<S::ItemRef<'_>> {
		self.borrow_item(0.into())
	}

	#[inline]
	pub fn borrow_last_item(&self) -> Option<S::ItemRef<'_>> {
		self.borrow_item((self.item_count()-1).into())
	}

	/// Find the offset of the item matching the given key.
	///
	/// If the key matches no item in this node,
	/// this funtion returns the index and id of the child that may match the key,
	/// or `Err(None)` if it is a leaf.
	#[inline]
	pub fn offset_of<Q: ?Sized>(&self, key: &Q) -> Result<Offset, (usize, Option<usize>)> where S::Key: Borrow<Q>, Q: Ord {
		match &self.desc {
			Desc::Internal(node) => match node.offset_of(key) {
				Ok(i) => Ok(i),
				Err((index, child_id)) => Err((index, Some(child_id)))
			},
			Desc::Leaf(leaf) => match leaf.offset_of(key) {
				Ok(i) => Ok(i),
				Err(index) =>  Err((index.unwrap(), None))
			}
		}
	}

	/// Returns the current number of children.
	#[inline]
	pub fn child_count(&self) -> usize {
		match &self.desc {
			Desc::Internal(node) => node.child_count(),
			Desc::Leaf(_) => 0
		}
	}

	/// Returns the id of the child with the given index, if any.
	/// 
	/// Note that in the case of leaf nodes, this always return `None`.
	pub fn child_id(&self, index: usize) -> Option<usize> {
		match &self.desc {
			Desc::Internal(node) => node.child_id(index),
			Desc::Leaf(_) => None
		}
	}

	/// Returns the index of the child with the given id, if any.
	#[inline]
	pub fn child_index(&self, id: usize) -> Option<usize> {
		match &self.desc {
			Desc::Internal(node) => node.child_index(id),
			Desc::Leaf(_) => None
		}
	}

	#[inline]
	pub fn children(&self) -> Children<S, I> {
		match &self.desc {
			Desc::Leaf(_) => Children::Leaf,
			Desc::Internal(node) => Children::Internal(node.children())
		}
	}

	/// Returns the maximum capacity of this node.
	/// 
	/// Must be at least 6 for internal nodes, and 7 for leaf nodes.
	/// 
	/// The node is considered overflowing if it contains `max_capacity` items.
	pub fn max_capacity(&self) -> usize {
		match &self.desc {
			Desc::Internal(node) => node.max_capacity(),
			Desc::Leaf(node) => node.max_capacity()
		}
	}

	/// Returns the minimum capacity of this node.
	/// 
	/// The node is considered underflowing if it contains less items than this value.
	#[inline]
	pub fn min_capacity(&self) -> usize {
		match &self.desc {
			Desc::Internal(node) => node.min_capacity(),
			Desc::Leaf(node) => node.min_capacity()
		}
	}

	/// Checks if the node is overflowing.
	/// 
	/// For an internal node, this is when it contains `max_capacity` items.
	/// For a leaf node, this is when it contains `max_capacity + 1` items.
	#[inline]
	pub fn is_overflowing(&self) -> bool {
		self.item_count() >= self.max_capacity()
	}

	/// Checks if the node is underflowing.
	#[inline]
	pub fn is_underflowing(&self) -> bool {
		self.item_count() < self.min_capacity()
	}

	/// Returns the current balance of the node.
	#[inline]
	pub fn balance(&self) -> Balance {
		if self.is_overflowing() {
			Balance::Overflow
		} else if self.is_underflowing() {
			Balance::Underflow(self.item_count() == 0)
		} else {
			Balance::Balanced
		}
	}

	/// Write the label of the node in the DOT format.
	///
	/// Requires the `dot` feature.
	#[cfg(feature = "dot")]
	#[inline]
	pub fn dot_write_label<W: std::io::Write>(&self, f: &mut W) -> std::io::Result<()> where K: std::fmt::Display, V: std::fmt::Display {
		write!(f, "<c0> |")?;
		let mut i = 1;
		for branch in &self.other_children {
			write!(f, "{{{}|<c{}> {}}} |", branch.item.key(), i, branch.item.value())?;
			i += 1;
		}

		Ok(())
	}

	#[cfg(debug_assertions)]
	pub fn validate(&self, parent: Option<usize>, min: Option<S::KeyRef<'_>>, max: Option<S::KeyRef<'_>>) where S::Key: Ord {
		if self.parent() != parent {
			panic!("wrong parent")
		}

		if min.is_some() || max.is_some() { // not root
			match self.balance() {
				Balance::Overflow => panic!("node is overflowing"),
				Balance::Underflow(_) => panic!("node is underflowing"),
				_ => ()
			}
		}

		if self.item_count() == 0 {
			panic!("node is empty")
		}

		for i in 1..self.item_count() {
			let prev = i-1;
			if self.borrow_item(i.into()).unwrap().key().deref() < self.borrow_item(prev.into()).unwrap().key().deref() {
				panic!("items are not sorted")
			}
		}

		if let Some(min) = min {
			if let Some(item) = self.borrow_first_item() {
				if min.deref() >= &item.key() {
					panic!("item key is greater than right separator")
				}
			}
		}

		if let Some(max) = max {
			if let Some(item) = self.borrow_last_item() {
				if max.deref() <= &item.key() {
					panic!("item key is less than left separator")
				}
			}
		}
	}
}

pub type Ref<'a, S> = Reference<S, <S as Storage>::LeafRef<'a>, <S as Storage>::InternalRef<'a>>;

impl<'a, S: 'a + Storage, L: LeafConst<'a, S>, I: InternalConst<'a, S>> Reference<S, L, I> where S::Key: 'a, S::Value: 'a {
	/// Returns a reference to the item with the given offset in the node.
	pub fn item(&self, offset: Offset) -> Option<S::ItemRef<'a>> {
		match &self.desc {
			Desc::Internal(node) => node.item(offset),
			Desc::Leaf(node) => node.item(offset)
		}
	}

	#[inline]
	pub fn first_item(&self) -> Option<S::ItemRef<'a>> {
		self.item(0.into())
	}

	#[inline]
	pub fn last_item(&self) -> Option<S::ItemRef<'a>> {
		self.item((self.item_count()-1).into())
	}

	#[inline]
	pub fn get<Q: ?Sized>(&self, key: &Q) -> Result<Option<S::ValueRef<'a>>, usize> where S::Key: Borrow<Q>, Q: Ord {
		match &self.desc {
			Desc::Leaf(leaf) => Ok(leaf.get(key)),
			Desc::Internal(node) => match node.get(key) {
				Ok(value) => Ok(Some(value)),
				Err(e) => Err(e)
			}
		}
	}

	#[inline]
	pub fn separators(&self, i: usize) -> (Option<S::KeyRef<'a>>, Option<S::KeyRef<'a>>) {
		match &self.desc {
			Desc::Leaf(_) => (None, None),
			Desc::Internal(node) => node.separators(i)
		}
	}
}

pub type Mut<'a, S> = Reference<S, <S as StorageMut>::LeafMut<'a>, <S as StorageMut>::InternalMut<'a>>;

impl<'a, S: 'a + StorageMut, L: LeafMut<'a, S>, I: InternalMut<'a, S>> Reference<S, L, I> where S::Key: 'a, S::Value: 'a {
	/// Sets the parent node id.
	pub fn set_parent(&mut self, parent: Option<usize>) {
		match &mut self.desc {
			Desc::Leaf(node) => node.set_parent(parent),
			Desc::Internal(node) => node.set_parent(parent)
		}
	}

	/// Sets the first child of the node.
	/// The node must be an internal node.
	/// 
	/// # Panics
	/// 
	/// This function panics if the node is a leaf.
	pub fn set_first_child(&mut self, child_id: usize) {
		match &mut self.desc {
			Desc::Internal(node) => node.set_first_child(child_id),
			Desc::Leaf(_) => panic!("cannot set first child of a leaf node.")
		}
	}

	/// Returns a mutable reference to the item at the given offset, if any.
	pub fn into_item_mut(self, offset: Offset) -> Option<S::ItemMut<'a>> {
		match self.desc {
			Desc::Leaf(node) => node.into_item_mut(offset),
			Desc::Internal(node) => node.into_item_mut(offset)
		}
	}

	#[inline]
	pub fn into_get_mut<Q: ?Sized>(self, key: &Q) -> Result<Option<S::ValueMut<'a>>, usize> where S::Key: Borrow<Q>, Q: Ord {
		match self.desc {
			Desc::Leaf(leaf) => Ok(leaf.get_mut(key)),
			Desc::Internal(node) => match node.get_mut(key) {
				Ok(value) => Ok(Some(value)),
				Err(e) => Err(e)
			}
		}
	}

	/// Insert the given item at the given offset.
	/// 
	/// If this is an internal node, some `right_child_id` must be given.
	/// 
	/// # Panics
	/// 
	/// This may panics if the offset if greater than the current item count or
	/// if this is an internal node and `right_child_id` is `None`.
	pub fn insert(&mut self, offset: Offset, item: StorageItem<S>, right_child_id: Option<usize>) {
		match &mut self.desc {
			Desc::Leaf(node) => node.insert(offset, item),
			Desc::Internal(node) => node.insert(offset, item, right_child_id.unwrap())
		}
	}

	/// Removes the item at the given offset and returns it
	/// along with the identifier of its associated right child
	/// if the node is an internal node.
	pub fn remove(&mut self, offset: Offset) -> (StorageItem<S>, Option<usize>) {
		match &mut self.desc {
			Desc::Leaf(node) => {
				let item = node.remove(offset);
				(item, None)
			},
			Desc::Internal(node) => {
				let (item, child) = node.remove(offset);
				(item, Some(child))
			}
		}
	}

	#[inline]
	pub fn leaf_remove(&mut self, offset: Offset) -> Option<Result<Item<S::Key, S::Value>, usize>> {
		match &mut self.desc {
			Desc::Internal(node) => {
				if offset < node.item_count() {
					let left_child_index = offset.unwrap();
					Some(Err(node.child_id(left_child_index).unwrap()))
				} else {
					None
				}
			},
			Desc::Leaf(leaf) => {
				if offset < leaf.item_count() {
					Some(Ok(leaf.remove(offset)))
				} else {
					None
				}
			}
		}
	}

	#[inline]
	pub fn remove_rightmost_leaf(&mut self) -> Result<Item<S::Key, S::Value>, usize> {
		match &mut self.desc {
			Desc::Internal(node) => {
				let child_index = node.child_count() - 1;
				let child_id = node.child_id(child_index).unwrap();
				Err(child_id)
			},
			Desc::Leaf(leaf) => Ok(leaf.remove_last())
		}
	}

	#[inline]
	pub fn push_left(&mut self, item: StorageItem<S>, child_id: Option<usize>) {
		self.insert(0.into(), item, child_id)
	}

	/// Remove the first item of the node unless it would undeflow.
	#[inline]
	pub fn pop_left(&mut self) -> Result<(StorageItem<S>, Option<usize>), WouldUnderflow> {
		if self.item_count() <= self.min_capacity() {
			Err(WouldUnderflow)
		} else {
			Ok(self.remove(0.into()))
		}
	}

	#[inline]
	pub fn push_right(&mut self, item: StorageItem<S>, child_id: Option<usize>) -> Offset {
		let offset: Offset = self.item_count().into();
		self.insert(offset, item, child_id);
		offset
	}

	#[inline]
	pub fn pop_right(&mut self) -> Result<(Offset, StorageItem<S>, Option<usize>), WouldUnderflow> {
		if self.item_count() <= self.min_capacity() {
			Err(WouldUnderflow)
		} else {
			let offset: Offset = self.item_count().into();
			let (item, right_child_id) = self.remove(offset);
			Ok((offset, item, right_child_id))
		}
	}

	/// Replace the item at the given offset.
	/// 
	/// # Panic
	/// 
	/// This function panics if no item is at the given offset.
	pub fn replace(&mut self, offset: Offset, item: StorageItem<S>) -> StorageItem<S> {
		match &mut self.desc {
			Desc::Leaf(node) => node.replace(offset, item),
			Desc::Internal(node) => node.replace(offset, item)
		}
	}

	/// Split the node.
	/// Return the length of the node after split, the median item and the right node.
	pub fn split(&mut self) -> (usize, StorageItem<S>, Buffer<S>) {
		match &mut self.desc {
			Desc::Leaf(leaf) => {
				let (len, item, right_node) = leaf.split();
				(len, item, Buffer::Leaf(right_node))
			}
			Desc::Internal(node) => {
				let (len, item, right_leaf) = node.split();
				(len, item, Buffer::Internal(right_leaf))
			}
		}
	}

	/// Append `separator` and the content of the `other` node into this node.
	/// 
	/// Returns the new offset of the `separator`.
	#[inline]
	pub fn append(&mut self, separator: StorageItem<S>, other: Buffer<S>) -> Offset {
		match (&mut self.desc, other) {
			(Desc::Internal(node), Buffer::Internal(other)) => {
				node.append(separator, other)
			},
			(Desc::Leaf(node), Buffer::Leaf(other)) => {
				node.append(separator, other)
			},
			_ => panic!("trying to append incompatible node")
		}
	}
}

pub enum Children<'b, S, I> {
	Leaf,
	Internal(internal::Children<'b, S, I>)
}

impl<'a, 'b, S: 'a + Storage, I: InternalRef<'a, S>> Iterator for Children<'b, S, I> {
	type Item = usize;

	#[inline]
	fn next(&mut self) -> Option<usize> {
		match self {
			Children::Leaf => None,
			Children::Internal(inner) => inner.next()
		}
	}
}