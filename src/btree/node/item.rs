use super::{
	Storage,
	StorageMut,
	Offset
};

pub trait ItemAccess<S: Storage> {
	fn item_count(&self) -> usize;

	fn is_empty(&self) -> bool {
		self.item_count() == 0
	}

	fn borrow_item(&self, offset: Offset) -> Option<S::ItemRef<'_>>;
}

/// Item reference.
pub trait Mut<S: StorageMut> {
	fn swap(&mut self, item: &mut S::Item);
}

/// Item reference that can be unsafely copied.
/// 
/// This trait is used to optimize the consumption of the tree.
pub unsafe trait Read<S: StorageMut> {
	/// Copy the item.
	/// 
	/// # Safety
	/// 
	/// This function is unsafe because
	/// an item may not implement the `Copy` trait.
	/// The caller must ensure that the underlying item
	/// will be disposed of without running `drop`.
	unsafe fn read(&self) -> S::Item;
}

/// Item mutable reference that can be unsafely written to.
pub unsafe trait Write<S: StorageMut> {
	/// Write the item.
	/// 
	/// The previous value is not dropped.
	/// 
	/// # Safety
	/// 
	/// This function is unsafe because
	/// the previous value is not dropped.
	unsafe fn write(&mut self, item: S::Item);
}

/// Item mutable reference that can be replaced using a value of type `T`.
pub trait Replace<S: StorageMut, T> {
	type Output;

	/// Replace the item (or part of the item) using the given value of type `T`.
	/// 
	/// Returns a `Self::Output` representing the replaced value.
	fn replace(&mut self, item: T) -> Self::Output;
}

impl<S: StorageMut, T> Replace<S, S::Item> for T where T: Mut<S> {
	type Output = S::Item;

	fn replace(&mut self, mut item: S::Item) -> S::Item {
		self.swap(&mut item);
		item
	}
}