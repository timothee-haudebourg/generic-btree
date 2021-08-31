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

impl<K, L, V, W> PartialEq<Binding<L, W>> for Binding<K, V> where L: PartialEq<K>, W: PartialEq<V> {
	fn eq(&self, other: &Binding<L, W>) -> bool {
		other.key == self.key && other.value == self.value
	}
}