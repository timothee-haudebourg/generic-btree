use crate::btree::node::{
	Buffer,
	Ref,
	Mut
};
use super::Storage;

mod leaf;
mod internal;

pub use leaf::Leaf;
pub use internal::Internal;

pub enum Node<K, V> {
	Internal(Internal<K, V>),
	Leaf(Leaf<K, V>)
}

impl<'s, K, V, S: cc_traits::SlabMut<Node<K, V>>> From<Buffer<'s, &'s mut Storage<K, V, S>>> for Node<K, V> {
	fn from(node: Buffer<'s, &'s mut Storage<K, V, S>>) -> Self {
		match node {
			Buffer::Internal(node) => Self::Internal(node),
			Buffer::Leaf(node) => Self::Leaf(node)
		}
	}
}

impl<'s, K, V, S: cc_traits::SlabMut<Node<K, V>>> From<Node<K, V>> for Buffer<'s, &'s mut Storage<K, V, S>> {
	fn from(node: Node<K, V>) -> Self {
		match node {
			Node::Internal(node) => Self::Internal(node),
			Node::Leaf(node) => Self::Leaf(node)
		}
	}
}

impl<'s, K, V, S: 's + cc_traits::Slab<Node<K, V>>> From<&'s Node<K, V>> for Ref<'s, &'s Storage<K, V, S>> {
	fn from(n: &'s Node<K, V>) -> Self {
		match n {
			Node::Internal(node) => Self::internal(node),
			Node::Leaf(node) => Self::leaf(node)
		}
	}
}

impl<'r, 's: 'r, K, V, S: 's + cc_traits::SlabMut<Node<K, V>>> From<&'r mut Node<K, V>> for Mut<'r, 's, &'s mut Storage<K, V, S>> {
	fn from(n: &'r mut Node<K, V>) -> Self {
		// match n {
		// 	Node::Internal(node) => Self::internal(node),
		// 	Node::Leaf(node) => Self::leaf(node)
		// }
		panic!("TODO")
	}
}