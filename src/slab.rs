use std::{
	borrow::Borrow,
	marker::PhantomData
};
use crate::btree::{
	self,
	node::{
		Buffer,
		Mut as NodeMut
	}
};

pub mod node;
pub use node::Node;

#[cfg(feature="slab")]
mod map {
	use std::cmp::Ordering;
	use super::*;
	use crate::{
		btree::{
			KeyPartialOrd,
			KeyOrd,
			ItemPartialOrd,
			ItemOrd
		},
		map::Binding
	};

	pub type MapStorage<K, V> = Storage<Binding<K, V>, slab::Slab<Node<Binding<K, V>>>>;
	pub type Map<K, V> = crate::Map<MapStorage<K, V>>;

	impl<K, V> crate::map::MapStorage for MapStorage<K, V> {
		type KeyRef<'a> where Self: 'a, K: 'a, V: 'a = &'a K ;
		type ValueRef<'a> where Self: 'a, K: 'a, V: 'a = &'a V;

		fn split_ref<'a>(binding: &'a Binding<K, V>) -> (&'a K, &'a V) where Self: 'a {
			binding.as_pair()
		}
	}

	impl<K, V> crate::map::MapStorageMut for MapStorage<K, V> {
		type Key = K;
		type Value = V;
		type ValueMut<'a> where Self: 'a, K: 'a, V: 'a = &'a mut V;

		fn split(binding: Binding<K, V>) -> (K, V) {
			binding.into_pair()
		}

		fn split_mut<'a>(binding: &'a mut Binding<K, V>) -> (&'a K, &'a mut V) where Self: 'a {
			binding.into()
		}
	}

	impl<K, V> crate::btree::Insert<crate::map::Inserted<K, V>> for MapStorage<K, V> {
		fn allocate_item(&mut self, crate::map::Inserted(key, value): crate::map::Inserted<K, V>) -> Binding<K, V> {
			Binding::new(key, value)
		}
	}

	impl<'a, K, V> crate::btree::node::item::Replace<MapStorage<K, V>, crate::map::Inserted<K, V>> for &'a mut Binding<K, V> {
		type Output = V;

		fn replace(&mut self, crate::map::Inserted(_, value): crate::map::Inserted<K, V>) -> V {
			self.replace_value(value)
		}
	}

	unsafe impl<'a, K, V> crate::btree::node::item::Read<MapStorage<K, V>> for &'a Binding<K, V> {
		unsafe fn read(&self) -> Binding<K, V> {
			std::ptr::read(*self)
		}
	}

	unsafe impl<'a, K, V> crate::btree::node::item::Read<MapStorage<K, V>> for &'a mut Binding<K, V> {
		unsafe fn read(&self) -> Binding<K, V> {
			std::ptr::read(*self)
		}
	}

	unsafe impl<'a, K, V> crate::btree::node::item::Write<MapStorage<K, V>> for &'a mut Binding<K, V> {
		unsafe fn write(&mut self, value: Binding<K, V>) {
			std::ptr::write(*self, value)
		}
	}

	impl<Q: ?Sized, K, V> KeyPartialOrd<Q> for MapStorage<K, V>
	where
		Q: PartialOrd,
		K: Borrow<Q>
	{
		fn key_partial_cmp<'r>(binding: &Self::ItemRef<'r>, other: &Q) -> Option<Ordering> where Self: 'r {
			binding.key.borrow().partial_cmp(other)
		}
	}

	impl<K, V> KeyPartialOrd<crate::map::Inserted<K, V>> for MapStorage<K, V>
	where
		K: PartialOrd
	{
		fn key_partial_cmp<'r>(binding: &Self::ItemRef<'r>, other: &crate::map::Inserted<K, V>) -> Option<Ordering> where Self: 'r {
			binding.key.partial_cmp(&other.0)
		}
	}

	impl<K, V> KeyOrd for MapStorage<K, V>
	where
		K: Ord
	{
		fn key_cmp<'r, 's>(binding: &Self::ItemRef<'r>, other: &Self::ItemRef<'s>) -> Ordering where Self: 'r + 's {
			binding.key.cmp(&other.key)
		}
	}

	impl<K1, K2, V1, V2> ItemPartialOrd<MapStorage<K2, V2>> for MapStorage<K1, V1>
	where
		K1: PartialOrd<K2>,
		V1: PartialOrd<V2>
	{
		fn item_partial_cmp<'r, 's>(binding: &&'r Binding<K1, V1>, other: &&'s Binding<K2, V2>) -> Option<Ordering> where Self: 'r, MapStorage<K2, V2>: 's {
			(**binding).partial_cmp(*other)
		}
	}

	impl<K, V> ItemOrd for MapStorage<K, V>
	where
		K: Ord,
		V: Ord
	{
		fn item_cmp<'r, 's>(binding: &&'r Binding<K, V>, other: &&'s Binding<K, V>) -> Ordering where Self: 'r + 's {
			binding.key.cmp(&other.key)
		}
	}
}

#[cfg(feature="slab")]
pub use map::*;

const M: usize = 8;

/// Slab storage.
pub struct Storage<T, S> {
	slab: S,
	root: Option<usize>,
	len: usize,
	item: PhantomData<T>
}

impl<T, S: Default> Default for Storage<T, S> {
	fn default() -> Self {
		Self {
			slab: S::default(),
			root: None,
			len: 0,
			item: PhantomData
		}
	}
}

impl<T, S: cc_traits::Slab<Node<T>>> btree::Storage for Storage<T, S> {
	type ItemRef<'r> where S: 'r, T: 'r = &'r T;
	type LeafRef<'r> where S: 'r, T: 'r = &'r node::Leaf<T>;
	type InternalRef<'r> where S: 'r, T: 'r = &'r node::Internal<T>;

	fn root(&self) -> Option<usize> {
		self.root
	}

	fn len(&self) -> usize {
		self.len
	}

	fn node(&self, id: usize) -> Option<btree::node::Ref<'_, Self>> {
		self.slab.get(id).map(|node| node.into())
	}
}

unsafe impl<T, S: cc_traits::SlabMut<Node<T>>> btree::StorageMut for Storage<T, S> {
	type Item = T;
	type LeafNode = node::Leaf<T>;
	type InternalNode = node::Internal<T>;
	
	type ItemMut<'r> where S: 'r, T: 'r = &'r mut T;
	type LeafMut<'r> where S: 'r, T: 'r = &'r mut node::Leaf<T>;
	type InternalMut<'r> where S: 'r, T: 'r = &'r mut node::Internal<T>;

	fn set_root(&mut self, root: Option<usize>) {
		self.root = root
	}
	
	fn set_len(&mut self, new_len: usize) {
		self.len = new_len
	}

	fn allocate_node(&mut self, node: Buffer<Self>) -> usize {
		self.slab.insert(node.into())
	}

	fn release_node(&mut self, id: usize) -> Buffer<Self> {
		self.slab.remove(id).unwrap().into()
	}

	fn node_mut(&mut self, id: usize) -> Option<NodeMut<Self>> {
		self.slab.get_mut(id).map(|node| node.into())
	}
}

impl<'a, T, S: cc_traits::SlabMut<Node<T>>> btree::node::item::Mut<Storage<T, S>> for &'a mut T {
	fn swap(&mut self, other: &mut T) {
		std::mem::swap(*self, other)
	}
}