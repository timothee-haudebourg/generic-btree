use super::{
	btree,
	Node,
	Storage
};

pub type Item<K, V> = crate::btree::node::Item<K, V>;

impl<'r, K, V, S: cc_traits::SlabMut<Node<K, V>>> btree::node::item::Mut<'r, &'r mut Storage<K, V, S>> for &'r mut Item<K, V> {
	// ...
}