use std::{
	fmt,
	ops::Deref
};
use super::{
	Storage,
	StorageMut,
	Address,
	Item
};

/// A view into a single entry in a map, which may either be vacant or occupied.
///
/// This enum is constructed from the [`entry`](`Map#entry`) method on [`Map`].
pub enum Entry<'a, S: Storage> {
	Vacant(VacantEntry<'a, S>),
	Occupied(OccupiedEntry<'a, S>)
}

use Entry::*;

impl<'a, S: Storage> Entry<'a, S> {
	/// Gets the address of the entry in the B-Tree.
	#[inline]
	pub fn address(&self) -> Address {
		match self {
			Occupied(entry) => entry.address(),
			Vacant(entry) => entry.address()
		}
	}

	/// Returns a reference to this entry's key.
	///
	/// # Examples
	///
	/// ```
	/// use generic_btree::slab::Map;
	///
	/// let mut map: Map<&str, usize> = Map::new();
	/// assert_eq!(map.entry("poneyland").key(), &"poneyland");
	/// ```
	#[inline]
	pub fn key(&self) -> EntryKey<'_, S> {
		match *self {
			Occupied(ref entry) => EntryKey::Occupied(entry.key()),
			Vacant(ref entry) => EntryKey::Vacant(entry.key()),
		}
	}
}

pub enum EntryKey<'a, S: 'a + Storage> {
	Occupied(S::KeyRef<'a>),
	Vacant(&'a S::Key)
}

impl<'a, S: 'a + Storage> Deref for EntryKey<'a, S> {
	type Target = S::Key;

	fn deref(&self) -> &Self::Target {
		match self {
			Self::Occupied(key) => key.deref(),
			Self::Vacant(key) => key
		}
	}
}

impl<'a, S: 'a + Storage> fmt::Debug for EntryKey<'a, S> where S::KeyRef<'a>: fmt::Debug, S::Key: fmt::Debug {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Vacant(key) => key.fmt(f),
			Self::Occupied(key) => key.fmt(f)
		}
	}
}

impl<'a, 'b, S: 'a + Storage, T> PartialEq<&'b T> for EntryKey<'a, S> where S::Key: PartialEq<T> {
	fn eq(&self, other: &&'b T) -> bool {
		self.deref() == *other
	}
}

impl<'a, S: StorageMut> Entry<'a, S> {
	/// Ensures a value is in the entry by inserting the default if empty, and returns
	/// a mutable reference to the value in the entry.
	///
	/// # Examples
	///
	/// ```
	/// use generic_btree::slab::Map;
	///
	/// let mut map: Map<&str, usize> = Map::new();
	/// map.entry("poneyland").or_insert(12);
	///
	/// assert_eq!(map["poneyland"], 12);
	/// ```
	#[inline]
	pub fn or_insert(self, default: S::Value) -> S::ValueMut<'a> {
		match self {
			Occupied(entry) => entry.into_mut(),
			Vacant(entry) => entry.insert(default),
		}
	}

	/// Ensures a value is in the entry by inserting the result of the default function if empty,
	/// and returns a mutable reference to the value in the entry.
	///
	/// # Examples
	///
	/// ```
	/// use generic_btree::slab::Map;
	///
	/// let mut map: Map<&str, String> = Map::new();
	/// let s = "hoho".to_string();
	///
	/// map.entry("poneyland").or_insert_with(|| s);
	///
	/// assert_eq!(map["poneyland"], "hoho".to_string());
	/// ```
	#[inline]
	pub fn or_insert_with<F: FnOnce() -> S::Value>(self, default: F) -> S::ValueMut<'a> {
		match self {
			Occupied(entry) => entry.into_mut(),
			Vacant(entry) => entry.insert(default()),
		}
    }

	/// Ensures a value is in the entry by inserting, if empty, the result of the default function,
	/// which takes the key as its argument, and returns a mutable reference to the value in the
	/// entry.
	///
	/// # Examples
	///
	/// ```
	/// #![feature(or_insert_with_key)]
	/// use generic_btree::slab::Map;
	///
	/// let mut map: Map<&str, usize> = Map::new();
	///
	/// map.entry("poneyland").or_insert_with_key(|key| key.chars().count());
	///
	/// assert_eq!(map["poneyland"], 9);
	/// ```
	#[inline]
	pub fn or_insert_with_key<F: FnOnce(&S::Key) -> S::Value>(self, default: F) -> S::ValueMut<'a> {
		match self {
			Occupied(entry) => entry.into_mut(),
			Vacant(entry) => {
				let value = default(entry.key());
				entry.insert(value)
			}
		}
	}

	/// Provides in-place mutable access to an occupied entry before any
	/// potential inserts into the map.
	///
	/// # Examples
	///
	/// ```
	/// use generic_btree::slab::Map;
	///
	/// let mut map: Map<&str, usize> = Map::new();
	///
	/// map.entry("poneyland")
	///    .and_modify(|e| { *e += 1 })
	///    .or_insert(42);
	/// assert_eq!(map["poneyland"], 42);
	///
	/// map.entry("poneyland")
	///    .and_modify(|e| { *e += 1 })
	///    .or_insert(42);
	/// assert_eq!(map["poneyland"], 43);
	/// ```
	#[inline]
	pub fn and_modify<F>(self, f: F) -> Self where F: FnOnce(S::ValueMut<'_>) {
		match self {
			Occupied(mut entry) => {
				f(entry.get_mut());
				Occupied(entry)
			}
			Vacant(entry) => Vacant(entry),
		}
	}

	/// Ensures a value is in the entry by inserting the default value if empty,
	/// and returns a mutable reference to the value in the entry.
	///
	/// # Examples
	///
	/// ```
	/// use generic_btree::slab::Map;
	///
	/// let mut map: Map<&str, Option<usize>> = Map::new();
	/// map.entry("poneyland").or_default();
	///
	/// assert_eq!(map["poneyland"], None);
	/// ```
	#[inline]
	pub fn or_default(self) -> S::ValueMut<'a> where S::Value: Default {
		match self {
			Occupied(entry) => entry.into_mut(),
			Vacant(entry) => entry.insert(Default::default()),
		}
	}
}

impl<'a, S: Storage> fmt::Debug for Entry<'a, S> where S::Key: fmt::Debug, for<'r> S::KeyRef<'r>: fmt::Debug, for<'r> S::ValueRef<'r>: fmt::Debug {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Occupied(entry) => entry.fmt(f),
			Vacant(entry) => entry.fmt(f)
		}
	}
}

/// A view into a vacant entry in a [`Map`].
/// It is part of the [`Entry`] enum.
pub struct VacantEntry<'a, S: Storage> {
	pub(crate) map: &'a mut S,
	pub(crate) key: S::Key,
	pub(crate) addr: Address
}

impl<'a, S: Storage> VacantEntry<'a, S> {
	/// Gets the address of the vacant entry in the B-Tree.
	#[inline]
	pub fn address(&self) -> Address {
		self.addr
	}

	/// Gets a reference to the keys that would be used when inserting a value through the `VacantEntry`.
	///
	/// ## Example
	/// ```
	/// use generic_btree::slab::Map;
	///
	/// let mut map: Map<&str, usize> = Map::new();
	/// assert_eq!(map.entry("poneyland").key(), &"poneyland");
	/// ```
	#[inline]
	pub fn key(&self) -> &S::Key {
		&self.key
	}

	/// Take ownership of the key.
	///
	/// ## Example
	/// ```
	/// use generic_btree::slab::Map;
	/// use generic_btree::btree::Entry;
	///
	/// let mut map: Map<&str, usize> = Map::new();
	///
	/// if let Entry::Vacant(v) = map.entry("poneyland") {
	///     v.into_key();
	/// }
	/// ```
	#[inline]
	pub fn into_key(self) -> S::Key {
		self.key
	}
}

impl<'a, S: StorageMut> VacantEntry<'a, S> {
	/// Sets the value of the entry with the `VacantEntry`'s key,
	/// and returns a mutable reference to it.
	///
	/// ## Example
	/// ```
	/// use generic_btree::slab::Map;
	/// use generic_btree::btree::Entry;
	///
	/// let mut map: Map<&str, u32> = Map::new();
	///
	/// if let Entry::Vacant(o) = map.entry("poneyland") {
	///     o.insert(37);
	/// }
	/// assert_eq!(map["poneyland"], 37);
	/// ```
	#[inline]
	pub fn insert(self, value: S::Value) -> S::ValueMut<'a> {
		let addr = self.map.insert_at(self.addr, Item::new(self.key, value));
		self.map.item_mut(addr).unwrap().into_value_mut()
	}
}

impl<'a, S: Storage> fmt::Debug for VacantEntry<'a, S> where S::Key: fmt::Debug {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_tuple("VacantEntry").field(self.key()).finish()
	}
}

/// A view into an occupied entry in a [`Map`].
/// It is part of the [`Entry`] enum.
pub struct OccupiedEntry<'a, S> {
	pub(crate) map: &'a mut S,
	pub(crate) addr: Address
}

impl<'a, S: Storage> OccupiedEntry<'a, S> {
	/// Gets the address of the occupied entry in the B-Tree.
	#[inline]
	pub fn address(&self) -> Address {
		self.addr
	}

	/// Gets a reference to the value in the entry.
	///
	/// # Example
	/// ```
	/// use generic_btree::slab::Map;
	/// use generic_btree::btree::Entry;
	///
	/// let mut map: Map<&str, usize> = Map::new();
	/// map.entry("poneyland").or_insert(12);
	///
	/// if let Entry::Occupied(o) = map.entry("poneyland") {
	///     assert_eq!(o.get(), &12);
	/// }
	/// ```
	#[inline]
	pub fn get(&self) -> S::ValueRef<'_> {
		self.map.item(self.addr).unwrap().value()
	}

	/// Gets a reference to the key in the entry.
	///
	/// # Example
	/// ```
	/// use generic_btree::slab::Map;
	///
	/// let mut map: Map<&str, usize> = Map::new();
	/// map.entry("poneyland").or_insert(12);
	/// assert_eq!(map.entry("poneyland").key(), &"poneyland");
	/// ```
	#[inline]
	pub fn key(&self) -> S::KeyRef<'_> {
		self.map.item(self.addr).unwrap().key()
	}
}

impl<'a, S: StorageMut> OccupiedEntry<'a, S> {
	/// Gets a mutable reference to the value in the entry.
	///
	/// If you need a reference to the OccupiedEntry that may outlive
	/// the destruction of the Entry value, see into_mut.
	///
	/// # Example
	/// ```
	/// use generic_btree::slab::Map;
	/// use generic_btree::btree::Entry;
	///
	/// let mut map: Map<&str, usize> = Map::new();
	/// map.entry("poneyland").or_insert(12);
	///
	/// assert_eq!(map["poneyland"], 12);
	/// if let Entry::Occupied(mut o) = map.entry("poneyland") {
	///     *o.get_mut() += 10;
	///     assert_eq!(*o.get(), 22);
	///
	///     // We can use the same Entry multiple times.
	///     *o.get_mut() += 2;
	/// }
	/// assert_eq!(map["poneyland"], 24);
	/// ```
	#[inline]
	pub fn get_mut(&mut self) -> S::ValueMut<'_> {
		self.map.item_mut(self.addr).unwrap().into_value_mut()
	}

	/// Sets the value of the entry with the OccupiedEntry's key,
	/// and returns the entry's old value.
	///
	/// # Example
	/// ```
	/// use generic_btree::slab::Map;
	/// use generic_btree::btree::Entry;
	///
	/// let mut map: Map<&str, usize> = Map::new();
	/// map.entry("poneyland").or_insert(12);
	///
	/// if let Entry::Occupied(mut o) = map.entry("poneyland") {
	///     assert_eq!(o.insert(15), 12);
	/// }
	/// assert_eq!(map["poneyland"], 15);
	/// ```
	#[inline]
	pub fn insert(&mut self, value: S::Value) -> S::Value {
		self.map.item_mut(self.addr).unwrap().set_value(value)
	}

	/// Converts the entry into a mutable reference to its value.
	///
	/// If you need multiple references to the `OccupiedEntry`, see [`get_mut`].
	///
	/// [`get_mut`]: #method.get_mut
	///
	/// # Example
	///
	/// ```
	/// use generic_btree::slab::Map;
	/// use generic_btree::btree::Entry;
	///
	/// let mut map: Map<&str, usize> = Map::new();
	/// map.entry("poneyland").or_insert(12);
	///
	/// assert_eq!(map["poneyland"], 12);
	/// if let Entry::Occupied(o) = map.entry("poneyland") {
	///     *o.into_mut() += 10;
	/// }
	/// assert_eq!(map["poneyland"], 22);
	/// ```
	#[inline]
	pub fn into_mut(self) -> S::ValueMut<'a> {
		self.map.item_mut(self.addr).unwrap().into_value_mut()
	}

	/// Takes the value of the entry out of the map, and returns it.
	///
	/// # Examples
	///
	/// ```
	/// use generic_btree::slab::Map;
	/// use generic_btree::btree::Entry;
	///
	/// let mut map: Map<&str, usize> = Map::new();
	/// map.entry("poneyland").or_insert(12);
	///
	/// if let Entry::Occupied(o) = map.entry("poneyland") {
	///     assert_eq!(o.remove(), 12);
	/// }
	/// // If we try to get "poneyland"'s value, it'll panic:
	/// // println!("{}", map["poneyland"]);
	/// ```
	#[inline]
    pub fn remove(self) -> S::Value {
		self.map.remove_at(self.addr).unwrap().0.into_value()
	}

	/// Take ownership of the key and value from the map.
	///
	/// # Example
	/// ```
	/// use generic_btree::slab::Map;
	/// use generic_btree::btree::Entry;
	///
	/// let mut map: Map<&str, usize> = Map::new();
	/// map.entry("poneyland").or_insert(12);
	///
	/// if let Entry::Occupied(o) = map.entry("poneyland") {
	///     // We delete the entry from the map.
	///     o.remove_entry();
	/// }
	///
	/// // If now try to get the value, it will panic:
	/// // println!("{}", map["poneyland"]);
	/// ```
	#[inline]
	pub fn remove_entry(self) -> Item<S::Key, S::Value> {
		self.map.remove_at(self.addr).unwrap().0
	}
}

impl<'a, S: Storage> fmt::Debug for OccupiedEntry<'a, S> where for<'r> S::KeyRef<'r>: fmt::Debug, for<'r>  S::ValueRef<'r>: fmt::Debug {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_struct("OccupiedEntry").field("key", &self.key()).field("value", &self.get()).finish()
	}
}

/// Iterator that can mutate the tree in place.
pub struct EntriesMut<'a, S> {
	/// The tree reference.
	btree: &'a mut S,

	/// Address of the next item, or last valid address.
	addr: Address,

	len: usize
}

impl<'a, S: Storage> EntriesMut<'a, S> {
	/// Create a new iterator over all the items of the map.
	#[inline]
	pub(crate) fn new(btree: &'a mut S) -> EntriesMut<'a, S> {
		let addr = btree.first_back_address();
		let len = btree.len();
		EntriesMut {
			btree,
			addr,
			len
		}
	}

	/// Get the next visited item without moving the iterator position.
	#[inline]
	pub fn peek(&'a self) -> Option<S::ItemRef<'a>> {
		self.btree.item(self.addr)
	}
}

impl<'a, S: StorageMut> EntriesMut<'a, S> {
	/// Get the next visited item without moving the iterator position.
	#[inline]
	pub fn peek_mut(&'a mut self) -> Option<S::ItemMut<'a>> {
		self.btree.item_mut(self.addr)
	}

	/// Get the next item and move the iterator to the next position.
	#[inline]
	pub fn next_item(&mut self) -> Option<S::ItemMut<'a>> {
		// this is safe because only one mutable reference to the same item can be emitted.
		let btree: &'a mut S = unsafe {
			std::ptr::read(&self.btree)
		};

		let after_addr = btree.next_item_or_back_address(self.addr);
		match btree.item_mut(self.addr) {
			Some(item) => {
				self.len -= 1;
				self.addr = after_addr.unwrap();
				Some(item)
			},
			None => None
		}
	}

	/// Insert a new item in the map before the next item.
	///
	/// ## Correctness
	/// 
	/// It is safe to insert any key-value pair here, however this might break the well-formedness
	/// of the underlying tree, which relies on several invariants.
	/// To preserve these invariants,
	/// the key must be *strictly greater* than the previous visited item's key,
	/// and *strictly less* than the next visited item
	/// (which you can retrive through `IterMut::peek` without moving the iterator).
	/// If this rule is not respected, the data structure will become unusable
	/// (invalidate the specification of every method of the API).
	#[inline]
	pub fn insert(&mut self, key: S::Key, value: S::Value) {
		let addr = self.btree.insert_at(self.addr, Item::new(key, value));
		self.btree.next_item_or_back_address(addr);
		self.len += 1;
	}

	/// Remove the next item and return it.
	#[inline]
	pub fn remove(&mut self) -> Option<Item<S::Key, S::Value>> {
		match self.btree.remove_at(self.addr) {
			Some((item, addr)) => {
				self.len -= 1;
				self.addr = addr;
				Some(item)
			},
			None => None
		}
	}
}

impl<'a, S: StorageMut> Iterator for EntriesMut<'a, S> {
	type Item = S::ItemMut<'a>;

	#[inline]
	fn size_hint(&self) -> (usize, Option<usize>) {
		(self.len, Some(self.len))
	}

	#[inline]
	fn next(&mut self) -> Option<Self::Item> {
		self.next_item()
	}
}