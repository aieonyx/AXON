//! AXON primitive type aliases.
pub type AxonInt   = i64;  pub type AxonUint  = u64;
pub type AxonFloat = f64;  pub type AxonBool  = bool; pub type AxonByte = u8;
pub type I8=i8; pub type U8=u8; pub type I16=i16; pub type U16=u16;
pub type I32=i32; pub type U32=u32; pub type I64=i64; pub type U64=u64;
pub type I128=i128; pub type U128=u128;
pub type F32=f32; pub type F64=f64;
pub type Usize=usize; pub type Isize=isize;

#[cfg(test)]
mod tests {
    use super::*; use core::mem::size_of;
    #[test] fn types_aliases_are_correct_sizes() {
        assert_eq!(size_of::<AxonInt>(),8); assert_eq!(size_of::<AxonFloat>(),8);
        assert_eq!(size_of::<AxonBool>(),1); assert_eq!(size_of::<I128>(),16);
        assert_eq!(size_of::<F32>(),4);
    }
    #[test] fn axon_int_arithmetic() { assert_eq!(100_i64 - 42, 58); }
    #[test] fn axon_float_precision() { assert!((1.0_f64/3.0 - 0.333_333_333_333_333_3).abs() < 1e-15); }
}
