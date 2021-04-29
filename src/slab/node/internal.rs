use staticvec::StaticVec;
use crate::{
	btree::{
		self,
		node::Offset
	},
	slab::{
		M,
		Item,
		Node,
		Storage
	}
};

struct Branch<K, V> {
	item: Item<K, V>,
	child_id: usize
}

pub struct Internal<K, V> {
	parent: usize,
	first_child_id: usize,
	branches: StaticVec<Branch<K, V>, M>
}

impl<K, V> Default for Internal<K, V> {
	fn default() -> Self {
		Self {
			parent: usize::MAX,
			first_child_id: usize::MAX,
			branches: StaticVec::new()
		}
	}
}

impl<K, V, S: cc_traits::SlabMut<Node<K, V>>> btree::node::buffer::Internal<Storage<K, V, S>> for Internal<K, V> {
	fn parent(&self) -> Option<usize> {
		if self.parent == usize::MAX {
			None
		} else {
			Some(self.parent)
		}
	}

	fn set_parent(&mut self, parent: Option<usize>) {
		self.parent = parent.unwrap_or(usize::MAX)
	}

	fn item_count(&self) -> usize {
		self.branches.len()
	}

	fn item(&self, offset: Offset) -> Option<&Item<K, V>> {
		self.branches.get(offset.unwrap()).map(|b| &b.item)
	}

	fn child_id(&self, index: usize) -> Option<usize> {
		if index == 0 {
			Some(self.first_child_id)
		} else {
			self.branches.get(index - 1).map(|b| b.child_id)
		}
	}
	
	fn max_capacity(&self) -> usize {
		M
	}

	fn set_first_child(&mut self, id: usize) {
		self.first_child_id = id;
	}

	fn push_right(&mut self, item: Item<K, V>, child: usize) {
		self.branches.push(Branch {
			item,
			child_id: child
		})
	}
}

impl<'a, K, V, S: 'a + cc_traits::Slab<Node<K, V>>> btree::node::ItemAccess<'a, Storage<K, V, S>> for &'a Internal<K, V> {
	/// Returns the current number of items stored in this node.
	fn item_count(&self) -> usize {
		panic!("TODO")
	}

	/// Returns a reference to the item with the given offset in the node.
	fn item(&self, offset: Offset) -> Option<&'a Item<K, V>> {
		panic!("TODO")
	}
}

impl<'a, K, V, S: 'a + cc_traits::Slab<Node<K, V>>> btree::node::InternalRef<'a, Storage<K, V, S>> for &'a Internal<K, V> {
	/// Returns the identifer of the parent node, if any.
	fn parent(&self) -> Option<usize> {
		panic!("TODO")
	}

	/// Returns the id of the child with the given index, if any.
	/// 
	/// Note that in the case of leaf nodes, this always return `None`.
	fn child_id(&self, index: usize) -> Option<usize> {
		panic!("TODO")
	}

	/// Returns the maximum capacity of this node.
	/// 
	/// Must be at least 6 for internal nodes, and 7 for leaf nodes.
	/// 
	/// The node is considered overflowing if it contains `max_capacity` items.
	fn max_capacity(&self) -> usize {
		panic!("TODO")
	}
}

impl<'a, K, V, S: 'a + cc_traits::SlabMut<Node<K, V>>> btree::node::InternalMut<'a, Storage<K, V, S>> for &'a mut Internal<K, V> {
	fn set_parent(&mut self, parent: Option<usize>) {
		panic!("TODO")
	}

	fn set_first_child(&mut self, id: usize) {
		panic!("TODO")
	}

	/// Returns a mutable reference to the item with the given offset in the node.
	fn into_item_mut(self, offset: Offset) -> Option<&'a mut Item<K, V>> {
		panic!("TODO")
	}

	fn insert(&mut self, offset: Offset, item: Item<K, V>, right_child_id: usize) {
		panic!("TODO")
	}

	fn remove(&mut self, offset: Offset) -> (Item<K, V>, usize) {
		panic!("TODO")
	}

	fn replace(&mut self, offset: Offset, item: Item<K, V>) -> Item<K, V> {
		panic!("TODO")
	}

	fn append(&mut self, separator: Item<K, V>, other: Internal<K, V>) -> Offset {
		panic!("TODO")
	}
}