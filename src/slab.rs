use std::marker::PhantomData;
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
use crate::map::Binding;
#[cfg(feature="slab")]
pub type Map<K, V> = crate::Map<Storage<Binding<K, V>, slab::Slab<Node<Binding<K, V>>>>>;

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
			item: PhantomData,
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