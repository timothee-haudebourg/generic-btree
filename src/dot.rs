pub trait Display {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result;

    fn dot(&self) -> Displayed<Self> {
        Displayed(self)
    }
}

pub struct Displayed<'a, T: ?Sized>(&'a T);

impl<'a, T: Display> std::fmt::Display for Displayed<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
