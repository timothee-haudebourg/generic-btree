use std::{
	cmp::Ordering,
	borrow::Borrow
};

pub struct Binding<K, V> {
	pub key: K,
	pub value: V
}

impl<K, V> Binding<K, V> {
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
	pub fn into_pair(self) -> (K, V) {
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
	/// 
	/// ## Safety
	/// 
	/// The caller must ensure that the value has already been dropped,
	/// or has been copied outside the binding and will properly be dropped later.
	#[inline]
	pub unsafe fn forget_value(self) {
		let (key, value) = self.into_pair();
		std::mem::drop(key);
		std::mem::forget(value);
	}
}

impl<'a, K, V> From<&'a Binding<K, V>> for (&'a K, &'a V) {
	fn from(binding: &'a Binding<K, V>) -> Self {
		binding.as_pair()
	}
}

impl<'a, K, V> From<&'a mut Binding<K, V>> for (&'a K, &'a mut V) {
	fn from(binding: &'a mut Binding<K, V>) -> Self {
		(&binding.key, &mut binding.value)
	}
}

impl<K, L, V, W> PartialOrd<Binding<L, W>> for Binding<K, V> where K: PartialOrd<L>, V: PartialOrd<W> {
	fn partial_cmp(&self, other: &Binding<L, W>) -> Option<Ordering> {
		match self.key.partial_cmp(&other.key) {
			Some(Ordering::Equal) => self.value.partial_cmp(&other.value),
			o => o
		}
	}
}

impl<K, L, V, W> PartialEq<Binding<L, W>> for Binding<K, V> where K: PartialEq<L>, V: PartialEq<W> {
	fn eq(&self, other: &Binding<L, W>) -> bool {
		self.key == other.key && self.value == other.value
	}
}

impl<K, V> Eq for Binding<K, V> where K: Eq, V: Eq {}

impl<K, V> Ord for Binding<K, V> where K: Ord, V: Ord {
	fn cmp(&self, other: &Self) -> Ordering {
		match self.key.cmp(&other.key) {
			Ordering::Equal => self.value.cmp(&other.value),
			o => o
		}
	}
}

impl<K, V> Borrow<K> for Binding<K, V> {
	fn borrow(&self) -> &K {
		&self.key
	}
}