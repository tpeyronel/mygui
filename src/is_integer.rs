pub trait IsInteger {
    fn is_integer(&self) -> bool;
}

impl IsInteger for f32 {
    fn is_integer(&self) -> bool {
        self.fract() == 0.0
    }
}