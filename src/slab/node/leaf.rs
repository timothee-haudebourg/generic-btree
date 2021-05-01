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

pub struct Leaf<K, V> {
	parent: usize,
	items: StaticVec<Item<K, V>, {M+1}>
}

impl<K, V> Default for Leaf<K, V> {
	fn default() -> Self {
		Self {
			parent: usize::MAX,
			items: StaticVec::new()
		}
	}
}

impl<K, V, S: cc_traits::SlabMut<Node<K, V>>> btree::node::buffer::Leaf<Storage<K, V, S>> for Leaf<K, V> {
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
		self.items.len()
	}

	fn item(&self, offset: Offset) -> Option<&Item<K, V>> {
		self.items.get(offset.unwrap())
	}

	fn max_capacity(&self) -> usize {
		M+1
	}

	fn push_right(&mut self, item: Item<K, V>) {
		self.items.push(item)
	}

	fn forget(self) {
		std::mem::forget(self.items)
	}
}

impl<'a, K, V, S: 'a + cc_traits::Slab<Node<K, V>>> btree::node::ItemAccess<'a, Storage<K, V, S>> for &'a Leaf<K, V> {
	/// Returns the current number of items stored in this node.
	fn item_count(&self) -> usize {
		self.items.len()
	}

	/// Returns a reference to the item with the given offset in the node.
	fn borrow_item(&self, offset: Offset) -> Option<&Item<K, V>> {
		self.items.get(offset.unwrap())
	}
}

impl<'a, K, V, S: 'a + cc_traits::Slab<Node<K, V>>> btree::node::LeafRef<'a, Storage<K, V, S>> for &'a Leaf<K, V> {
	fn parent(&self) -> Option<usize> {
		if self.parent == usize::MAX {
			None
		} else {
			Some(self.parent)
		}
	}

	fn max_capacity(&self) -> usize {
		M+1
	}
}

impl<'a, K, V, S: 'a + cc_traits::Slab<Node<K, V>>> btree::node::LeafConst<'a, Storage<K, V, S>> for &'a Leaf<K, V> {
	fn item(&self, offset: Offset) -> Option<&'a Item<K, V>> {
		self.items.get(offset.unwrap())
	}
}

impl<'a, K, V, S: 'a + cc_traits::Slab<Node<K, V>>> btree::node::ItemAccess<'a, Storage<K, V, S>> for &'a mut Leaf<K, V> {
	/// Returns the current number of items stored in this node.
	fn item_count(&self) -> usize {
		self.items.len()
	}

	/// Returns a reference to the item with the given offset in the node.
	fn borrow_item(&self, offset: Offset) -> Option<&Item<K, V>> {
		self.items.get(offset.unwrap())
	}
}

impl<'a, K, V, S: 'a + cc_traits::Slab<Node<K, V>>> btree::node::LeafRef<'a, Storage<K, V, S>> for &'a mut Leaf<K, V> {
	fn parent(&self) -> Option<usize> {
		if self.parent == usize::MAX {
			None
		} else {
			Some(self.parent)
		}
	}

	fn max_capacity(&self) -> usize {
		M+1
	}
}

impl<'a, K, V, S: 'a + cc_traits::SlabMut<Node<K, V>>> btree::node::LeafMut<'a, Storage<K, V, S>> for &'a mut Leaf<K, V> {
	fn set_parent(&mut self, parent: Option<usize>) {
		self.parent = parent.unwrap_or(usize::MAX)
	}

	fn item_mut(&mut self, offset: Offset) -> Option<&mut Item<K, V>> {
		self.items.get_mut(offset.unwrap())
	}

	fn into_item_mut(self, offset: Offset) -> Option<&'a mut Item<K, V>> {
		self.items.get_mut(offset.unwrap())
	}

	fn insert(&mut self, offset: Offset, item: Item<K, V>) {
		self.items.insert(offset.unwrap(), item)
	}

	fn remove(&mut self, offset: Offset) -> Item<K, V> {
		self.items.remove(offset.unwrap())
	}

	fn append(&mut self, separator: Item<K, V>, mut other: Leaf<K, V>) -> Offset {
		let offset = self.items.len().into();
		self.items.push(separator);
		self.items.append(&mut other.items);
		offset
	}
}