use super::{
	btree,
	Node,
	Storage
};

pub type Item<K, V> = crate::btree::node::Item<K, V>;

impl<'a, K, V, S: 'a + cc_traits::Slab<Node<K, V>>> btree::node::item::Ref<'a, Storage<K, V, S>> for &'a Item<K, V> {
	fn key(&self) -> &'a K {
		&self.key
	}

	fn value(&self) -> &'a V {
		&self.value
	}
}

impl<'a, K, V, S: 'a + cc_traits::SlabMut<Node<K, V>>> btree::node::item::Mut<'a, Storage<K, V, S>> for &'a mut Item<K, V> {
	fn key(&self) -> &K {
		&self.key
	}

	fn value(&self) -> &V {
		&self.value
	}
	
	fn key_mut(&mut self) -> &mut K {
		&mut self.key
	}

	fn into_key_mut(self) -> &'a mut K {
		&mut self.key
	}

	fn value_mut(&mut self) -> &mut V {
		&mut self.value
	}

	fn into_value_mut(self) -> &'a mut V {
		&mut self.value
	}

	fn into_pair_mut(self) -> (&'a mut K, &'a mut V) {
		(&mut self.key, &mut self.value)
	}
}