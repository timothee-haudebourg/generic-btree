use std::ops::{
	Deref,
	DerefMut
};
use super::{
	Storage,
	StorageMut,
	Offset
};

pub type StorageItem<S> = Item<<S as Storage>::Key, <S as Storage>::Value>;

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

pub trait ItemAccess<'a, S: 'a + Storage> {
	fn item_count(&self) -> usize;

	fn is_empty(&self) -> bool {
		self.item_count() == 0
	}

	fn item(&self, offset: Offset) -> Option<S::ItemRef<'a>>;
}

impl<'a, T, S: 'a + Storage> ItemAccess<'a, S> for &'a mut T where &'a T: ItemAccess<'a, S> {
	fn item_count(&self) -> usize {
		self.item_count()
	}

	fn item(&self, offset: Offset) -> Option<S::ItemRef<'a>> {
		self.item(offset)
	}
}

/// Item reference.
pub trait Ref<'a, S: 'a + Storage> {
	/// Returns a reference to the item key.
	fn key(&self) -> S::KeyRef<'a>;

	/// Returns a refrence to the item value.
	fn value(&self) -> S::ValueRef<'a>;

	#[inline]
	fn split(&self) -> (S::KeyRef<'a>, S::ValueRef<'a>) {
		(self.key(), self.value())
	}
}

impl<'a, T, S: 'a + Storage> Ref<'a, S> for &'a mut T where &'a T: Ref<'a, S> {
	fn key(&self) -> S::KeyRef<'a> {
		self.key()
	}

	fn value(&self) -> S::ValueRef<'a> {
		self.value()
	}
}

/// Item reference.
pub trait Mut<'a, S: 'a + StorageMut>: Ref<'a, S> {
	/// Returns a mutable reference to the item key.
	fn key_mut(&mut self) -> S::KeyMut<'_>;

	fn into_key_mut(self) -> S::KeyMut<'a>;

	/// Returns a mutable reference to the item value.
	/// 
	/// # Safety
	/// 
	/// The caller must ensure that the item has been initialized.
	fn value_mut(&mut self) -> S::ValueMut<'_>;

	fn into_value_mut(self) -> S::ValueMut<'a>;

	fn swap(&mut self, item: &mut Item<S::Key, S::Value>) {
		std::mem::swap(self.key_mut().deref_mut(), &mut item.key);
		std::mem::swap(self.value_mut().deref_mut(), &mut item.value);
	}

	fn replace(&mut self, mut item: Item<S::Key, S::Value>) -> Item<S::Key, S::Value> {
		self.swap(&mut item);
		item
	}

	fn set(&mut self, key: S::Key, value: S::Value) -> Item<S::Key, S::Value> {
		let mut item = Item::new(key, value);
		self.swap(&mut item);
		item
	}

	fn set_value(&mut self, mut value: S::Value) -> S::Value {
		std::mem::swap(self.value_mut().deref_mut(), &mut value);
		value
	}
}