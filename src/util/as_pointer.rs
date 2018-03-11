use std::fmt::{Debug, Formatter, Pointer, Result as FmtResult};

pub struct AsPointer<'a, T: 'a + Pointer>(pub &'a T);

impl<'a, T: Pointer> Debug for AsPointer<'a, T> {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        Pointer::fmt(&self.0, fmt)
    }
}
