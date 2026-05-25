//! AxonHash, AxonHasher, FnvHasher.
//!
//! # Security warning
//! FnvHasher is NOT suitable for attacker-controlled input (HashDoS).
//! Never use as the default hasher for network input, IPC, or user keys.
//! axon_alloc will provide a randomised hasher for HashMap.
pub trait AxonHash { fn axon_hash<H: AxonHasher>(&self, state: &mut H); }
pub trait AxonHasher {
    fn write(&mut self, bytes: &[u8]);
    fn finish(&self) -> u64;
    fn write_u8(&mut self,v:u8)     { self.write(&[v]); }
    fn write_u16(&mut self,v:u16)   { self.write(&v.to_le_bytes()); }
    fn write_u32(&mut self,v:u32)   { self.write(&v.to_le_bytes()); }
    fn write_u64(&mut self,v:u64)   { self.write(&v.to_le_bytes()); }
    fn write_u128(&mut self,v:u128) { self.write(&v.to_le_bytes()); }
    fn write_usize(&mut self,v:usize){ self.write_u64(v as u64); }
    fn write_i8(&mut self,v:i8)     { self.write_u8(v as u8); }
    fn write_i16(&mut self,v:i16)   { self.write_u16(v as u16); }
    fn write_i32(&mut self,v:i32)   { self.write_u32(v as u32); }
    fn write_i64(&mut self,v:i64)   { self.write_u64(v as u64); }
    fn write_i128(&mut self,v:i128) { self.write_u128(v as u128); }
    fn write_isize(&mut self,v:isize){ self.write_i64(v as i64); }
}
macro_rules! impl_hash { ($t:ty,$w:ident) => { impl AxonHash for $t { fn axon_hash<H:AxonHasher>(&self,s:&mut H){ s.$w(*self as _); } } }; }
impl_hash!(u8,write_u8); impl_hash!(u16,write_u16); impl_hash!(u32,write_u32);
impl_hash!(u64,write_u64); impl_hash!(u128,write_u128); impl_hash!(usize,write_usize);
impl_hash!(i8,write_i8); impl_hash!(i16,write_i16); impl_hash!(i32,write_i32);
impl_hash!(i64,write_i64); impl_hash!(i128,write_i128); impl_hash!(isize,write_isize);
impl AxonHash for bool { fn axon_hash<H:AxonHasher>(&self,s:&mut H){ s.write_u8(*self as u8); } }
impl AxonHash for &[u8] { fn axon_hash<H:AxonHasher>(&self,s:&mut H){ s.write_usize(self.len()); s.write(self); } }
impl AxonHash for &str  { fn axon_hash<H:AxonHasher>(&self,s:&mut H){ self.as_bytes().axon_hash(s); } }

pub struct FnvHasher(u64);
impl FnvHasher {
    const BASIS: u64 = 14_695_981_039_346_656_037;
    const PRIME: u64 = 1_099_511_628_211;
    pub const fn new() -> Self { Self(Self::BASIS) }
}
impl Default for FnvHasher { fn default() -> Self { Self::new() } }
impl AxonHasher for FnvHasher {
    fn write(&mut self, bytes: &[u8]) { for &b in bytes { self.0 ^= b as u64; self.0 = self.0.wrapping_mul(Self::PRIME); } }
    fn finish(&self) -> u64 { self.0 }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn h<T:AxonHash>(v:T)->u64{ let mut h=FnvHasher::new(); v.axon_hash(&mut h); h.finish() }
    #[test] fn fnv_deterministic()        { assert_eq!(h(42_u64),h(42_u64)); assert_eq!(h("axon"),h("axon")); }
    #[test] fn fnv_different_values_differ(){ assert_ne!(h(0_u64),h(1_u64)); }
    #[test] fn fnv_bool_hash()            { assert_ne!(h(true),h(false)); }
    #[test] fn hasher_all_write_methods() {
        let mut hh=FnvHasher::new();
        hh.write_i8(-1); hh.write_i16(-2); hh.write_i32(-3); hh.write_i64(-4);
        hh.write_i128(-5); hh.write_isize(-6); let _=hh.finish();
    }
    #[test] fn hash_i16_and_i128_consistent(){ assert_eq!(h(-32768_i16),h(-32768_i16)); assert_eq!(h(i128::MAX),h(i128::MAX)); }
    #[test] fn hasher_write_methods_complete() {
        let mut hh=FnvHasher::new();
        hh.write_i16(-32768_i16); hh.write_i128(i128::MIN); let _=hh.finish();
    }
    #[test] fn hasher_signed_unsigned_symmetry() {
        let mut h1=FnvHasher::new(); h1.write_i16(-1_i16);
        let mut h2=FnvHasher::new(); h2.write_u16(0xFFFF_u16);
        assert_eq!(h1.finish(),h2.finish());
    }
}
