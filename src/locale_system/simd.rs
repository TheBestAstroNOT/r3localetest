use std::simd::Mask;
use std::simd::Simd;
use std::simd::prelude::SimdPartialEq;

#[cfg(all(target_arch = "aarch64", target_feature = "sve"))]
pub const LANES: usize = 16;

#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
pub const LANES: usize = 16;

#[cfg(all(target_arch = "x86_64", target_feature = "avx512f"))]
pub const LANES: usize = 64;

#[cfg(all(target_arch = "x86_64", target_feature = "avx2", not(target_feature = "avx512f")))]
pub const LANES: usize = 32;

#[cfg(all(target_arch = "x86_64", target_feature = "sse2", not(any(target_feature = "avx2", target_feature = "avx512f"))))]
pub const LANES: usize = 16;

#[cfg(not(any(
    all(target_arch = "aarch64", any(target_feature = "sve", target_feature = "neon")),
    all(target_arch = "x86_64", any(target_feature = "avx512f", target_feature = "avx2", target_feature = "sse2"))
)))]
pub const LANES: usize = 1;

pub fn get_bitmask(i: usize, bytes: &[u8], pattern: [Option<u8>; 3]) -> u64 {
    let chunk0 = Simd::<u8, LANES>::from_slice(&bytes[i..i + LANES]);
    let chunk1 = Simd::<u8, LANES>::from_slice(&bytes[i + 1..i + LANES + 1]);
    let chunk2 = Simd::<u8, LANES>::from_slice(&bytes[i + 2..i + LANES + 2]);

    let mask0 = pattern[0].map_or(Mask::<i8, LANES>::splat(true), |b| chunk0.simd_eq(Simd::splat(b)));
    let mask1 = pattern[1].map_or(Mask::<i8, LANES>::splat(true), |b| chunk1.simd_eq(Simd::splat(b)));
    let mask2 = pattern[2].map_or(Mask::<i8, LANES>::splat(true), |b| chunk2.simd_eq(Simd::splat(b)));

    (mask0 & mask1 & mask2).to_bitmask()
}