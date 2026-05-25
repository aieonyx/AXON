//! AxonFrom / AxonInto conversion traits.
pub trait AxonFrom<T>: Sized { fn axon_from(value: T) -> Self; }
pub trait AxonInto<T>: Sized { fn axon_into(self) -> T; }
impl<T, U: AxonFrom<T>> AxonInto<U> for T { fn axon_into(self) -> U { U::axon_from(self) } }
impl<T> AxonFrom<T> for T { fn axon_from(v: T) -> T { v } }
#[cfg(test)]
mod tests {
    use super::*;
    struct Km(f64); impl AxonFrom<f64> for Km { fn axon_from(v:f64)->Self{Km(v)} }
    #[test] fn axon_from_blanket()  { assert!((Km::axon_from(42.0).0-42.0).abs()<f64::EPSILON); }
    #[test] fn axon_into_blanket()  { let k:Km=(7.5_f64).axon_into(); assert!((k.0-7.5).abs()<f64::EPSILON); }
    #[test] fn axon_from_reflexive(){ assert_eq!(i32::axon_from(99),99); }
    #[test] fn axon_into_reflexive(){ let x:i32=55_i32.axon_into(); assert_eq!(x,55); }
}
