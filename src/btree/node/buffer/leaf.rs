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

	fn item(&self, offset: Offset) -> Option<&Item<S::Key, S::Value>>;

	fn max_capacity(&self) -> usize;

	fn push_right(&mut self, item: Item<S::Key, S::Value>);
}