use super::{
	StorageMut,
	Offset,
	Item
};

/// Leaf node buffer.
pub trait Leaf<S: StorageMut>: Default {
	fn parent(&self) -> Option<usize>;

	fn set_parent(&mut self, parent: Option<usize>);

	fn item_count(&self) -> usize;

	fn item(&self, offset: Offset) -> Option<S::ItemRef<'_>>;

	fn max_capacity(&self) -> usize;

	fn push_right(&mut self, item: S::Item);

	/// Drop this leaf node without dropping the items.
	///
	/// Used without care, this may lead to memory leaks.
	fn forget(self);
}