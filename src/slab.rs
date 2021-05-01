use std::marker::PhantomData;
use slab::Slab;
use crate::{
	btree::{
		self,
		node::{
			Buffer,
			Mut as NodeMut
		}
	}
};

mod item;
pub mod node;

pub use item::Item;
pub use node::Node;

pub type Map<K, V> = crate::Map<Storage<K, V, Slab<Node<K, V>>>>;

const M: usize = 8;

/// Slab storage.
pub struct Storage<K, V, S> {
	slab: S,
	root: Option<usize>,
	len: usize,
	key: PhantomData<K>,
	value: PhantomData<V>
}

impl<K, V, S: Default> Default for Storage<K, V, S> {
	fn default() -> Self {
		Self {
			slab: S::default(),
			root: None,
			len: 0,
			key: PhantomData,
			value: PhantomData
		}
	}
}

impl<K, V, S: cc_traits::Slab<Node<K, V>>> btree::Storage for Storage<K, V, S> {
	type Key = K;
	type Value = V;

	type KeyRef<'r> where K: 'r = &'r K;
	type ValueRef<'r> where V: 'r = &'r V;
	type LeafRef<'r> where S: 'r, V: 'r, K: 'r = &'r node::Leaf<Self::Key, Self::Value>;
	type InternalRef<'r> where S: 'r, V: 'r, K: 'r = &'r node::Internal<Self::Key, Self::Value>;
	type ItemRef<'r> where S: 'r, V: 'r, K: 'r = &'r Item<Self::Key, Self::Value>;

	fn root(&self) -> Option<usize> {
		self.root
	}

	fn len(&self) -> usize {
		self.len
	}

	fn node<'r>(&'r self, id: usize) -> Option<btree::node::Ref<'r, Self>> where V: 'r, K: 'r {
		self.slab.get(id).map(|node| node.into())
	}
}

unsafe impl<K, V, S: cc_traits::SlabMut<Node<K, V>>> btree::StorageMut for Storage<K, V, S> {
	type LeafNode = node::Leaf<K, V>;
	type InternalNode = node::Internal<K, V>;
	
	type KeyMut<'r> where K: 'r = &'r mut K;
	type ValueMut<'r> where V: 'r = &'r mut V;
	type LeafMut<'r> where S: 'r, K: 'r, V: 'r = &'r mut node::Leaf<K, V>;
	type InternalMut<'r> where S: 'r, K: 'r, V: 'r = &'r mut node::Internal<K, V>;
	type ItemMut<'r> where S: 'r, K: 'r, V: 'r = &'r mut Item<K, V>;

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