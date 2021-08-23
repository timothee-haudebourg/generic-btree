use std::{
	borrow::Borrow,
	iter::{
		FusedIterator,
		FromIterator
	},
	ops::{
		RangeBounds,
		Index
	},
	cmp::{
		Ord,
		PartialOrd,
		Ordering
	},
	hash::{
		Hash,
		Hasher
	}
};
use crate::{
	Storage,
	StorageMut,
	btree::{
		Entry,
		OccupiedEntry,
		EntriesMut,
		Keys,
		IntoKeys,
		Values,
		IntoValues,
		ValuesMut,
		node::{
			ItemMut
		}
	}
};

pub struct Map<S: Storage> {
	btree: S
}

impl<S: Storage> Map<S> {
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
	/// use generic_btree::slab::Map;
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
	/// use generic_btree::slab::Map;
	///
	/// let mut a = Map::new();
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
	/// use generic_btree::slab::Map;
	///
	/// let mut map: Map<i32, &str> = Map::new();
	/// map.insert(1, "a");
	/// assert_eq!(map.get_key_value(&1), Some((&1, &"a")));
	/// assert_eq!(map.get_key_value(&2), None);
	/// ```
	#[inline]
	pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<S::ValueRef<'_>> where S::Key: Borrow<Q>, Q: Ord {
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
	/// use generic_btree::slab::Map;
	///
	/// let mut map = Map::new();
	/// map.insert(1, "a");
	/// assert_eq!(map.get_key_value(&1), Some((&1, &"a")));
	/// assert_eq!(map.get_key_value(&2), None);
	/// ```
	#[inline]
	pub fn get_key_value<Q: ?Sized>(&self, k: &Q) -> Option<(S::KeyRef<'_>, S::ValueRef<'_>)>
	where
		S::Key: Borrow<Q>,
		Q: Ord,
	{
		self.btree.get_item(k).map(|item| item.as_pair())
	}

	/// Returns the first key-value pair in the map.
	/// The key in this pair is the minimum key in the map.
	///
	/// # Example
	///
	/// ```
	/// use generic_btree::slab::Map;
	///
	/// let mut map = Map::new();
	/// assert_eq!(map.first_key_value(), None);
	/// map.insert(1, "b");
	/// map.insert(2, "a");
	/// assert_eq!(map.first_key_value(), Some((&1, &"b")));
	/// ```
	#[inline]
	pub fn first_key_value(&self) -> Option<(S::KeyRef<'_>, S::ValueRef<'_>)> {
		self.btree.first_item().map(|item| item.as_pair())
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
	pub fn last_key_value(&self) -> Option<(S::KeyRef<'_>, S::ValueRef<'_>)> {
		self.btree.last_item().map(|item| item.as_pair())
	}

	/// Gets an iterator over the entries of the map, sorted by key.
	///
	/// # Example
	///
	/// ```
	/// use generic_btree::slab::Map;
	///
	/// let mut map = Map::new();
	/// map.insert(3, "c");
	/// map.insert(2, "b");
	/// map.insert(1, "a");
	///
	/// for (key, value) in map.iter() {
	///     println!("{}: {}", key, value);
	/// }
	///
	/// let (first_key, first_value) = map.iter().next().unwrap();
	/// assert_eq!((*first_key, *first_value), (1, "a"));
	/// ```
	#[inline]
	pub fn iter(&self) -> Iter<S> {
		Iter::new(&self.btree)
	}

	/// Constructs a double-ended iterator over a sub-range of elements in the map.
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
	///
	/// # Example
	///
	/// ```
	/// use generic_btree::slab::Map;
	/// use std::ops::Bound::Included;
	///
	/// let mut map = Map::new();
	/// map.insert(3, "a");
	/// map.insert(5, "b");
	/// map.insert(8, "c");
	/// for (&key, &value) in map.range((Included(&4), Included(&8))) {
	///     println!("{}: {}", key, value);
	/// }
	/// assert_eq!(Some((&5, &"b")), map.range(4..).next());
	/// ```
	#[inline]
	pub fn range<T: ?Sized, R>(&self, range: R) -> Range<S>
	where
		T: Ord,
		S::Key: Borrow<T>,
		R: RangeBounds<T>,
	{
		Range::new(&self.btree, range)
	}

	/// Gets an iterator over the keys of the map, in sorted order.
	///
	/// # Example
	///
	/// ```
	/// use generic_btree::slab::Map;
	///
	/// let mut a = Map::new();
	/// a.insert(2, "b");
	/// a.insert(1, "a");
	///
	/// let keys: Vec<_> = a.keys().cloned().collect();
	/// assert_eq!(keys, [1, 2]);
	/// ```
	#[inline]
	pub fn keys(&self) -> Keys<S> {
		self.btree.keys()
	}

	/// Gets an iterator over the values of the map, in order by key.
	///
	/// # Example
	///
	/// ```
	/// use generic_btree::slab::Map;
	///
	/// let mut a = Map::new();
	/// a.insert(1, "hello");
	/// a.insert(2, "goodbye");
	///
	/// let values: Vec<&str> = a.values().cloned().collect();
	/// assert_eq!(values, ["hello", "goodbye"]);
	/// ```
	#[inline]
	pub fn values(&self) -> Values<S> {
		self.btree.values()
	}

	/// Returns `true` if the map contains a value for the specified key.
	///
	/// The key may be any borrowed form of the map's key type, but the ordering
	/// on the borrowed form *must* match the ordering on the key type.
	///
	/// # Example
	/// ```
	/// use generic_btree::slab::Map;
	///
	/// let mut map: Map<i32, &str> = Map::new();
	/// map.insert(1, "a");
	/// assert_eq!(map.contains_key(&1), true);
	/// assert_eq!(map.contains_key(&2), false);
	/// ```
	#[inline]
	pub fn contains_key<Q: ?Sized>(&self, key: &Q) -> bool where S::Key: Borrow<Q>, Q: Ord {
		self.btree.contains_key(key)
	}

	/// Write the tree in the DOT graph descrption language.
	///
	/// Requires the `dot` feature.
	#[cfg(feature = "dot")]
	#[inline]
	pub fn dot_write<W: std::io::Write>(&self, f: &mut W) -> std::io::Result<()> where S::Key: std::fmt::Display, S::Value: std::fmt::Display {
		self.btree.dot_write(f)
	}

	pub fn btree(&self) -> &S {
		&self.btree
	}
}

impl<S: StorageMut> Map<S> {
	// TODO clear

	/// Returns a mutable reference to the value corresponding to the key.
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
	/// if let Some(x) = map.get_mut(&1) {
	///     *x = "b";
	/// }
	/// assert_eq!(map[&1], "b");
	/// ```
	#[inline]
	pub fn get_mut(&mut self, key: &S::Key) -> Option<S::ValueMut<'_>> where S::Key: Ord {
		self.btree.get_mut(key)
	}

	/// Gets the given key's corresponding entry in the map for in-place manipulation.
	///
	/// # Example
	///
	/// ```
	/// use generic_btree::slab::Map;
	///
	/// let mut letters = Map::new();
	///
	/// for ch in "a short treatise on fungi".chars() {
	///     let counter = letters.entry(ch).or_insert(0);
	///     *counter += 1;
	/// }
	///
	/// assert_eq!(letters[&'s'], 2);
	/// assert_eq!(letters[&'t'], 3);
	/// assert_eq!(letters[&'u'], 1);
	/// assert_eq!(letters.get(&'y'), None);
	/// ```
	#[inline]
	pub fn entry(&mut self, key: S::Key) -> Entry<S> where S::Key: Ord {
		self.btree.entry(key)
	}

	/// Returns the first entry in the map for in-place manipulation.
	/// The key of this entry is the minimum key in the map.
	///
	/// # Example
	///
	/// ```
	/// use generic_btree::slab::Map;
	///
	/// let mut map = Map::new();
	/// map.insert(1, "a");
	/// map.insert(2, "b");
	/// if let Some(mut entry) = map.first_entry() {
	///     if *entry.key() > 0 {
	///         entry.insert("first");
	///     }
	/// }
	/// assert_eq!(*map.get(&1).unwrap(), "first");
	/// assert_eq!(*map.get(&2).unwrap(), "b");
	/// ```
	#[inline]
	pub fn first_entry(&mut self) -> Option<OccupiedEntry<S>> {
		self.btree.first_entry()
	}

	/// Returns the last entry in the map for in-place manipulation.
	/// The key of this entry is the maximum key in the map.
	///
	/// # Example
	///
	/// ```
	/// use generic_btree::slab::Map;
	///
	/// let mut map = Map::new();
	/// map.insert(1, "a");
	/// map.insert(2, "b");
	/// if let Some(mut entry) = map.last_entry() {
	///     if *entry.key() > 0 {
	///         entry.insert("last");
	///     }
	/// }
	/// assert_eq!(*map.get(&1).unwrap(), "a");
	/// assert_eq!(*map.get(&2).unwrap(), "last");
	/// ```
	#[inline]
	pub fn last_entry(&mut self) -> Option<OccupiedEntry<S>> {
		self.btree.last_entry()
	}

	/// Insert a key-value pair in the tree.
	#[inline]
	pub fn insert(&mut self, key: S::Key, value: S::Value) -> Option<S::Value> where S::Key: Ord {
		self.btree.insert(key, value)
	}

	/// Replace a key-value pair in the tree.
	#[inline]
	pub fn replace(&mut self, key: S::Key, value: S::Value) -> Option<(S::Key, S::Value)> where S::Key: Ord {
		self.btree.replace(key, value).map(|item| item.into_inner())
	}

	/// Removes and returns the first element in the map.
	/// The key of this element is the minimum key that was in the map.
	///
	/// # Example
	///
	/// Draining elements in ascending order, while keeping a usable map each iteration.
	///
	/// ```
	/// use generic_btree::slab::Map;
	///
	/// let mut map = Map::new();
	/// map.insert(1, "a");
	/// map.insert(2, "b");
	/// while let Some((key, _val)) = map.pop_first() {
	///     assert!(map.iter().all(|(k, _v)| *k > key));
	/// }
	/// assert!(map.is_empty());
	/// ```
	#[inline]
	pub fn pop_first(&mut self) -> Option<(S::Key, S::Value)> {
		self.btree.pop_first().map(|item| item.into_inner())
	}

	/// Removes and returns the last element in the map.
	/// The key of this element is the maximum key that was in the map.
	///
	/// # Example
	///
	/// Draining elements in descending order, while keeping a usable map each iteration.
	///
	/// ```
	/// use generic_btree::slab::Map;
	///
	/// let mut map = Map::new();
	/// map.insert(1, "a");
	/// map.insert(2, "b");
	/// while let Some((key, _val)) = map.pop_last() {
	///     assert!(map.iter().all(|(k, _v)| *k < key));
	/// }
	/// assert!(map.is_empty());
	/// ```
	#[inline]
	pub fn pop_last(&mut self) -> Option<(S::Key, S::Value)> {
		self.btree.pop_last().map(|item| item.into_inner())
	}

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
	pub fn remove<Q: ?Sized>(&mut self, key: &Q) -> Option<S::Value> where S::Key: Borrow<Q>, Q: Ord {
		self.btree.remove(key)
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
	pub fn remove_entry<Q: ?Sized>(&mut self, key: &Q) -> Option<(S::Key, S::Value)> where S::Key: Borrow<Q>, Q: Ord {
		self.btree.remove_entry(key).map(|item| item.into_inner())
	}

	/// Removes and returns the binding in the map, if any, of which key matches the given one.
	#[inline]
	pub fn take<Q: ?Sized>(&mut self, key: &Q) -> Option<(S::Key, S::Value)> where S::Key: Borrow<Q>, Q: Ord {
		self.btree.take(key).map(|item| item.into_inner())
	}

	/// General-purpose update function.
	///
	/// This can be used to insert, compare, replace or remove the value associated to the given
	/// `key` in the tree.
	/// The action to perform is specified by the `action` function.
	/// This function is called once with:
	///  - `Some(value)` when `value` is aready associated to `key` or
	///  - `None` when the `key` is not associated to any value.
	///
	/// The `action` function must return a pair (`new_value`, `result`) where
	/// `new_value` is the new value to be associated to `key`
	/// (if it is `None` any previous binding is removed) and
	/// `result` is the value returned by the entire `update` function call.
	#[inline]
	pub fn update<T, F>(&mut self, key: S::Key, action: F) -> T where S::Key: Ord, F: FnOnce(Option<S::Value>) -> (Option<S::Value>, T) {
		self.btree.update(key, action)
	}

	/// Gets a mutable iterator over the entries of the map, sorted by key.
	///
	/// # Example
	///
	/// ```
	/// use generic_btree::slab::Map;
	///
	/// let mut map = Map::new();
	/// map.insert("a", 1);
	/// map.insert("b", 2);
	/// map.insert("c", 3);
	///
	/// // add 10 to the value if the key isn't "a"
	/// for (key, value) in map.iter_mut() {
	///     if key != &"a" {
	///         *value += 10;
	///     }
	/// }
	/// ```
	#[inline]
	pub fn iter_mut(&mut self) -> IterMut<S> {
		IterMut::new(&mut self.btree)
	}

	#[inline]
	pub fn into_iter(self) -> IntoIter<S> {
		IntoIter::new(self)
	}

	/// Creates a consuming iterator visiting all the keys, in sorted order.
	/// The map cannot be used after calling this.
	/// The iterator element type is `K`.
	///
	/// # Example
	///
	/// ```
	/// use generic_btree::slab::Map;
	///
	/// let mut a = Map::new();
	/// a.insert(2, "b");
	/// a.insert(1, "a");
	///
	/// let keys: Vec<i32> = a.into_keys().collect();
	/// assert_eq!(keys, [1, 2]);
	/// ```
	#[inline]
	pub fn into_keys(self) -> IntoKeys<S> {
		self.btree.into_keys()
	}

	/// Creates a consuming iterator visiting all the values, in order by key.
	/// The map cannot be used after calling this.
	/// The iterator element type is `V`.
	///
	/// # Example
	///
	/// ```
	/// use generic_btree::slab::Map;
	///
	/// let mut a = Map::new();
	/// a.insert(1, "hello");
	/// a.insert(2, "goodbye");
	///
	/// let values: Vec<&str> = a.into_values().collect();
	/// assert_eq!(values, ["hello", "goodbye"]);
	/// ```
	#[inline]
	pub fn into_values(self) -> IntoValues<S> {
		self.btree.into_values()
	}

	/// Gets a mutable iterator over the entries of the map, sorted by key, that allows insertion and deletion of the iterated entries.
	/// 
	/// # Correctness
	/// 
	/// It is safe to insert any key-value pair while iterating,
	/// however this might break the well-formedness
	/// of the underlying tree, which relies on several invariants.
	/// To preserve these invariants,
	/// the inserted key must be *strictly greater* than the previous visited item's key,
	/// and *strictly less* than the next visited item
	/// (which you can retrive through [`EntriesMut::peek`] without moving the iterator).
	/// If this rule is not respected, the data structure will become unusable
	/// (invalidate the specification of every method of the API).
	///
	/// # Example
	///
	/// ```
	/// use generic_btree::slab::Map;
	///
	/// let mut map = Map::new();
	/// map.insert("a", 1);
	/// map.insert("b", 2);
	/// map.insert("d", 4);
	///
	/// let mut entries = map.entries_mut();
	/// entries.next();
	/// entries.next();
	/// entries.insert("c", 3);
	/// 
	/// let entries: Vec<_> = map.into_iter().collect();
	/// assert_eq!(entries, vec![("a", 1), ("b", 2), ("c", 3), ("d", 4)]);
	/// ```
	#[inline]
	pub fn entries_mut(&mut self) -> EntriesMut<S> {
		EntriesMut::new(&mut self.btree)
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
	///
	/// # Example
	///
	/// ```
	/// use generic_btree::slab::Map;
	///
	/// let mut map: Map<&str, i32> = ["Alice", "Bob", "Carol", "Cheryl"]
	///     .iter()
	///     .map(|&s| (s, 0))
	///     .collect();
	/// for (_, balance) in map.range_mut("B".."Cheryl") {
	///     *balance += 100;
	/// }
	/// for (name, balance) in &map {
	///     println!("{} => {}", name, balance);
	/// }
	/// ```
	#[inline]
	pub fn range_mut<T: ?Sized, R>(&mut self, range: R) -> RangeMut<S>
	where
		T: Ord,
		S::Key: Borrow<T>,
		R: RangeBounds<T>,
	{
		RangeMut::new(&mut self.btree, range)
	}

	/// Gets a mutable iterator over the values of the map, in order by key.
	///
	/// # Example
	///
	/// ```
	/// use generic_btree::slab::Map;
	///
	/// let mut a = Map::new();
	/// a.insert(1, String::from("hello"));
	/// a.insert(2, String::from("goodbye"));
	///
	/// for value in a.values_mut() {
	///     value.push_str("!");
	/// }
	///
	/// let values: Vec<String> = a.values().cloned().collect();
	/// assert_eq!(values, [String::from("hello!"),
	///                     String::from("goodbye!")]);
	/// ```
	#[inline]
	pub fn values_mut(&mut self) -> ValuesMut<S> {
		self.btree.values_mut()
	}

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
	///
	/// # Example
	///
	/// Splitting a map into even and odd keys, reusing the original map:
	///
	/// ```
	/// use generic_btree::slab::Map;
	///
	/// let mut map: Map<i32, i32> = (0..8).map(|x| (x, x)).collect();
	/// let evens: Map<_, _> = map.drain_filter(|k, _v| k % 2 == 0).collect();
	/// let odds = map;
	/// assert_eq!(evens.keys().copied().collect::<Vec<_>>(), vec![0, 2, 4, 6]);
	/// assert_eq!(odds.keys().copied().collect::<Vec<_>>(), vec![1, 3, 5, 7]);
	/// ```
	#[inline]
	pub fn drain_filter<F>(&mut self, pred: F) -> DrainFilter<S, F> where F: FnMut(&S::Key, &mut S::Value) -> bool {
		DrainFilter::new(&mut self.btree, pred)
	}

	/// Retains only the elements specified by the predicate.
	///
	/// In other words, remove all pairs `(k, v)` such that `f(&k, &mut v)` returns `false`.
	///
	/// # Example
	///
	/// ```
	/// use generic_btree::slab::Map;
	///
	/// let mut map: Map<i32, i32> = (0..8).map(|x| (x, x*10)).collect();
	/// // Keep only the elements with even-numbered keys.
	/// map.retain(|&k, _| k % 2 == 0);
	/// assert!(map.into_iter().eq(vec![(0, 0), (2, 20), (4, 40), (6, 60)]));
	/// ```
	#[inline]
	pub fn retain<F>(&mut self, mut f: F)
	where
		F: FnMut(&S::Key, &mut S::Value) -> bool,
	{
		self.drain_filter(|k, v| !f(k, v));
	}

	pub fn btree_mut(&mut self) -> &mut S {
		&mut self.btree
	}
}

impl<Q: ?Sized, S: Storage> Index<&Q> for Map<S>
where
	S::Key: Ord + Borrow<Q>,
	Q: Ord,
	for<'r> S::ValueRef<'r>: Into<&'r S::Value>
{
	type Output = S::Value;

	/// Returns a reference to the value corresponding to the supplied key.
	///
	/// # Panics
	///
	/// Panics if the key is not present in the `Map`.
	#[inline]
	fn index(&self, key: &Q) -> &S::Value {
		self.get(key).expect("no entry found for key").into()
	}
}

impl<S: Storage, T: Storage> PartialEq<Map<T>> for Map<S> where T::Key: PartialEq<S::Key>, T::Value: PartialEq<S::Value> {
	fn eq(&self, other: &Map<T>) -> bool {
		self.btree.eq(&other.btree)
	}
}

impl<S: Storage + Default> Default for Map<S> {
	#[inline]
	fn default() -> Self {
		Self::new()
	}
}

impl<S: StorageMut + Default> FromIterator<(S::Key, S::Value)> for Map<S> where S::Key: Ord {
	#[inline]
	fn from_iter<T>(iter: T) -> Self where T: IntoIterator<Item = (S::Key, S::Value)> {
		let mut map = Self::new();

		for (key, value) in iter {
			map.insert(key, value);
		}

		map
	}
}

impl<S: StorageMut> Extend<(S::Key, S::Value)> for Map<S> where S::Key: Ord {
	#[inline]
	fn extend<T>(&mut self, iter: T) where T: IntoIterator<Item = (S::Key, S::Value)> {
		for (key, value) in iter {
			self.insert(key, value);
		}
	}
}

// impl<'a, S: StorageMut> Extend<(&'a S::Key, &'a S::Value)> for Map<S> where S::Key: Ord + Copy, S::Value: Copy {
// 	#[inline]
// 	fn extend<T>(&mut self, iter: T) where T: IntoIterator<Item = (&'a S::Key, &'a S::Value)> {
// 		self.extend(iter.into_iter().map(|(&key, &value)| (key, value)));
// 	}
// }

impl<S: Storage> Eq for Map<S> where S::Key: Eq, S::Value: Eq {}

impl<S: Storage, T: Storage> PartialOrd<Map<T>> for Map<S> where T::Key: PartialOrd<S::Key>, T::Value: PartialOrd<S::Value> {
	fn partial_cmp(&self, other: &Map<T>) -> Option<Ordering> {
		self.btree.partial_cmp(&other.btree)
	}
}

impl<S: Storage> Ord for Map<S> where S::Key: Ord, S::Value: Ord {
	fn cmp(&self, other: &Self) -> Ordering {
		self.btree.cmp(&other.btree)
	}
}

impl<S: Storage> Hash for Map<S> where S::Key: Hash, S::Value: Hash {
	#[inline]
	fn hash<H: Hasher>(&self, h: &mut H) {
		self.btree.hash(h)
	}
}

pub struct Iter<'a, S: Storage> {
	inner: crate::btree::Iter<'a, S>
}

impl<'a, S: Storage> Iter<'a, S> {
	#[inline]
	fn new(btree: &'a S) -> Self {
		Self {
			inner: btree.iter()
		}
	}
}

impl<'a, S: 'a + Storage> Iterator for Iter<'a, S> {
	type Item = (S::KeyRef<'a>, S::ValueRef<'a>);

	#[inline]
	fn size_hint(&self) -> (usize, Option<usize>) {
		self.inner.size_hint()
	}

	#[inline]
	fn next(&mut self) -> Option<Self::Item> {
		self.inner.next().map(|item| item.as_pair())
	}
}

impl<'a, S: 'a + Storage> FusedIterator for Iter<'a, S> {}
impl<'a, S: 'a + Storage> ExactSizeIterator for Iter<'a, S> {}

impl<'a, S: 'a + Storage> DoubleEndedIterator for Iter<'a, S> {
	#[inline]
	fn next_back(&mut self) -> Option<Self::Item> {
		self.inner.next().map(|item| item.as_pair())
	}
}

impl<'a, S: Storage> IntoIterator for &'a Map<S> {
	type IntoIter = Iter<'a, S>;
	type Item = (S::KeyRef<'a>, S::ValueRef<'a>);

	#[inline]
	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

pub struct IterMut<'a, S: StorageMut> {
	inner: crate::btree::IterMut<'a, S>
}

impl<'a, S: StorageMut> IterMut<'a, S> {
	#[inline]
	fn new(btree: &'a mut S) -> Self {
		Self {
			inner: btree.iter_mut()
		}
	}
}

impl<'a, S: 'a + StorageMut> Iterator for IterMut<'a, S> {
	type Item = (S::KeyMut<'a>, S::ValueMut<'a>);

	#[inline]
	fn size_hint(&self) -> (usize, Option<usize>) {
		self.inner.size_hint()
	}

	#[inline]
	fn next(&mut self) -> Option<Self::Item> {
		self.inner.next().map(|item| item.into_pair_mut())
	}
}

impl<'a, S: 'a + StorageMut> FusedIterator for IterMut<'a, S> {}
impl<'a, S: 'a + StorageMut> ExactSizeIterator for IterMut<'a, S> {}

impl<'a, S: 'a + StorageMut> DoubleEndedIterator for IterMut<'a, S> {
	#[inline]
	fn next_back(&mut self) -> Option<Self::Item> {
		self.inner.next().map(|item| item.into_pair_mut())
	}
}

impl<'a, S: 'a + StorageMut> IntoIterator for &'a mut Map<S> {
	type IntoIter = IterMut<'a, S>;
	type Item = (S::KeyMut<'a>, S::ValueMut<'a>);

	#[inline]
	fn into_iter(self) -> Self::IntoIter {
		self.iter_mut()
	}
}

pub struct IntoIter<S> {
	inner: crate::btree::IntoIter<S>
}

impl<S: StorageMut> IntoIter<S> {
	pub(crate) fn new(map: Map<S>) -> Self {
		Self {
			inner: map.btree.into_iter()
		}
	}
}

impl<S: StorageMut> FusedIterator for IntoIter<S> { }
impl<S: StorageMut> ExactSizeIterator for IntoIter<S> { }

impl<S: StorageMut> Iterator for IntoIter<S> {
	type Item = (S::Key, S::Value);

	#[inline]
	fn size_hint(&self) -> (usize, Option<usize>) {
		self.inner.size_hint()
	}

	#[inline]
	fn next(&mut self) -> Option<Self::Item> {
		self.inner.next().map(|item| item.into_inner())
	}
}

impl<S: StorageMut> DoubleEndedIterator for IntoIter<S> {
	#[inline]
	fn next_back(&mut self) -> Option<Self::Item> {
		self.inner.next_back().map(|item| item.into_inner())
	}
}

impl<S: StorageMut> IntoIterator for Map<S> {
	type IntoIter = IntoIter<S>;
	type Item = (S::Key, S::Value);

	#[inline]
	fn into_iter(self) -> Self::IntoIter {
		self.into_iter()
	}
}

pub struct DrainFilter<'a, S: StorageMut, F> where F: FnMut(&S::Key, &mut S::Value) -> bool {
	inner: crate::btree::DrainFilterInner<'a, S>,
	f: F
}

impl<'a, S: StorageMut, F> DrainFilter<'a, S, F> where F: FnMut(&S::Key, &mut S::Value) -> bool {
	#[inline]
	fn new(btree: &'a mut S, f: F) -> Self {
		Self {
			inner: crate::btree::DrainFilterInner::new(btree),
			f
		}
	}
}

impl<'a, S: StorageMut, F> Iterator for DrainFilter<'a, S, F> where F: FnMut(&S::Key, &mut S::Value) -> bool {
	type Item = (S::Key, S::Value);

	#[inline]
	fn size_hint(&self) -> (usize, Option<usize>) {
		self.inner.size_hint()
	}

	#[inline]
	fn next(&mut self) -> Option<Self::Item> {
		let f = &mut self.f;
		self.inner.next_consume(|item : S::ItemMut<'_>| filter::<S, F>(f, item)).map(|item| item.into_inner())
	}
}

fn filter<S: StorageMut, F>(f: &mut F, item: S::ItemMut<'_>) -> bool where F: FnMut(&S::Key, &mut S::Value) -> bool {
	use std::ops::{
		Deref,
		DerefMut
	};
	let (key, mut value) = item.into_pair_mut();
	f(key.deref(), value.deref_mut())
}

impl<'a, S: StorageMut, F> FusedIterator for DrainFilter<'a, S, F> where F: FnMut(&S::Key, &mut S::Value) -> bool {}

impl<'a, S: StorageMut, F> Drop for DrainFilter<'a, S, F> where F: FnMut(&S::Key, &mut S::Value) -> bool {
	#[inline]
	fn drop(&mut self) {
		loop {
			if self.next().is_none() {
				break
			}
		}
	}
}

pub struct Range<'a, S: Storage> {
	inner: crate::btree::Range<'a, S>
}

impl<'a, S: Storage> Range<'a, S> {
	#[inline]
	fn new<T, R>(btree: &'a S, range: R) -> Self where T: Ord + ?Sized, R: RangeBounds<T>, S::Key: Borrow<T> {
		Self {
			inner: btree.range(range)
		}
	}
}

impl<'a, S: 'a + Storage> Iterator for Range<'a, S> {
	type Item = (S::KeyRef<'a>, S::ValueRef<'a>);

	#[inline]
	fn size_hint(&self) -> (usize, Option<usize>) {
		self.inner.size_hint()
	}

	#[inline]
	fn next(&mut self) -> Option<Self::Item> {
		self.inner.next().map(|item| item.as_pair())
	}
}

impl<'a, S: 'a + Storage> FusedIterator for Range<'a, S> {}
impl<'a, S: 'a + Storage> ExactSizeIterator for Range<'a, S> {}

impl<'a, S: 'a + Storage> DoubleEndedIterator for Range<'a, S> {
	#[inline]
	fn next_back(&mut self) -> Option<Self::Item> {
		self.inner.next().map(|item| item.as_pair())
	}
}

pub struct RangeMut<'a, S: StorageMut> {
	inner: crate::btree::RangeMut<'a, S>
}

impl<'a, S: StorageMut> RangeMut<'a, S> {
	#[inline]
	fn new<T, R>(btree: &'a mut S, range: R) -> Self where T: Ord + ?Sized, R: RangeBounds<T>, S::Key: Borrow<T> {
		Self {
			inner: btree.range_mut(range)
		}
	}
}

impl<'a, S: 'a + StorageMut> Iterator for RangeMut<'a, S> {
	type Item = (S::KeyMut<'a>, S::ValueMut<'a>);

	#[inline]
	fn size_hint(&self) -> (usize, Option<usize>) {
		self.inner.size_hint()
	}

	#[inline]
	fn next(&mut self) -> Option<Self::Item> {
		self.inner.next().map(|item| item.into_pair_mut())
	}
}

impl<'a, S: 'a + StorageMut> FusedIterator for RangeMut<'a, S> {}
impl<'a, S: 'a + StorageMut> ExactSizeIterator for RangeMut<'a, S> {}

impl<'a, S: 'a + StorageMut> DoubleEndedIterator for RangeMut<'a, S> {
	#[inline]
	fn next_back(&mut self) -> Option<Self::Item> {
		self.inner.next().map(|item| item.into_pair_mut())
	}
}