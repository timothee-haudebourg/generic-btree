use std::borrow::Borrow;
use crate::{
	Storage,
	StorageMut,
	slab,
	btree::node::{
		ItemRef,
		ItemMut
	}
};

pub struct Map<K, V, S: Storage<Key=K, Value=V> = slab::Storage<K, V>> {
	btree: S
}

impl<K, V, S: Storage<Key=K, Value=V>> Map<K, V, S> {
	/// Create a new empty map.
	pub fn new() -> Self where S: Default {
		Self {
			btree: S::default()
		}
	}

	/// Returns `true` if the map contains no elements.
	///
	/// # Example
	///
	/// ```
	/// use generic_btree::Map;
	///
	/// let mut a = Map::new();
	/// assert!(a.is_empty());
	/// a.insert(1, "a");
	/// assert!(!a.is_empty());
	/// ```
	#[inline]
	pub fn is_empty(&self) -> bool {
		self.btree.is_empty()
	}

	/// Returns the number of elements in the map.
	///
	/// # Example
	///
	/// ```
	/// use btree_slab::BTreeMap;
	///
	/// let mut a = BTreeMap::new();
	/// assert_eq!(a.len(), 0);
	/// a.insert(1, "a");
	/// assert_eq!(a.len(), 1);
	/// ```
	#[inline]
	pub fn len(&self) -> usize {
		self.btree.len()
	}

	/// Returns the key-value pair corresponding to the supplied key.
	///
	/// The supplied key may be any borrowed form of the map's key type, but the ordering
	/// on the borrowed form *must* match the ordering on the key type.
	///
	/// # Example
	///
	/// ```
	/// use btree_slab::BTreeMap;
	///
	/// let mut map: BTreeMap<i32, &str> = BTreeMap::new();
	/// map.insert(1, "a");
	/// assert_eq!(map.get_key_value(&1), Some((&1, &"a")));
	/// assert_eq!(map.get_key_value(&2), None);
	/// ```
	#[inline]
	pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<S::ValueRef<'_>> where K: Borrow<Q>, Q: Ord {
		self.btree.get(key)
	}

	/// Returns the key-value pair corresponding to the supplied key.
	///
	/// The supplied key may be any borrowed form of the map's key type, but the ordering
	/// on the borrowed form *must* match the ordering on the key type.
	///
	/// # Examples
	///
	/// ```
	/// use btree_slab::BTreeMap;
	///
	/// let mut map = BTreeMap::new();
	/// map.insert(1, "a");
	/// assert_eq!(map.get_key_value(&1), Some((&1, &"a")));
	/// assert_eq!(map.get_key_value(&2), None);
	/// ```
	#[inline]
	pub fn get_key_value<Q: ?Sized>(&self, k: &Q) -> Option<(S::KeyRef<'_>, S::ValueRef<'_>)>
	where
		K: Borrow<Q>,
		Q: Ord,
	{
		self.btree.get_item(k).map(|item| item.split())
	}

	/// Returns the first key-value pair in the map.
	/// The key in this pair is the minimum key in the map.
	///
	/// # Example
	///
	/// ```
	/// use btree_slab::BTreeMap;
	///
	/// let mut map = BTreeMap::new();
	/// assert_eq!(map.first_key_value(), None);
	/// map.insert(1, "b");
	/// map.insert(2, "a");
	/// assert_eq!(map.first_key_value(), Some((&1, &"b")));
	/// ```
	#[inline]
	pub fn first_key_value(&self) -> Option<(S::KeyRef<'_>, S::ValueRef<'_>)> {
		self.btree.first_item().map(|item| item.split())
	}

	/// Returns the last key-value pair in the map.
	/// The key in this pair is the maximum key in the map.
	///
	/// # Examples
	///
	/// Basic usage:
	///
	/// ```
	/// use btree_slab::BTreeMap;
	///
	/// let mut map = BTreeMap::new();
	/// map.insert(1, "b");
	/// map.insert(2, "a");
	/// assert_eq!(map.last_key_value(), Some((&2, &"a")));
	/// ```
	#[inline]
	pub fn last_key_value(&self) -> Option<(S::KeyRef<'_>, S::ValueRef<'_>)> {
		self.btree.last_item().map(|item| item.split())
	}
}

// impl<'a, S: Storage> IntoIterator for &'a S {
// 	type IntoIter = Iter<'a, S>;
// 	type Item = S::ItemRef<'a>;

// 	#[inline]
// 	fn into_iter(self) -> Iter<'a, S> {
// 		self.iter()
// 	}
// }