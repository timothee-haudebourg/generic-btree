use std::{
	ops::{
		Deref,
		DerefMut,
		RangeBounds
	},
	cmp::Ordering,
	hash::{
		Hash,
		Hasher
	},
	borrow::Borrow
};

pub mod node;
mod iter;
// mod entry;

use node::{
	Balance,
	Address,
	Offset,
	Item,
	WouldUnderflow,
	item::{Replace, Mut as ItemMut}
};
pub use iter::{
	Iter,
	IterMut,
	IntoIter,
	// Keys,
	// IntoKeys,
	// Values,
	// IntoValues,
	// ValuesMut,
	Range,
	RangeMut,
	DrainFilter
};
pub(crate) use iter::DrainFilterInner;
// pub use entry::{
// 	Entry,
// 	VacantEntry,
// 	OccupiedEntry,
// 	EntriesMut
// };

/// B-Tree validation error.
#[derive(Debug)]
pub enum ValidationError {
	/// Some node is missing.
	MissingNode(usize),

	/// The leaves of the tree have different depths.
	NotBalanced,

	/// A node is referenced as the child of some node, but it is declared with a different parent.
	/// 
	/// The first parameter is the node id,
	/// then the found parent id referencing the node as a child,
	/// then the expected parent id.
	WrongParent(usize, Option<usize>, Option<usize>), // (node id, found parent, expected parent)

	/// The given node is overflowing.
	Overflow(usize),

	/// The given node is underflowing.
	Underflow(usize),

	/// The items inside the given node are not sorted.
	UnsortedNode(usize),

	/// The smallest item key of the node is smaller than the left separator of the node.
	UnsortedFromLeft(usize),

	/// The greatest item key of the node is greater than the right separator of the node.
	UnsortedFromRight(usize),
}

pub trait ItemPartialOrd<T: ?Sized>: Storage {
	fn item_partial_cmp<'r>(item: &Self::ItemRef<'r>, other: &T) -> Option<Ordering> where Self: 'r;
}

pub trait ItemOrd: Storage {
	fn item_cmp<'r, 's>(item: &Self::ItemRef<'r>, other: &Self::ItemRef<'s>) -> Ordering where Self: 'r + 's;
}

/// Data storage.
pub trait Storage: Sized {
	/// Item reference.
	type ItemRef<'r>: 'r where Self: 'r;

	/// Leaf node reference.
	type LeafRef<'r>: 'r + node::LeafConst<'r, Self> where Self: 'r;

	/// Internal node reference.
	type InternalRef<'r>: 'r + node::InternalConst<'r, Self> where Self: 'r;

	/// Get the root node id.
	///
	/// Returns `None` if the tree is empty.
	fn root(&self) -> Option<usize>;

	/// Returns the number of items in the B-Tree.
	fn len(&self) -> usize;

	/// Returns `true` if the map contains no elements.
	#[inline]
	fn is_empty(&self) -> bool {
		self.root().is_none()
	}

	/// Returns the node with the given id, if any.
	fn node<'r>(&'r self, id: usize) -> Option<node::Ref<'r, Self>>;

	/// Returns the key-value pair corresponding to the supplied key.
	///
	/// The supplied key may be any borrowed form of the map's key type, but the ordering
	/// on the borrowed form *must* match the ordering on the key type.
	#[inline]
	fn get<Q: ?Sized>(&self, key: &Q) -> Option<Self::ItemRef<'_>> where Self: ItemPartialOrd<Q> {
		match self.root() {
			Some(id) => self.get_in(key, id),
			None => None
		}
	}

	/// Get a reference to the value associated to the given `key` in the node `id`, if any.
	#[inline]
	fn get_in<Q: ?Sized>(&self, key: &Q, mut id: usize) -> Option<Self::ItemRef<'_>> where Self: ItemPartialOrd<Q> {
		loop {
			let node = self.node(id).unwrap();
			match node.get(key) {
				Ok(value_opt) => return value_opt,
				Err(child_id) => {
					id = child_id
				}
			}
		}
	}

	fn item(&self, addr: Address) -> Option<Self::ItemRef<'_>> {
		self.node(addr.id).map(|node| node.item(addr.offset)).flatten()
	}

	/// Returns the item corresponding to the supplied key.
	///
	/// The supplied key may be any borrowed form of the map's key type, but the ordering
	/// on the borrowed form *must* match the ordering on the key type.
	#[inline]
	fn get_item<'a, Q: ?Sized>(&'a self, k: &Q) -> Option<Self::ItemRef<'a>>
	where Self: ItemPartialOrd<Q>
	{
		match self.address_of(k) {
			Ok(addr) => self.item(addr),
			Err(_) => None
		}
	}

	/// Returns the first key-value pair in the map.
	/// The key in this pair is the minimum key in the map.
	#[inline]
	fn first_item(&self) -> Option<Self::ItemRef<'_>> {
		match self.first_item_address() {
			Some(addr) => self.item(addr),
			None => None
		}
	}

	/// Returns the last key-value pair in the map.
	/// The key in this pair is the maximum key in the map.
	///
	/// # Examples
	///
	/// Basic usage:
	///
	/// ```
	/// use generic_btree::slab::Map;
	///
	/// let mut map = Map::new();
	/// map.insert(1, "b");
	/// map.insert(2, "a");
	/// assert_eq!(map.last_key_value(), Some((&2, &"a")));
	/// ```
	#[inline]
	fn last_item(&self) -> Option<Self::ItemRef<'_>> {
		match self.last_item_address() {
			Some(addr) => self.item(addr),
			None => None
		}
	}

	fn first_item_address(&self) -> Option<Address> {
		match self.root() {
			Some(mut id) => loop {
				match self.node(id).unwrap().child_id(0) {
					Some(child_id) => {
						id = child_id
					},
					None => return Some(Address::new(id, 0.into()))
				}
			},
			None => None
		}
	}

	fn first_back_address(&self) -> Address {
		match self.root() {
			Some(mut id) => loop {
				match self.node(id).unwrap().child_id(0) {
					Some(child_id) => {
						id = child_id
					},
					None => return Address::new(id, 0.into()) // TODO FIXME thechnically not the first
				}
			},
			None => Address::nowhere()
		}
	}

	fn last_item_address(&self) -> Option<Address> {
		match self.root() {
			Some(mut id) => loop {
				let node = self.node(id).unwrap();
				let index = node.item_count();
				match node.child_id(index) {
					Some(child_id) => id = child_id,
					None => return Some(Address::new(id, (index - 1).into()))
				}
			},
			None => None
		}
	}

	fn last_valid_address(&self) -> Address {
		match self.root() {
			Some(mut id) => loop {
				let node = self.node(id).unwrap();
				let index = node.item_count();
				match node.child_id(index) {
					Some(child_id) => id = child_id,
					None => return Address::new(id, index.into())
				}
			},
			None => Address::nowhere()
		}
	}

	fn normalize(&self, mut addr: Address) -> Option<Address> {
		if addr.is_nowhere() {
			None
		} else {
			loop {
				let node = self.node(addr.id).unwrap();
				if addr.offset >= node.item_count() {
					match node.parent() {
						Some(parent_id) => {
							addr.offset = self.node(parent_id).unwrap().child_index(addr.id).unwrap().into();
							addr.id = parent_id;
						},
						None => return None
					}
				} else {
					return Some(addr)
				}
			}
		}
	}

	#[inline]
	fn leaf_address(&self, mut addr: Address) -> Address {
		if !addr.is_nowhere() {
			loop {
				let node = self.node(addr.id).unwrap();
				match node.child_id(addr.offset.unwrap()) { // TODO unwrap may fail here!
					Some(child_id) => {
						addr.id = child_id;
						addr.offset = self.node(child_id).unwrap().item_count().into()
					},
					None => break
				}
			}
		}

		addr
	}

	/// Get the address of the item located before this address.
	#[inline]
	fn previous_item_address(&self, mut addr: Address) -> Option<Address> {
		if addr.is_nowhere() {
			return None
		}

		loop {
			let node = self.node(addr.id).unwrap();

			match node.child_id(addr.offset.unwrap()) { // TODO unwrap may fail here.
				Some(child_id) => {
					addr.offset = self.node(child_id).unwrap().item_count().into();
					addr.id = child_id;
				},
				None => {
					loop {
						if addr.offset > 0 {
							addr.offset.decr();
							return Some(addr)
						}

						match self.node(addr.id).unwrap().parent() {
							Some(parent_id) => {
								addr.offset = self.node(parent_id).unwrap().child_index(addr.id).unwrap().into();
								addr.id = parent_id;
							},
							None => return None
						}
					}
				}
			}
		}
	}

	#[inline]
	fn previous_front_address(&self, mut addr: Address) -> Option<Address> {
		if addr.is_nowhere() {
			return None
		}

		loop {
			let node = self.node(addr.id).unwrap();
			match addr.offset.value() {
				Some(offset) => {
					let index = if offset < node.item_count() {
						offset
					} else {
						node.item_count()
					};

					match node.child_id(index) {
						Some(child_id) => {
							addr.offset = (self.node(child_id).unwrap().item_count()).into();
							addr.id = child_id;
						},
						None => {
							addr.offset.decr();
							break
						}
					}
				},
				None => {
					match node.parent() {
						Some(parent_id) => {
							addr.offset = self.node(parent_id).unwrap().child_index(addr.id).unwrap().into();
							addr.offset.decr();
							addr.id = parent_id;
							break
						},
						None => return None
					}
				}
			}
		}

		Some(addr)
	}

	/// Get the address of the item located after this address if any.
	#[inline]
	fn next_item_address(&self, mut addr: Address) -> Option<Address> {
		if addr.is_nowhere() {
			return None
		}

		let item_count = self.node(addr.id).unwrap().item_count();
		if addr.offset < item_count {
			addr.offset.incr();
		} else if addr.offset > item_count {
			return None
		}

		// let original_addr_shifted = addr;

		loop {
			let node = self.node(addr.id).unwrap();

			match node.child_id(addr.offset.unwrap()) { // unwrap may fail here.
				Some(child_id) => {
					addr.offset = 0.into();
					addr.id = child_id;
				},
				None => {
					loop {
						let node = self.node(addr.id).unwrap();

						if addr.offset < node.item_count() {
							return Some(addr)
						}

						match node.parent() {
							Some(parent_id) => {
								addr.offset = self.node(parent_id).unwrap().child_index(addr.id).unwrap().into();
								addr.id = parent_id;
							},
							None => {
								// return Some(original_addr_shifted)
								return None
							}
						}
					}
				}
			}
		}
	}

	#[inline]
	fn next_back_address(&self, mut addr: Address) -> Option<Address> {
		if addr.is_nowhere() {
			return None
		}

		loop {
			let node = self.node(addr.id).unwrap();
			let index = match addr.offset.value() {
				Some(offset) => offset + 1,
				None => 0
			};

			if index <= node.item_count() {
				match node.child_id(index) {
					Some(child_id) => {
						addr.offset = Offset::before();
						addr.id = child_id;
					},
					None => {
						addr.offset = index.into();
						break
					}
				}
			} else {
				match node.parent() {
					Some(parent_id) => {
						addr.offset = self.node(parent_id).unwrap().child_index(addr.id).unwrap().into();
						addr.id = parent_id;
						break
					},
					None => {
						return None
					}
				}
			}
		}

		Some(addr)
	}

	#[inline]
	fn next_item_or_back_address(&self, mut addr: Address) -> Option<Address> {
		if addr.is_nowhere() {
			return None
		}

		let item_count = self.node(addr.id).unwrap().item_count();
		if addr.offset < item_count {
			addr.offset.incr();
		} else if addr.offset > item_count {
			return None
		}

		let original_addr_shifted = addr;

		loop {
			let node = self.node(addr.id).unwrap();

			match node.child_id(addr.offset.unwrap()) { // TODO unwrap may fail here.
				Some(child_id) => {
					addr.offset = 0.into();
					addr.id = child_id;
				},
				None => {
					loop {
						let node = self.node(addr.id).unwrap();

						if addr.offset < node.item_count() {
							return Some(addr)
						}

						match node.parent() {
							Some(parent_id) => {
								addr.offset = self.node(parent_id).unwrap().child_index(addr.id).unwrap().into();
								addr.id = parent_id;
							},
							None => {
								return Some(original_addr_shifted)
							}
						}
					}
				}
			}
		}
	}

	/// Get the address of the given key.
	/// 
	/// Returns `Ok(addr)` if the key is used in the tree.
	/// If the key is not used in the tree then `Err(addr)` is returned,
	/// where `addr` can be used to insert the missing key.
	fn address_of<Q: ?Sized>(&self, key: &Q) -> Result<Address, Address> where Self: ItemPartialOrd<Q> {
		match self.root() {
			Some(id) => self.address_in(id, key),
			None => Err(Address::nowhere())
		}
	}

	fn address_in<Q: ?Sized>(&self, mut id: usize, key: &Q) -> Result<Address, Address> where Self: ItemPartialOrd<Q> {
		loop {
			match self.node(id).unwrap().offset_of(key) {
				Ok(offset) => {
					return Ok(Address { id, offset })
				},
				Err((offset, None)) => {
					return Err(Address::new(id, offset.into()))
				},
				Err((_, Some(child_id))) => {
					id = child_id;
				}
			}
		}
	}

	/// Gets an iterator over the entries of the map, sorted by key.
	#[inline]
	fn iter(&self) -> Iter<Self> {
		Iter::new(self)
	}

	/// Constructs a mutable double-ended iterator over a sub-range of elements in the map.
	/// The simplest way is to use the range syntax `min..max`, thus `range(min..max)` will
	/// yield elements from min (inclusive) to max (exclusive).
	/// The range may also be entered as `(Bound<T>, Bound<T>)`, so for example
	/// `range((Excluded(4), Included(10)))` will yield a left-exclusive, right-inclusive
	/// range from 4 to 10.
	///
	/// # Panics
	///
	/// Panics if range `start > end`.
	/// Panics if range `start == end` and both bounds are `Excluded`.
	#[inline]
	fn range<T: ?Sized, R>(&self, range: R) -> Range<Self>
	where
		T: Ord,
		R: RangeBounds<T>,
		Self: ItemPartialOrd<T>,
	{
		Range::new(self, range)
	}

	// /// Gets an iterator over the keys of the map, in sorted order.
	// #[inline]
	// fn keys(&self) -> Keys<Self> {
	// 	Keys::new(self)
	// }

	// /// Gets an iterator over the values of the map, in order by key.
	// #[inline]
	// fn values(&self) -> Values<Self> {
	// 	Values::new(self)
	// }

	#[inline]
	fn eq<'r, S: Storage>(&'r self, other: &S) -> bool where S: ItemPartialOrd<Self::ItemRef<'r>> {
		if self.len() == other.len() {
			let mut it1 = self.iter();
			let mut it2 = other.iter();

			loop {
				match (it1.next(), it2.next()) {
					(None, None) => break,
					(Some(item1), Some(item2)) => {
						if S::item_partial_cmp(&item2, &item1).map(Ordering::is_ne).unwrap_or(true) {
							return false
						}
					},
					_ => return false
				}
			}

			true
		} else {
			false
		}
	}

	#[inline]
	fn partial_cmp<'r, S: Storage>(&'r self, other: &S) -> Option<Ordering> where S: ItemPartialOrd<Self::ItemRef<'r>> {
		let mut it1 = self.iter();
		let mut it2 = other.iter();

		loop {
			match (it1.next(), it2.next()) {
				(None, None) => return Some(Ordering::Equal),
				(_, None) => return Some(Ordering::Greater),
				(None, _) => return Some(Ordering::Less),
				(Some(item1), Some(item2)) => match S::item_partial_cmp(&item2, &item1) {
					Some(Ordering::Greater) => return Some(Ordering::Less),
					Some(Ordering::Less) => return Some(Ordering::Greater),
					Some(Ordering::Equal) => match S::item_partial_cmp(&item2, &item1) {
						Some(Ordering::Greater) => return Some(Ordering::Less),
						Some(Ordering::Less) => return Some(Ordering::Greater),
						Some(Ordering::Equal) => (),
						None => return None
					},
					None => return None
				}
			}
		}
	}

	#[inline]
	fn cmp(&self, other: &Self) -> Ordering where Self: ItemOrd {
		let mut it1 = self.iter();
		let mut it2 = other.iter();

		loop {
			match (it1.next(), it2.next()) {
				(None, None) => return Ordering::Equal,
				(_, None) => return Ordering::Greater,
				(None, _) => return Ordering::Less,
				(Some(item1), Some(item2)) => match Self::item_cmp(&item2, &item1) {
					Ordering::Greater => return Ordering::Less,
					Ordering::Less => return Ordering::Greater,
					Ordering::Equal => match Self::item_cmp(&item2, &item1) {
						Ordering::Greater => return Ordering::Less,
						Ordering::Less => return Ordering::Greater,
						Ordering::Equal => ()
					}
				}
			}
		}
	}

	#[inline]
	fn hash<'r, H: Hasher>(&'r self, h: &mut H) where Self::ItemRef<'r>: Hash {
		for item in self.iter() {
			item.hash(h);
		}
	}

	/// Write the tree in the DOT graph description language.
	///
	/// Requires the `dot` feature.
	#[cfg(feature = "dot")]
	#[inline]
	fn dot_write<W: std::io::Write>(&self, f: &mut W) -> std::io::Result<()> where for<'r> Self::ItemRef<'r>: crate::dot::Display {
		write!(f, "digraph tree {{\n\tnode [shape=record];\n")?;
		match self.root() {
			Some(id) => self.dot_write_node(f, id)?,
			None => ()
		}
		write!(f, "}}")
	}

	/// Write the given node in the DOT graph description language.
	///
	/// Requires the `dot` feature.
	#[cfg(feature = "dot")]
	#[inline]
	fn dot_write_node<W: std::io::Write>(&self, f: &mut W, id: usize) -> std::io::Result<()> where for<'r> Self::ItemRef<'r>: crate::dot::Display {
		let name = format!("n{}", id);
		let node = self.node(id).unwrap();

		write!(f, "\t{} [label=\"", name)?;
		if let Some(parent) = node.parent() {
			write!(f, "({})|", parent)?;
		}

		// node.dot_write_label(f)?;
		use crate::dot::Display;
		write!(f, "{}({})\"];\n", node.dot(), id)?;

		for child_id in node.children() {
			self.dot_write_node(f, child_id)?;
			let child_name = format!("n{}", child_id);
			write!(f, "\t{} -> {}\n", name, child_name)?;
		}

		Ok(())
	}

	#[cfg(debug_assertions)]
	fn validate(&self) -> Result<(), ValidationError> where Self: ItemOrd {
		match self.root() {
			Some(id) => {
				self.validate_node(id, None, None, None)?;
			},
			None => ()
		}

		Ok(())
	}

	/// Validate the given node and returns the depth of the node.
	#[cfg(debug_assertions)]
	fn validate_node<'a>(&'a self, id: usize, parent: Option<usize>, min: Option<Self::ItemRef<'a>>, max: Option<Self::ItemRef<'a>>) -> Result<usize, ValidationError> where Self: ItemOrd {
		let node = self.node(id).ok_or(ValidationError::MissingNode(id))?;
		let (mut min, mut max) = node.validate(id, parent, min, max)?;

		let mut depth = None;
		for (i, child_id) in node.children().enumerate() {
			let (child_min, child_max) = node.separators(i);
			let min = child_min.or_else(|| min.take());
			let max = child_max.or_else(|| max.take());

			let child_depth = self.validate_node(child_id, Some(id), min, max)?;
			match depth {
				None => depth = Some(child_depth),
				Some(depth) => {
					if depth != child_depth {
						return Err(ValidationError::NotBalanced)
					}
				}
			}
		}

		Ok(match depth {
			Some(depth) => depth + 1,
			None => 0
		})
	}
}

/// Mutable data storage.
/// 
/// # Correctness
/// 
/// When using a mutable reference to the key of an item of the B-Tree,
/// the user is responsible of keeping the well sortedness of the
/// items.
/// 
/// # Safety
/// 
/// Implementations of this trait must ensure that
/// two items with different addresses do not alias.
pub unsafe trait StorageMut: Storage {
	type Item;
	type LeafNode: node::buffer::Leaf<Self>;
	type InternalNode: node::buffer::Internal<Self>;

	type ItemMut<'r>: 'r + node::item::Mut<Self> where Self: 'r;
	type LeafMut<'r>: 'r + node::LeafMut<'r, Self> where Self: 'r;
	type InternalMut<'r>: 'r + node::InternalMut<'r, Self> where Self: 'r;

	/// Sets the roo node by id.
	fn set_root(&mut self, root: Option<usize>);

	/// Update the length of the B-Tree.
	fn set_len(&mut self, new_len: usize);

	/// Increments the length of the B-Tree by 1.
	fn incr_len(&mut self) {
		self.set_len(self.len() + 1)
	}

	/// Decrements the length of the B-Tree by 1.
	fn decr_len(&mut self) {
		self.set_len(self.len() - 1)
	}

	fn allocate_node(&mut self, node: node::Buffer<Self>) -> usize;

	/// Allocate the given node and setup its children parent id.
	fn insert_node(&mut self, node: node::Buffer<Self>) -> usize {
		let child_count = node.child_count();
		let id = self.allocate_node(node);
		
		for i in 0..child_count {
			let child_id = self.node(id).unwrap().child_id(i).unwrap();
			self.node_mut(child_id).unwrap().set_parent(Some(id))
		}

		id
	}

	/// Remove the node with the given `id`.
	/// 
	/// # Panic
	/// 
	/// This funciton panics if the node does not exists.
	fn release_node(&mut self, id: usize) -> node::Buffer<Self>;

	/// Returns the node with the given id, if any.
	fn node_mut(&mut self, id: usize) -> Option<node::Mut<'_, Self>>;

	fn item_mut(&mut self, addr: Address) -> Option<Self::ItemMut<'_>> {
		self.node_mut(addr.id).map(|node| node.into_item_mut(addr.offset)).flatten()
	}

	/// Returns a mutable reference to the value corresponding to the key.
	///
	/// The key may be any borrowed form of the map's key type, but the ordering
	/// on the borrowed form *must* match the ordering on the key type.
	#[inline]
	fn get_mut<Q: ?Sized>(&mut self, key: &Q) -> Option<Self::ItemMut<'_>> where Self: ItemPartialOrd<Q> {
		let addr = self.address_of(key);
		match addr {
			Ok(addr) => Some(self.item_mut(addr).unwrap()),
			Err(_) => None
		}
	}

	/// Gets an iterator over the mutable entries of the map, sorted by key.
	#[inline]
	fn iter_mut(&mut self) -> IterMut<Self> {
		IterMut::new(self)
	}

	// /// Gets the given key's corresponding entry in the map for in-place manipulation.
	// #[inline]
	// fn entry(&mut self, key: Self::Key) -> Entry<Self> where Self::Key: Ord {
	// 	match self.address_of(&key) {
	// 		Ok(addr) => {
	// 			Entry::Occupied(OccupiedEntry {
	// 				map: self,
	// 				addr
	// 			})
	// 		},
	// 		Err(addr) => {
	// 			Entry::Vacant(VacantEntry {
	// 				map: self,
	// 				key,
	// 				addr
	// 			})
	// 		}
	// 	}
	// }

	// /// Returns the first entry in the map for in-place manipulation.
	// /// The key of this entry is the minimum key in the map.
	// ///
	// /// # Example
	// ///
	// /// ```
	// /// use generic_btree::slab::Map;
	// ///
	// /// let mut map = Map::new();
	// /// map.insert(1, "a");
	// /// map.insert(2, "b");
	// /// if let Some(mut entry) = map.first_entry() {
	// ///     if *entry.key() > 0 {
	// ///         entry.insert("first");
	// ///     }
	// /// }
	// /// assert_eq!(*map.get(&1).unwrap(), "first");
	// /// assert_eq!(*map.get(&2).unwrap(), "b");
	// /// ```
	// #[inline]
	// fn first_entry(&mut self) -> Option<OccupiedEntry<Self>> {
	// 	match self.first_item_address() {
	// 		Some(addr) => {
	// 			Some(OccupiedEntry {
	// 				map: self,
	// 				addr
	// 			})
	// 		},
	// 		None => None
	// 	}
	// }

	// /// Returns the last entry in the map for in-place manipulation.
	// /// The key of this entry is the maximum key in the map.
	// ///
	// /// # Example
	// ///
	// /// ```
	// /// use generic_btree::slab::Map;
	// ///
	// /// let mut map = Map::new();
	// /// map.insert(1, "a");
	// /// map.insert(2, "b");
	// /// if let Some(mut entry) = map.last_entry() {
	// ///     if *entry.key() > 0 {
	// ///         entry.insert("last");
	// ///     }
	// /// }
	// /// assert_eq!(*map.get(&1).unwrap(), "a");
	// /// assert_eq!(*map.get(&2).unwrap(), "last");
	// /// ```
	// #[inline]
	// fn last_entry(&mut self) -> Option<OccupiedEntry<Self>> {
	// 	match self.last_item_address() {
	// 		Some(addr) => {
	// 			Some(OccupiedEntry {
	// 				map: self,
	// 				addr
	// 			})
	// 		},
	// 		None => None
	// 	}
	// }

	// /// Gets a mutable iterator over the entries of the map, sorted by key, that allows insertion and deletion of the iterated entries.
	// #[inline]
	// fn entries_mut(&mut self) -> EntriesMut<Self> {
	// 	EntriesMut::new(self)
	// }

	/// Constructs a mutable double-ended iterator over a sub-range of elements in the map.
	/// The simplest way is to use the range syntax `min..max`, thus `range(min..max)` will
	/// yield elements from min (inclusive) to max (exclusive).
	/// The range may also be entered as `(Bound<T>, Bound<T>)`, so for example
	/// `range((Excluded(4), Included(10)))` will yield a left-exclusive, right-inclusive
	/// range from 4 to 10.
	///
	/// # Panics
	///
	/// Panics if range `start > end`.
	/// Panics if range `start == end` and both bounds are `Excluded`.
	#[inline]
	fn range_mut<T: ?Sized, R>(&mut self, range: R) -> RangeMut<Self>
	where
		T: Ord,
		R: RangeBounds<T>,
		Self: ItemPartialOrd<T>,
	{
		RangeMut::new(self, range)
	}

	// /// Gets a mutable iterator over the values of the map, in order by key.
	// ///
	// /// # Example
	// ///
	// /// ```
	// /// use generic_btree::slab::Map;
	// ///
	// /// let mut a = Map::new();
	// /// a.insert(1, String::from("hello"));
	// /// a.insert(2, String::from("goodbye"));
	// ///
	// /// for value in a.values_mut() {
	// ///     value.push_str("!");
	// /// }
	// ///
	// /// let values: Vec<String> = a.values().cloned().collect();
	// /// assert_eq!(values, [String::from("hello!"),
	// ///                     String::from("goodbye!")]);
	// /// ```
	// #[inline]
	// fn values_mut(&mut self) -> ValuesMut<Self> {
	// 	ValuesMut::new(self)
	// }

	/// Insert a key-value pair in the tree.
	#[inline]
	fn insert<'a, T>(&'a mut self, item: T) -> Option<<Self::ItemMut<'a> as Replace<Self, T>>::Output>
	where
		Self: Insert<T> + ItemPartialOrd<T>,
		Self::ItemMut<'a>: Replace<Self, T>
	{
		match self.address_of(&item) {
			Ok(addr) => {
				Some(self.replace_at(addr, item))
			},
			Err(addr) => {
				let allocated_item = self.allocate_item(item);
				self.insert_exactly_at(addr, allocated_item, None);
				None
			}
		}
	}

	fn insert_at(&mut self, addr: Address, item: Self::Item) -> Address {
		self.insert_exactly_at(self.leaf_address(addr), item, None)
	}

	fn insert_exactly_at(&mut self, addr: Address, item: Self::Item, opt_right_id: Option<usize>) -> Address {
		if addr.is_nowhere() {
			if self.is_empty() {
				let new_root = node::Buffer::leaf(None, item);
				let id = self.insert_node(new_root);
				self.set_root(Some(id));
				self.incr_len();
				Address { id, offset: 0.into() }
			} else {
				panic!("invalid item address")
			}
		} else {
			if self.is_empty() {
				panic!("invalid item address")
			} else {
				self.node_mut(addr.id).unwrap().insert(addr.offset, item, opt_right_id);
				let new_addr = self.rebalance(addr.id, addr);
				self.incr_len();
				new_addr
			}
		}
	}

	// /// Replace a key-value pair in the tree.
	// #[inline]
	// fn replace(&mut self, item: Self::Item) -> Option<Item<Self::Key, Self::Value>> where Self::Key: Ord {
	// 	match self.address_of(&key) {
	// 		Ok(addr) => {
	// 			Some(self.replace_at(addr, key, value))
	// 		},
	// 		Err(addr) => {
	// 			self.insert_exactly_at(addr, Item::new(key, value), None);
	// 			None
	// 		}
	// 	}
	// }

	fn replace_at<'a, T>(&'a mut self, addr: Address, item: T) -> <Self::ItemMut<'a> as Replace<Self, T>>::Output where Self::ItemMut<'a>: Replace<Self, T> {
		self.node_mut(addr.id).unwrap().into_item_mut(addr.offset).unwrap().replace(item)
	}

	// /// Removes and returns the first element in the map.
	// /// The key of this element is the minimum key that was in the map.
	// ///
	// /// # Example
	// ///
	// /// Draining elements in ascending order, while keeping a usable map each iteration.
	// ///
	// /// ```
	// /// use generic_btree::slab::Map;
	// ///
	// /// let mut map = Map::new();
	// /// map.insert(1, "a");
	// /// map.insert(2, "b");
	// /// while let Some((key, _val)) = map.pop_first() {
	// ///     assert!(map.iter().all(|(k, _v)| *k > key));
	// /// }
	// /// assert!(map.is_empty());
	// /// ```
	// #[inline]
	// fn pop_first(&mut self) -> Option<Item<Self::Key, Self::Value>> {
	// 	self.first_entry().map(|entry| entry.remove_entry())
	// }

	// /// Removes and returns the last element in the map.
	// /// The key of this element is the maximum key that was in the map.
	// ///
	// /// # Example
	// ///
	// /// Draining elements in descending order, while keeping a usable map each iteration.
	// ///
	// /// ```
	// /// use generic_btree::slab::Map;
	// ///
	// /// let mut map = Map::new();
	// /// map.insert(1, "a");
	// /// map.insert(2, "b");
	// /// while let Some((key, _val)) = map.pop_last() {
	// ///     assert!(map.iter().all(|(k, _v)| *k < key));
	// /// }
	// /// assert!(map.is_empty());
	// /// ```
	// #[inline]
	// fn pop_last(&mut self) -> Option<Item<Self::Key, Self::Value>> {
	// 	self.last_entry().map(|entry| entry.remove_entry())
	// }

	/// Removes a key from the map, returning the value at the key if the key
	/// was previously in the map.
	///
	/// The key may be any borrowed form of the map's key type, but the ordering
	/// on the borrowed form *must* match the ordering on the key type.
	///
	/// # Example
	///
	/// ```
	/// use generic_btree::slab::Map;
	///
	/// let mut map = Map::new();
	/// map.insert(1, "a");
	/// assert_eq!(map.remove(&1), Some("a"));
	/// assert_eq!(map.remove(&1), None);
	/// ```
	#[inline]
	fn remove<Q: ?Sized>(&mut self, key: &Q) -> Option<Self::Item> where Self: ItemPartialOrd<Q> {
		match self.address_of(key) {
			Ok(addr) => {
				let (item, _) = self.remove_at(addr).unwrap();
				Some(item)
			},
			Err(_) => None
		}
	}

	#[inline]
	fn remove_at(&mut self, addr: Address) -> Option<(Self::Item, Address)> {
		self.decr_len();
		let item = self.node_mut(addr.id).unwrap().leaf_remove(addr.offset);
		match item {
			Some(Ok(item)) => { // removed from a leaf.
				let addr = self.rebalance(addr.id, addr);
				Some((item, addr))
			},
			Some(Err(left_child_id)) => { // remove from an internal node.
				let new_addr = self.next_item_or_back_address(addr).unwrap();
				let (separator, leaf_id) = self.remove_rightmost_leaf_of(left_child_id);
				let item = self.node_mut(addr.id).unwrap().replace(addr.offset, separator);
				let addr = self.rebalance(leaf_id, new_addr);
				Some((item, addr))
			},
			None => None
		}
	}

	#[inline]
	fn remove_rightmost_leaf_of(&mut self, mut id: usize) -> (Self::Item, usize) {
		loop {
			match self.node_mut(id).unwrap().remove_rightmost_leaf() {
				Ok(result) => return (result, id),
				Err(child_id) => {
					id = child_id;
				}
			}
		}
	}

	/// Removes a key from the map, returning the stored key and value if the key
	/// was previously in the map.
	///
	/// The key may be any borrowed form of the map's key type, but the ordering
	/// on the borrowed form *must* match the ordering on the key type.
	///
	/// # Example
	///
	/// Basic usage:
	///
	/// ```
	/// use generic_btree::slab::Map;
	///
	/// let mut map = Map::new();
	/// map.insert(1, "a");
	/// assert_eq!(map.remove_entry(&1), Some((1, "a")));
	/// assert_eq!(map.remove_entry(&1), None);
	/// ```
	#[inline]
	fn remove_entry<Q: ?Sized>(&mut self, key: &Q) -> Option<Self::Item> where Self: ItemPartialOrd<Q> {
		match self.address_of(key) {
			Ok(addr) => {
				let (item, _) = self.remove_at(addr).unwrap();
				Some(item)
			},
			Err(_) => None
		}
	}

	/// Removes and returns the binding in the map, if any, of which key matches the given one.
	#[inline]
	fn take<Q: ?Sized>(&mut self, key: &Q) -> Option<Self::Item> where Self: ItemPartialOrd<Q> {
		match self.address_of(key) {
			Ok(addr) => {
				let (item, _) = self.remove_at(addr).unwrap();
				Some(item)
			},
			Err(_) => None
		}
	}

	// /// General-purpose update function.
	// ///
	// /// This can be used to insert, compare, replace or remove the value associated to the given
	// /// `key` in the tree.
	// /// The action to perform is specified by the `action` function.
	// /// This function is called once with:
	// ///  - `Some(value)` when `value` is aready associated to `key` or
	// ///  - `None` when the `key` is not associated to any value.
	// ///
	// /// The `action` function must return a pair (`new_value`, `result`) where
	// /// `new_value` is the new value to be associated to `key`
	// /// (if it is `None` any previous binding is removed) and
	// /// `result` is the value returned by the entire `update` function call.
	// #[inline]
	// fn update<T, F>(&mut self, key: Self::Key, action: F) -> T where Self::Key: Ord, F: FnOnce(Option<Self::Value>) -> (Option<Self::Value>, T) {
	// 	match self.root() {
	// 		Some(id) => self.update_in(id, key, action),
	// 		None => {
	// 			let (to_insert, result) = action(None);

	// 			if let Some(value) = to_insert {
	// 				let new_root = node::Buffer::leaf(None, Item::new(key, value));
	// 				let root_id = self.insert_node(new_root);
	// 				self.set_root(Some(root_id));
	// 				self.incr_len()
	// 			}

	// 			result
	// 		}
	// 	}
	// }

	// fn update_in<T, F>(&mut self, mut id: usize, key: Self::Key, action: F) -> T where Self::Key: Ord, F: FnOnce(Option<Self::Value>) -> (Option<Self::Value>, T) {
	// 	loop {
	// 		let offset = self.node(id).unwrap().offset_of(&key);
	// 		match offset {
	// 			Ok(offset) => unsafe {
	// 				let result = {
	// 					let mut item = self.node_mut(id).unwrap().into_item_mut(offset).unwrap();
	// 					let value = std::ptr::read(item.value().deref());
	// 					let (opt_new_value, result) = action(Some(value));
	// 					if let Some(new_value) = opt_new_value {
	// 						std::ptr::write(item.value_mut().deref_mut(), new_value);
	// 						return result
	// 					}

	// 					result
	// 				};

	// 				let (item, _) = self.remove_at(Address::new(id, offset)).unwrap();
	// 				// item's value has been moved, it must not be dropped again.
	// 				item.forget_value();

	// 				return result
	// 			},
	// 			Err((offset, None)) => {
	// 				let (opt_new_value, result) = action(None);
	// 				if let Some(new_value) = opt_new_value {
	// 					let leaf_addr = Address::new(id, offset.into());
	// 					self.insert_exactly_at(leaf_addr, Item::new(key, new_value), None);
	// 				}

	// 				return result
	// 			},
	// 			Err((_, Some(child_id))) => {
	// 				id = child_id;
	// 			}
	// 		}
	// 	}
	// }

	// fn update_at<T, F>(&mut self, addr: Address, action: F) -> T where Self::Key: Ord, F: FnOnce(Self::Value) -> (Option<Self::Value>, T) {
	// 	unsafe {
	// 		let result = {
	// 			let mut item = self.node_mut(addr.id).unwrap().into_item_mut(addr.offset).unwrap();
	// 			let value = std::ptr::read(item.value().deref());
	// 			let (opt_new_value, result) = action(value);

	// 			if let Some(new_value) = opt_new_value {
	// 				std::ptr::write(item.value_mut().deref_mut(), new_value);
	// 				return result
	// 			}

	// 			result
	// 		};

	// 		let (item, _) = self.remove_at(addr).unwrap();
	// 		// item's value has been moved, it must not be dropped again.
	// 		item.forget_value();

	// 		return result
	// 	}
	// }

	/// Creates an iterator which uses a closure to determine if an element should be removed.
	///
	/// If the closure returns true, the element is removed from the map and yielded.
	/// If the closure returns false, or panics, the element remains in the map and will not be
	/// yielded.
	///
	/// Note that `drain_filter` lets you mutate every value in the filter closure, regardless of
	/// whether you choose to keep or remove it.
	///
	/// If the iterator is only partially consumed or not consumed at all, each of the remaining
	/// elements will still be subjected to the closure and removed and dropped if it returns true.
	///
	/// It is unspecified how many more elements will be subjected to the closure
	/// if a panic occurs in the closure, or a panic occurs while dropping an element,
	/// or if the `DrainFilter` value is leaked.
	#[inline]
	fn drain_filter<F>(&mut self, pred: F) -> DrainFilter<Self, F> where F: FnMut(Self::ItemMut<'_>) -> bool {
		DrainFilter::new(self, pred)
	}

	/// Retains only the elements specified by the predicate.
	///
	/// In other words, remove all pairs `(k, v)` such that `f(&k, &mut v)` returns `false`.
	#[inline]
	fn retain<F>(&mut self, mut f: F)
	where
		F: FnMut(Self::ItemMut<'_>) -> bool,
	{
		self.drain_filter(|item| !f(item));
	}

	/// Rebalance the node with the given id.
	/// 
	/// # Panics
	/// 
	/// This function panics if no node has the given `id`.
	#[inline]
	fn rebalance(&mut self, mut id: usize, mut addr: Address) -> Address {
		let mut balance = self.node(id).unwrap().balance();

		loop {
			match balance {
				Balance::Balanced => {
					break
				},
				Balance::Overflow => {
					assert!(!self.node_mut(id).unwrap().is_underflowing());

					let (median_offset, median, right_node) = self.node_mut(id).unwrap().split();
					let right_id = self.insert_node(right_node);

					let parent = self.node(id).unwrap().parent();
					match parent {
						Some(parent_id) => {
							let mut parent = self.node_mut(parent_id).unwrap();
							let offset = parent.child_index(id).unwrap().into();
							parent.insert(offset, median, Some(right_id));

							// new address.
							if addr.id == id {
								if addr.offset == median_offset {
									addr = Address { id: parent_id, offset }
								} else if addr.offset > median_offset {
									addr = Address {
										id: right_id,
										offset: (addr.offset.unwrap() - median_offset - 1).into()
									}
								}
							} else if addr.id == parent_id {
								if addr.offset >= offset {
									addr.offset.incr()
								}
							}

							id = parent_id;
							balance = parent.balance()
						},
						None => {
							let left_id = id;
							let new_root = node::Buffer::binary(None, left_id, median, right_id);
							let root_id = self.insert_node(new_root);

							self.set_root(Some(root_id));
							self.node_mut(left_id).unwrap().set_parent(Some(root_id));
							self.node_mut(right_id).unwrap().set_parent(Some(root_id));

							// new address.
							if addr.id == id {
								if addr.offset == median_offset {
									addr = Address { id: root_id, offset: 0.into() }
								} else if addr.offset > median_offset {
									addr = Address {
										id: right_id,
										offset: (addr.offset.unwrap() - median_offset - 1).into()
									}
								}
							}

							break
						}
					};
				},
				Balance::Underflow(is_empty) => {
					let parent = self.node(id).unwrap().parent();
					match parent {
						Some(parent_id) => {
							let index = self.node(parent_id).unwrap().child_index(id).unwrap();
							// An underflow append in the child node.
							// First we try to rebalance the tree by rotation.
							if self.try_rotate_left(parent_id, index, &mut addr) || self.try_rotate_right(parent_id, index, &mut addr) {
								break
							} else {
								// Rotation didn't work.
								// This means that all existing child sibling have enough few elements to be merged with this child.
								let (new_balance, new_addr) = self.merge(parent_id, index, addr);
								balance = new_balance;
								addr = new_addr;
								// The `merge` function returns the current balance of the parent node,
								// since it may underflow after the merging operation.
								id = parent_id
							}
						},
						None => {
							// if root is empty.
							if is_empty {
								let first_child = self.node(id).unwrap().child_id(0);
								self.set_root(first_child);

								// update root's parent and addr.
								match self.root() {
									Some(root_id) => {
										let mut root = self.node_mut(root_id).unwrap();
										root.set_parent(None);

										if addr.id == id {
											addr.id = root_id;
											addr.offset = root.item_count().into()
										}
									},
									None => {
										addr = Address::nowhere()
									}
								}

								self.release_node(id);
							}

							break
						}
					}
				}
			}
		}

		addr
	}

	/// Try to rotate left the node `id` to benefits the child number `deficient_child_index`.
	///
	/// Returns true if the rotation succeeded, of false if the target child has no right sibling,
	/// or if this sibling would underflow.
	#[inline]
	fn try_rotate_left(&mut self, id: usize, deficient_child_index: usize, addr: &mut Address) -> bool {
		let pivot_offset = deficient_child_index.into();
		let right_sibling_index = deficient_child_index + 1;
		let (right_sibling_id, deficient_child_id) = {
			let node = self.node(id).unwrap();

			if right_sibling_index >= node.child_count() {
				return false // no right sibling
			}

			(node.child_id(right_sibling_index).unwrap(), node.child_id(deficient_child_index).unwrap())
		};

		let left = self.node_mut(right_sibling_id).unwrap().pop_left();
		match left {
			Ok((opt_child_id, mut value)) => {
				self.node_mut(id).unwrap().into_item_mut(pivot_offset).unwrap().swap(&mut value);
				let left_offset = self.node_mut(deficient_child_id).unwrap().push_right(value, opt_child_id);

				// update opt_child's parent
				if let Some(child_id) = opt_child_id {
					self.node_mut(child_id).unwrap().set_parent(Some(deficient_child_id))
				}

				// update address.
				if addr.id == right_sibling_id { // addressed item is in the right node.
					if addr.offset == 0 {
						// addressed item is moving to pivot.
						addr.id = id;
						addr.offset = pivot_offset;
					} else {
						// addressed item stays on right.
						addr.offset.decr();
					}
				} else if addr.id == id { // addressed item is in the parent node.
					if addr.offset == pivot_offset {
						// addressed item is the pivot, moving to the left (deficient) node.
						addr.id = deficient_child_id;
						addr.offset = left_offset;
					}
				}

				true // rotation succeeded
			},
			Err(WouldUnderflow) => false // the right sibling would underflow.
		}
	}

	/// Try to rotate right the node `id` to benefits the child number `deficient_child_index`.
	///
	/// Returns true if the rotation succeeded, of false if the target child has no left sibling,
	/// or if this sibling would underflow.
	#[inline]
	fn try_rotate_right(&mut self, id: usize, deficient_child_index: usize, addr: &mut Address) -> bool {
		if deficient_child_index > 0 {
			let left_sibling_index = deficient_child_index - 1;
			let pivot_offset = left_sibling_index.into();
			let (left_sibling_id, deficient_child_id) = {
				let node = self.node(id).unwrap();
				(node.child_id(left_sibling_index).unwrap(), node.child_id(deficient_child_index).unwrap())
			};
			let right = self.node_mut(left_sibling_id).unwrap().pop_right();
			match right {
				Ok((left_offset, mut value, opt_child_id)) => {
					self.node_mut(id).unwrap().into_item_mut(pivot_offset).unwrap().swap(&mut value);
					self.node_mut(deficient_child_id).unwrap().push_left(opt_child_id, value);

					// update opt_child's parent
					if let Some(child_id) = opt_child_id {
						self.node_mut(child_id).unwrap().set_parent(Some(deficient_child_id))
					}

					// update address.
					if addr.id == deficient_child_id { // addressed item is in the right (deficient) node.
						addr.offset.incr();
					} else if addr.id == left_sibling_id { // addressed item is in the left node.
						if addr.offset == left_offset {
							// addressed item is moving to pivot.
							addr.id = id;
							addr.offset = pivot_offset;
						}
					} else if addr.id == id { // addressed item is in the parent node.
						if addr.offset == pivot_offset {
							// addressed item is the pivot, moving to the left (deficient) node.
							addr.id = deficient_child_id;
							addr.offset = 0.into();
						}
					}

					true // rotation succeeded
				},
				Err(WouldUnderflow) => false // the left sibling would underflow.
			}
		} else {
			false // no left sibling.
		}
	}

	/// Merge the child `deficient_child_index` in node `id` with one of its direct sibling.
	#[inline]
	fn merge(&mut self, id: usize, deficient_child_index: usize, mut addr: Address) -> (Balance, Address) {
		let offset: Offset = if deficient_child_index > 0 {
			// merge with left sibling
			(deficient_child_index-1).into()
		} else {
			// merge with right sibling
			deficient_child_index.into()
		};

		let (left_id, separator, right_id, balance) = {
			let mut node = self.node_mut(id).unwrap();
			let left_id = node.child_id(offset.unwrap()).unwrap();
			let (item, right_id) = node.remove(offset);
			let balance = node.balance();
			(left_id, item, right_id.unwrap(), balance)
		};

		// update children's parent.
		let right_node = self.release_node(right_id);
		for right_child_id in right_node.children() {
			self.node_mut(right_child_id).unwrap().set_parent(Some(left_id));
		}

		// actually merge.
		let left_offset = self.node_mut(left_id).unwrap().append(separator, right_node);

		// update addr.
		if addr.id == id {
			if addr.offset == offset {
				addr.id = left_id;
				addr.offset = left_offset;
			} else if addr.offset > offset {
				addr.offset.decr();
			}
		} else if addr.id == right_id {
			addr.id = left_id;
			addr.offset = (addr.offset.unwrap() + left_offset.unwrap() + 1).into();
		}

		(balance, addr)
	}

	/// Remove every entry from the map.
	fn clear(&mut self) {
		if let Some(id) = self.root() {
			self.clear_node(id)
		}

		self.set_root(None);
		self.set_len(0)
	}

	fn clear_node(&mut self, id: usize) {
		let node = self.release_node(id);
		for child_id in node.children() {
			self.clear_node(child_id)
		}
	}

	/// Remove every entry from the map without dropping the items.
	fn forget_all(&mut self) {
		if let Some(id) = self.root() {
			self.forget_node(id)
		}

		self.set_root(None);
		self.set_len(0)
	}

	fn forget_node(&mut self, id: usize) {
		let node = self.release_node(id);
		for child_id in node.children() {
			self.forget_node(child_id)
		}
		node.forget()
	}

	/// Moves all elements from `other` into `Self`, leaving `other` empty.
	#[inline]
	fn append(&mut self, other: &mut Self)
	where
		for<'r> Self::ItemRef<'r>: ItemRead<Self>,
		Self: Default + Insert<<Self as StorageMut>::Item> + ItemPartialOrd<Self::Item>
	{
		// Do we have to append anything at all?
		if other.is_empty() {
			return;
		}

		// We can just swap `self` and `other` if `self` is empty.
		if self.is_empty() {
			std::mem::swap(self, other);
			return;
		}

		let other = std::mem::take(other);
		for item in other.into_iter() {
			self.insert(item);
		}
	}

	#[inline]
	fn into_iter(self) -> IntoIter<Self> {
		IntoIter::new(self)
	}

	// /// Creates a consuming iterator visiting all the keys, in sorted order.
	// /// The map cannot be used after calling this.
	// /// The iterator element type is `Self::Key`.
	// #[inline]
	// fn into_keys(self) -> IntoKeys<Self> {
	// 	IntoKeys::new(self)
	// }

	// /// Creates a consuming iterator visiting all the values, in order by key.
	// /// The map cannot be used after calling this.
	// /// The iterator element type is `Self::Value`.
	// #[inline]
	// fn into_values(self) -> IntoValues<Self> {
	// 	IntoValues::new(self)
	// }
}

pub trait Insert<T>: StorageMut {
	fn allocate_item(&mut self, item: T) -> Self::Item;
}

/// Item reference that can be unsafely copied.
/// 
/// This trait is used to optimize the consumption of the tree.
pub trait ItemRead<S: StorageMut> {
	/// Copy the item.
	/// 
	/// # Safety
	/// 
	/// This function is unsafe because
	/// an item may not implement the `Copy` trait.
	/// The caller must ensure that the underlying item
	/// will be disposed of without running `drop`.
	unsafe fn read(&self) -> S::Item;
}