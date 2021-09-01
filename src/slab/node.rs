use std::borrow::Borrow;
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

pub enum Node<T> {
	Internal(Internal<T>),
	Leaf(Leaf<T>)
}

impl<T, S: cc_traits::SlabMut<Node<T>>> From<Buffer<Storage<T, S>>> for Node<T> {
	fn from(node: Buffer<Storage<T, S>>) -> Self {
		match node {
			Buffer::Internal(node) => Self::Internal(node),
			Buffer::Leaf(node) => Self::Leaf(node)
		}
	}
}

impl<T, S: cc_traits::SlabMut<Node<T>>> From<Node<T>> for Buffer<Storage<T, S>> {
	fn from(node: Node<T>) -> Self {
		match node {
			Node::Internal(node) => Self::Internal(node),
			Node::Leaf(node) => Self::Leaf(node)
		}
	}
}

impl<'r, T, S: 'r + cc_traits::Slab<Node<T>>> From<&'r Node<T>> for Ref<'r, Storage<T, S>> {
	fn from(n: &'r Node<T>) -> Self {
		match n {
			Node::Internal(node) => Self::internal(node),
			Node::Leaf(node) => Self::leaf(node)
		}
	}
}

impl<'r, T, S: 'r + cc_traits::SlabMut<Node<T>>> From<&'r mut Node<T>> for Mut<'r, Storage<T, S>> {
	fn from(n: &'r mut Node<T>) -> Self {
		match n {
			Node::Internal(node) => Self::internal(node),
			Node::Leaf(node) => Self::leaf(node)
		}
	}
}