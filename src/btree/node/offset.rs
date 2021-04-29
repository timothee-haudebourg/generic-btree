use std::{
	cmp::Ordering,
	ops::{
		Add,
		Sub,
		Div
	},
	fmt
};

/// Offset in a node.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Offset(usize);

impl Offset {
	pub fn before() -> Offset {
		Offset(usize::MAX)
	}

	pub fn is_before(&self) -> bool {
		self.0 == usize::MAX
	}

	pub fn value(&self) -> Option<usize> {
		if self.0 == usize::MAX {
			None
		} else {
			Some(self.0)
		}
	}

	pub fn unwrap(self) -> usize {
		if self.0 == usize::MAX {
			panic!("Offset out of bounds")
		} else {
			self.0
		}
	}

	pub fn incr(&mut self) {
		if self.0 == usize::MAX {
			self.0 = 0
		} else {
			self.0 += 1
		}
	}

	pub fn decr(&mut self) {
		if self.0 == 0 {
			self.0 = usize::MAX
		} else {
			self.0 -= 1
		}
	}
}

impl PartialOrd for Offset {
	fn partial_cmp(&self, offset: &Offset) -> Option<Ordering> {
		if self.0 == usize::MAX || offset.0 == usize::MAX {
			if self.0 == usize::MAX && offset.0 == usize::MAX {
				Some(Ordering::Equal)
			} else if self.0 == usize::MAX {
				Some(Ordering::Less)
			} else {
				Some(Ordering::Greater)
			}
		} else {
			self.0.partial_cmp(&offset.0)
		}
	}
}

impl Ord for Offset {
	#[inline]
	fn cmp(&self, offset: &Offset) -> Ordering {
		if self.0 == usize::MAX || offset.0 == usize::MAX {
			if self.0 == usize::MAX && offset.0 == usize::MAX {
				Ordering::Equal
			} else if self.0 == usize::MAX {
				Ordering::Less
			} else {
				Ordering::Greater
			}
		} else {
			self.0.cmp(&offset.0)
		}
	}
}

impl PartialEq<usize> for Offset {
	#[inline]
	fn eq(&self, offset: &usize) -> bool {
		self.0 != usize::MAX && self.0 == *offset
	}
}

impl PartialOrd<usize> for Offset {
	#[inline]
	fn partial_cmp(&self, offset: &usize) -> Option<Ordering> {
		if self.0 == usize::MAX {
			Some(Ordering::Less)
		} else {
			self.0.partial_cmp(offset)
		}
	}
}

impl Add for Offset {
	type Output = Self;

	#[inline]
	fn add(self, rhs: Self) -> Self {
		if self.0 == usize::MAX {
			if rhs.0 == usize::MAX {
				panic!("offset underflow")
			} else {
				rhs - 1
			}
		} else {
			Self(self.0 + rhs.0)
		}
	}
}

impl Add<usize> for Offset {
	type Output = Self;

	#[inline]
	fn add(self, rhs: usize) -> Self {
		if self.0 == usize::MAX {
			Self(rhs - 1)
		} else {
			Self(self.0 + rhs)
		}
	}
}

impl Sub for Offset {
	type Output = Self;

	#[inline]
	fn sub(self, rhs: Self) -> Self {
		if self.0 == usize::MAX {
			if rhs.0 == usize::MAX {
				Self(0)
			} else {
				panic!("offset underflow")
			}
		} else if self.0 >= rhs.0 {
			Self(self.0 - rhs.0)
		} else if rhs.0 + 1 == self.0 {
			Self(usize::MAX)
		} else {
			panic!("offset underflow")
		}
	}
}

impl Sub<usize> for Offset {
	type Output = Self;

	#[inline]
	fn sub(self, rhs: usize) -> Self {
		if self.0 == usize::MAX {
			panic!("offset underflow")
		} else if self.0 >= rhs {
			Self(self.0 - rhs)
		} else if rhs + 1 == self.0 {
			Self(usize::MAX)
		} else {
			panic!("offset underflow")
		}
	}
}

impl Div<usize> for Offset {
	type Output = Self;

	#[inline]
	fn div(self, rhs: usize) -> Self {
		if self.0 == usize::MAX {
			panic!("offset underflow")
		} else {
			Self(self.0 / rhs)
		}
	}
}

impl From<usize> for Offset {
	#[inline]
	fn from(offset: usize) -> Offset {
		Offset(offset)
	}
}

impl fmt::Display for Offset {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		if self.0 == usize::MAX {
			write!(f, "-1")
		} else {
			self.0.fmt(f)
		}
	}
}

impl fmt::Debug for Offset {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		if self.0 == usize::MAX {
			write!(f, "-1")
		} else {
			self.0.fmt(f)
		}
	}
}