pub trait MaskExt {
    fn all(&self) -> bool;
    fn any(&self) -> bool;
}

impl<const N: usize> MaskExt for [bool; N] {
    fn all(&self) -> bool {
        self.iter().all(|&b| b)
    }
    fn any(&self) -> bool {
        self.iter().any(|&b| b)
    }
}
