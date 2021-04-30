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

impl<K, V, S: cc_traits::SlabMut<Node<K, V>>> From<Buffer<Storage<K, V, S>>> for Node<K, V> {
	fn from(node: Buffer<Storage<K, V, S>>) -> Self {
		match node {
			Buffer::Internal(node) => Self::Internal(node),
			Buffer::Leaf(node) => Self::Leaf(node)
		}
	}
}

impl<K, V, S: cc_traits::SlabMut<Node<K, V>>> From<Node<K, V>> for Buffer<Storage<K, V, S>> {
	fn from(node: Node<K, V>) -> Self {
		match node {
			Node::Internal(node) => Self::Internal(node),
			Node::Leaf(node) => Self::Leaf(node)
		}
	}
}

impl<'a, K, V, S: 'a + cc_traits::Slab<Node<K, V>>> From<&'a Node<K, V>> for Ref<'a, Storage<K, V, S>> {
	fn from(n: &'a Node<K, V>) -> Self {
		match n {
			Node::Internal(node) => Self::internal(node),
			Node::Leaf(node) => Self::leaf(node)
		}
	}
}

impl<'a, K, V, S: 'a + cc_traits::SlabMut<Node<K, V>>> From<&'a mut Node<K, V>> for Mut<'a, Storage<K, V, S>> {
	fn from(n: &'a mut Node<K, V>) -> Self {
		match n {
			Node::Internal(node) => Self::internal(node),
			Node::Leaf(node) => Self::leaf(node)
		}
	}
}