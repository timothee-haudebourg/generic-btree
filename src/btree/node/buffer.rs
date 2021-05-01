use super::{
	StorageMut,
	Offset,
	Item
};

mod internal;
mod leaf;

pub use internal::Internal;
pub use leaf::Leaf;

pub enum Buffer<S: StorageMut> {
	Internal(S::InternalNode),
	Leaf(S::LeafNode)
}

impl<S: StorageMut> Buffer<S> {
	pub fn leaf(parent: Option<usize>, item: Item<S::Key, S::Value>) -> Self {
		let mut node = S::LeafNode::default();
		node.set_parent(parent);
		node.push_right(item);
		Self::Leaf(node)
	}

	pub fn binary(parent: Option<usize>, left_child: usize, item: Item<S::Key, S::Value>, right_child: usize) -> Self {
		let mut node = S::InternalNode::default();
		node.set_parent(parent);
		node.set_first_child(left_child);
		node.push_right(item, right_child);
		Self::Internal(node)
	}

	pub fn parent(&self) -> Option<usize> {
		match self {
			Self::Internal(node) => node.parent(),
			Self::Leaf(node) => node.parent()
		}
	}

	pub fn child_count(&self) -> usize {
		match self {
			Self::Internal(node) => node.child_count(),
			Self::Leaf(_) => 0
		}
	}

	pub fn children(&self) -> Children<S> {
		Children {
			inner: match self {
				Buffer::Leaf(_) => None,
				Buffer::Internal(node) => Some(node.children())
			}
		}
	}

	/// Drop this node without dropping the items.
	/// 
	/// Used without care, this may lead to memory leaks.
	#[inline]
	pub fn forget(self) {
		match self {
			Self::Internal(node) => node.forget(),
			Self::Leaf(node) => node.forget()
		}
	}
}

pub struct Children<'a, S: StorageMut> {
	inner: Option<internal::Children<'a, S, S::InternalNode>>
}

impl<'a, S: StorageMut> Iterator for Children<'a, S> {
	type Item = usize;

	fn next(&mut self) -> Option<usize> {
		self.inner.as_mut().map(|inner| inner.next()).flatten()
	}
}