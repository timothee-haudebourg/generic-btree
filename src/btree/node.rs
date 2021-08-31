use std::marker::PhantomData;
use super::{
	Storage,
	StorageMut,
	ItemOrd,
	ItemPartialOrd,
	ValidationError
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
pub use item::ItemAccess;
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

pub enum Desc<L, I> {
	Leaf(L),
	Internal(I)
}

pub struct Reference<S, L, I> {
	desc: Desc<L, I>,
	storage: PhantomData<S>
}

impl<S: Storage, L: LeafRef<S>, I: InternalRef<S>> Reference<S, L, I> {
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

	#[inline]
	pub fn items(&self) -> Items<S, L, I> {
		match &self.desc {
			Desc::Leaf(node) => Items::Leaf(node.items()),
			Desc::Internal(node) => Items::Internal(node.items())
		}
	}

	/// Find the offset of the item matching the given key.
	///
	/// If the key matches no item in this node,
	/// this funtion returns the index and id of the child that may match the key,
	/// or `Err(None)` if it is a leaf.
	#[inline]
	pub fn offset_of<Q: ?Sized>(&self, key: &Q) -> Result<Offset, (usize, Option<usize>)> where S: ItemPartialOrd<Q> {
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

	pub fn first_child_id(&self) -> Option<usize> {
		self.child_id(0)
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

	#[cfg(debug_assertions)]
	pub fn validate<'a>(&self, id: usize, parent: Option<usize>, min: Option<S::ItemRef<'a>>, max: Option<S::ItemRef<'a>>) -> Result<(Option<S::ItemRef<'a>>, Option<S::ItemRef<'a>>), ValidationError> where S: ItemOrd {
		if self.parent() != parent {
			return Err(ValidationError::WrongParent(id, self.parent(), parent))
		}

		if min.is_some() || max.is_some() { // not root
			match self.balance() {
				Balance::Overflow => return Err(ValidationError::Overflow(id)),
				Balance::Underflow(_) => return Err(ValidationError::Underflow(id)),
				_ => ()
			}
		}

		for i in 1..self.item_count() {
			let prev = i-1;
			if S::item_cmp(&self.borrow_item(i.into()).unwrap(), &self.borrow_item(prev.into()).unwrap()).is_lt() {
				return Err(ValidationError::UnsortedNode(id))
			}
		}

		if let Some(min) = &min {
			if let Some(item) = self.borrow_first_item() {
				if S::item_cmp(min, &item).is_ge() {
					return Err(ValidationError::UnsortedFromLeft(id))
				}
			}
		}

		if let Some(max) = &max {
			if let Some(item) = self.borrow_last_item() {
				if S::item_cmp(max, &item).is_le() {
					return Err(ValidationError::UnsortedFromRight(id))
				}
			}
		}

		Ok((min, max))
	}
}

#[cfg(feature = "dot")]
impl<S: Storage, L: LeafRef<S>, I: InternalRef<S>> crate::dot::Display for Reference<S, L, I> where for<'r> S::ItemRef<'r>: crate::dot::Display {
	#[inline]
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match &self.desc {
			Desc::Leaf(node) => {
				for item in node.items() {
					write!(f, "{{{}}}|", item.dot())?
					// write!(f, "{{{}|{}}}|", item.key().deref(), item.value().deref())?;
				}
			},
			Desc::Internal(node) => {
				write!(f, "<c0> |")?;
				for (_i, (_, item, _right)) in node.items().enumerate() {
					write!(f, "{{{}}}|", item.dot())?
					// write!(f, "{{{}|<c{}> {}}} |", item.key().deref(), i, item.value().deref())?;
				}
			}
		}

		Ok(())
	}
}

pub type Ref<'a, S> = Reference<S, <S as Storage>::LeafRef<'a>, <S as Storage>::InternalRef<'a>>;

impl<'a, S: 'a + Storage, L: LeafConst<'a, S>, I: InternalConst<'a, S>> Reference<S, L, I> {
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
	pub fn get<Q: ?Sized>(&self, key: &Q) -> Result<Option<S::ItemRef<'a>>, usize> where S: ItemPartialOrd<Q> {
		match &self.desc {
			Desc::Leaf(leaf) => Ok(leaf.get(key)),
			Desc::Internal(node) => match node.get(key) {
				Ok(value) => Ok(Some(value)),
				Err(e) => Err(e)
			}
		}
	}

	#[inline]
	pub fn separators(&self, i: usize) -> (Option<S::ItemRef<'a>>, Option<S::ItemRef<'a>>) {
		match &self.desc {
			Desc::Leaf(_) => (None, None),
			Desc::Internal(node) => node.separators(i)
		}
	}
}

pub type Mut<'a, S> = Reference<S, <S as StorageMut>::LeafMut<'a>, <S as StorageMut>::InternalMut<'a>>;

impl<'a, S: 'a + StorageMut, L: LeafMut<'a, S>, I: InternalMut<'a, S>> Reference<S, L, I> {
	/// Sets the parent node id.
	pub fn set_parent(&mut self, parent: Option<usize>) {
		match &mut self.desc {
			Desc::Leaf(node) => node.set_parent(parent),
			Desc::Internal(node) => node.set_parent(parent)
		}
	}

	/// Sets the first child of the node.
	/// 
	/// # Panics
	/// 
	/// This function panics if the node is a leeaf node and `child_id` is not `None`,
	/// or if the node is an internal node and `child_id` is `None`.
	pub fn set_first_child(&mut self, child_id: Option<usize>) {
		match (&mut self.desc, child_id) {
			(Desc::Internal(node), Some(child_id)) => node.set_first_child(child_id),
			(Desc::Internal(_), None) => panic!("first child of internal node cannot be `None`."),
			(Desc::Leaf(_), None) => (),
			(Desc::Leaf(_), Some(_)) => panic!("cannot set first child of a leaf node.")
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
	pub fn into_get_mut<Q: ?Sized>(self, key: &Q) -> Result<Option<S::ItemMut<'a>>, usize> where S: ItemPartialOrd<Q>, Self: 'a {
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
	pub fn insert(&mut self, offset: Offset, item: S::Item, right_child_id: Option<usize>) {
		match &mut self.desc {
			Desc::Leaf(node) => node.insert(offset, item),
			Desc::Internal(node) => node.insert(offset, item, right_child_id.unwrap())
		}
	}

	/// Removes the item at the given offset and returns it
	/// along with the identifier of its associated right child
	/// if the node is an internal node.
	pub fn remove(&mut self, offset: Offset) -> (S::Item, Option<usize>) {
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
	pub fn leaf_remove(&mut self, offset: Offset) -> Option<Result<S::Item, usize>> {
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
	pub fn remove_rightmost_leaf(&mut self) -> Result<S::Item, usize> {
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
	pub fn push_left(&mut self, child_id: Option<usize>, item: S::Item) {
		self.insert(0.into(), item, self.first_child_id());
		self.set_first_child(child_id);
	}

	/// Remove the first item of the node unless it would undeflow.
	#[inline]
	pub fn pop_left(&mut self) -> Result<(Option<usize>, S::Item), WouldUnderflow> {
		if self.item_count() <= self.min_capacity() {
			Err(WouldUnderflow)
		} else {
			let first_child_id = self.first_child_id();
			let (item, child_id) = self.remove(0.into());
			self.set_first_child(child_id);
			Ok((first_child_id, item))
		}
	}

	#[inline]
	pub fn push_right(&mut self, item: S::Item, child_id: Option<usize>) -> Offset {
		let offset: Offset = self.item_count().into();
		self.insert(offset, item, child_id);
		offset
	}

	#[inline]
	pub fn pop_right(&mut self) -> Result<(Offset, S::Item, Option<usize>), WouldUnderflow> {
		if self.item_count() <= self.min_capacity() {
			Err(WouldUnderflow)
		} else {
			let offset: Offset = (self.item_count() - 1).into();
			let (item, right_child_id) = self.remove(offset);
			Ok((offset, item, right_child_id))
		}
	}

	/// Replace the item at the given offset.
	/// 
	/// # Panic
	/// 
	/// This function panics if no item is at the given offset.
	pub fn replace(&mut self, offset: Offset, item: S::Item) -> S::Item {
		match &mut self.desc {
			Desc::Leaf(node) => node.replace(offset, item),
			Desc::Internal(node) => node.replace(offset, item)
		}
	}

	/// Split the node.
	/// Return the length of the node after split, the median item and the right node.
	pub fn split(&mut self) -> (usize, S::Item, Buffer<S>) {
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
	pub fn append(&mut self, separator: S::Item, other: Buffer<S>) -> Offset {
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

pub enum Items<'b, S, L, I> {
	Leaf(leaf::Items<'b, S, L>),
	Internal(internal::Items<'b, S, I>)
}

impl<'b, S: 'b + Storage, L: LeafRef<S>, I: InternalRef<S>> Iterator for Items<'b, S, L, I> {
	type Item = (Option<usize>, S::ItemRef<'b>, Option<usize>);

	#[inline]
	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::Leaf(inner) => inner.next().map(|item| (None, item, None)),
			Self::Internal(inner) => inner.next().map(|(left, item, right)| (Some(left), item, Some(right)))
		}
	}
}

pub enum Children<'b, S, I> {
	Leaf,
	Internal(internal::Children<'b, S, I>)
}

impl<'b, S: Storage, I: InternalRef<S>> Iterator for Children<'b, S, I> {
	type Item = usize;

	#[inline]
	fn next(&mut self) -> Option<usize> {
		match self {
			Children::Leaf => None,
			Children::Internal(inner) => inner.next()
		}
	}
}