use super::{
	Storage,
	StorageMut,
	Offset
};

pub struct Item<K, V> {
	pub key: K,
	pub value: V
}

impl<K, V> Item<K, V> {
	pub fn new(key: K, value: V) -> Self {
		Self {
			key,
			value
		}
	}

	#[inline]
	pub fn as_pair(&self) -> (&K, &V) {
		(&self.key, &self.value)
	}

	#[inline]
	pub fn into_inner(self) -> (K, V) {
		unsafe {
			// This is safe because `self` if never used/dropped after.
			let key = std::ptr::read(&self.key);
			let value = std::ptr::read(&self.value);
			std::mem::forget(self);
			(key, value)
		}
	}

	#[inline]
	pub fn replace_value(&mut self, mut value: V) -> V {
		std::mem::swap(&mut self.value, &mut value);
		value
	}

	#[inline]
	pub fn into_value(self) -> V {
		self.value
	}

	/// Drop the key but not the value which is assumed uninitialized.
	#[inline]
	pub unsafe fn forget_value(self) {
		let (key, value) = self.into_inner();
		std::mem::drop(key);
		std::mem::forget(value);
	}
}

impl<K, L, V, W> PartialEq<Item<L, W>> for Item<K, V> where L: PartialEq<K>, W: PartialEq<V> {
	fn eq(&self, other: &Item<L, W>) -> bool {
		other.key == self.key && other.value == self.value
	}
}

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

pub trait Replace<S: StorageMut, T> {
	type Output;

	fn replace(&mut self, item: T) -> Self::Output;
}

impl<S: StorageMut, T> Replace<S, S::Item> for T where T: Mut<S> {
	type Output = S::Item;

	fn replace(&mut self, mut item: S::Item) -> S::Item {
		self.swap(&mut item);
		item
	}
}