//! AxonDisplay trait.
pub trait AxonDisplay: core::fmt::Display {
    fn fmt_axon(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Display::fmt(self, f)
    }
}
impl<T: core::fmt::Display> AxonDisplay for T {}
#[cfg(test)]
mod tests {
    use super::*; use std::format;
    struct W(i32); impl core::fmt::Display for W { fn fmt(&self,f:&mut core::fmt::Formatter)->core::fmt::Result { write!(f,"W({})",self.0) } }
    #[test] fn display_trait_blanket() { fn req<T:AxonDisplay>(_:&T){} req(&42_i32); req(&"hi"); req(&W(7)); }
    #[test] fn display_fmt_axon_matches_display() { assert_eq!(format!("{}",W(99)),"W(99)"); }
}
