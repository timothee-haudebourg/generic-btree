use super::StorageMut;

/// Leaf node buffer.
pub trait Leaf<S: StorageMut>: Default {
	fn parent(&self) -> Option<usize>;

	fn set_parent(&mut self, parent: Option<usize>);

	fn item_count(&self) -> usize;

	// fn item<'a>(&'a self, offset: Offset) -> Option<S::ItemRef<'a>> where S: 'a;

	fn max_capacity(&self) -> usize;

	fn push_right(&mut self, item: S::Item);
	/// Drop this leaf node without dropping the items.
	///eee
	/// Used without care, this may lead to memory leaks.
	fn forget(self);
}