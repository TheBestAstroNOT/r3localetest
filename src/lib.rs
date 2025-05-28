#![feature(portable_simd)]

use std::simd::Simd;
use std::simd::Mask;
use std::simd::cmp::SimdPartialEq;

// SVE (Scalable Vector Extension) - ARM 64-bit
#[cfg(all(target_arch = "aarch64", target_feature = "sve"))]
pub const LANES: usize = 16; // example fixed width for SVE, adjust as needed

// NEON SIMD - ARM 64-bit
#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
pub const LANES: usize = 16; // NEON 128-bit = 16 * u8 lanes

// Fallback for aarch64 without explicit SIMD features
#[cfg(all(target_arch = "aarch64", not(any(target_feature = "sve", target_feature = "neon"))))]
pub const LANES: usize = 16; // assume NEON to avoid scalar fallback on ARM64

// AVX-512 - x86_64
#[cfg(all(target_arch = "x86_64", target_feature = "avx512f"))]
pub const LANES: usize = 64; // 512-bit = 64 * u8 lanes

// AVX2 - x86_64 without AVX-512
#[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
#[cfg(not(target_feature = "avx512f"))]
pub const LANES: usize = 32; // 256-bit = 32 * u8 lanes

// SSE2 - x86_64 without AVX2 or AVX-512
#[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
#[cfg(not(any(target_feature = "avx2", target_feature = "avx512f")))]
pub const LANES: usize = 16; // 128-bit = 16 * u8 lanes

// Scalar fallback
#[cfg(not(any(
    all(target_arch = "aarch64", target_feature = "sve"),
    all(target_arch = "aarch64", target_feature = "neon"),
    all(target_arch = "x86_64", target_feature = "avx512f"),
    all(target_arch = "x86_64", target_feature = "avx2"),
    all(target_arch = "x86_64", target_feature = "sse2"),
)))]
pub const LANES: usize = 1;


pub fn parse_keys_simd_bracketonly(){
    println!("Using size: {}", LANES);
        let input = "[[Key1]]\nValue1\n\
                    [[Key2]]\nValue2\n\
                    [[UserName]]\nAlice\n\
                    [[UserAge]]\n30\n\
                    [[UserEmail]]\nalice@example.com\n\
                    [[Settings]]\n\
                      [[Theme]]\nDark\n\
                      [[FontSize]]\n14\n\
                    [[Logs]]\nLog entry 1\nLog entry 2\nLog entry 3\n\
                    [[End]]\n";
        let bytes = input.as_bytes();
        let mut opening_matches = Vec::new();
        let mut closing_matches = Vec::new();
        let mut i = 0;

        while i + LANES + 2 <= bytes.len() {
            let opening_bitmask = get_bitmask(i, bytes, [Some(b'['), Some(b'['), None]);

            for lane in 0..LANES {
                if (opening_bitmask & (1 << lane)) != 0{
                    if i + lane > 0 {
                        if bytes[i + lane - 1] == b'\n'{
                            opening_matches.push(i + lane + 2);
                            let closing_bitmask = get_bitmask(i, bytes, [Some(b']'), Some(b']'), None]);
                            for lane in 0..LANES {
                                if (closing_bitmask & (1 << lane)) != 0 {
                                    closing_matches.push(i + lane);
                                }
                            }
                        }
                    }
                    else{
                       opening_matches.push(i + lane + 2);
                       let closing_bitmask = get_bitmask(i, bytes, [Some(b']'), Some(b']'), None]);
                       for lane in 0..LANES {
                            if (closing_bitmask & (1 << lane)) != 0 {
                               closing_matches.push(i + lane);
                           }
                       }
                    }
                }
            }

            i += 1;
        }

        opening_matches.sort_unstable();
        opening_matches.dedup();
        closing_matches.sort_unstable();
        closing_matches.dedup();

        let mut last_close: usize = 0;

        for (&open_pos, &close_pos) in opening_matches.iter().zip(closing_matches.iter()) {
            if open_pos < close_pos && close_pos <= bytes.len() {
                if last_close != 0 {
                    println!("Value: {}", String::from_utf8_lossy(&bytes[last_close..open_pos - 2]));
                }

                println!("Key: {}", String::from_utf8_lossy(&bytes[open_pos..close_pos]));
                last_close = close_pos + 2; // skip the closing "]]"
            } else {
                eprintln!(
                    "Skipping invalid range: open={}, close={}, len={}",
                    open_pos, close_pos, bytes.len()
                );
            }
        }
    if last_close < bytes.len() {
        println!("Value: {}", String::from_utf8_lossy(&bytes[last_close..]));
    }
}

pub fn parse_keys_simd_open_bracketonly(){
    println!("Using size: {}", LANES);
        let input = "\
                    \n[[Key1]]\nValue1\n\
                    [[Key2]]\nValue2\n\
                    [[UserName]]\nAlice\n\
                    [[UserAge]]\n30\n\
                    [[UserEmail]]\nalice@example.com\n\
                    [[Settings]]\n\
                      [[Theme]]\nDark\n\
                      [[FontSize]]\n14\n\
                    [[Logs]]\nLog entry 1\nLog entry 2\nLog entry 3\n\
                    [[End]]\n";
        let bytes = input.as_bytes();
        let mut opening_matches = Vec::new();
        let mut closing_matches = Vec::new();
        let mut i = 0;

        while i + LANES + 2 <= bytes.len() {
            let opening_bitmask = get_bitmask(i, bytes, [Some(b'['), Some(b'['), None]);

            for lane in 0..LANES {
                if (opening_bitmask & (1 << lane)) != 0{
                    if i + lane > 0 && bytes[i + lane - 1] == b'\n'{
                        opening_matches.push(i + lane + 2);
                        let mut j = 0;
                        while bytes[i + lane + j] != b']' && bytes[i + lane + j + 1] != b']'{
                            j+=1;
                        }
                        closing_matches.push(i + lane + j);
                    }
                }
            }

            i += 1;
        }

        opening_matches.sort_unstable();
        opening_matches.dedup();
        closing_matches.sort_unstable();
        closing_matches.dedup();

        for (&open_pos, &close_pos) in opening_matches.iter().zip(closing_matches.iter()) {
            if open_pos < close_pos && close_pos <= bytes.len() {
                    println!("Key: {}", String::from_utf8_lossy(&bytes[open_pos..close_pos]));
                } else {
                    eprintln!(
                        "Skipping invalid range: open={}, close={}, len={}",
                        open_pos,
                        close_pos,
                        bytes.len()
                    );
                }
        }
}

pub fn parse_keys_simd_all() {
    println!("Using size: {}", LANES);
    let input = "\
                        \n[[Key1]]\nValue1\n\
                        [[Key2]]\nValue2\n\
                        [[UserName]]\nAlice\n\
                        [[UserAge]]\n30\n\
                        [[UserEmail]]\nalice@example.com\n\
                        [[Settings]]\n\
                          [[Theme]]\nDark\n\
                          [[FontSize]]\n14\n\
                        [[Logs]]\nLog entry 1\nLog entry 2\nLog entry 3\n\
                        [[End]]\n";
    let bytes = input.as_bytes();
    let mut openingMatches = Vec::new();
    let mut closingMatches = Vec::new();
    let mut i = 0;

    while i + LANES + 2 <= bytes.len() {
        let opening_bitmask = get_bitmask(i, bytes, [Some(b'\n'), Some(b'['), Some(b'[')]);

        for lane in 0..LANES {
            if (opening_bitmask & (1 << lane)) != 0 {
                openingMatches.push(i + lane + 3);
                let closing_bitmask = get_bitmask(i, bytes, [Some(b']'), Some(b']'), None]);
                for lane in 0..LANES {
                    if (closing_bitmask & (1 << lane)) != 0 {
                        closingMatches.push(i + lane);
                    }
                }
            }
        }

        i += 1;
    }

    openingMatches.sort_unstable();
    openingMatches.dedup();
    closingMatches.sort_unstable();
    closingMatches.dedup();

    for (&open_pos, &close_pos) in openingMatches.iter().zip(closingMatches.iter()) {
        if open_pos < close_pos && close_pos <= bytes.len() {
                println!("Key: {}", String::from_utf8_lossy(&bytes[open_pos..close_pos]));
            } else {
                eprintln!(
                    "Skipping invalid range: open={}, close={}, len={}",
                    open_pos,
                    close_pos,
                    bytes.len()
                );
            }
    }
}

fn get_bitmask(i:usize, bytes: &[u8], pattern: [Option<u8>; 3]) -> u64 {

        let chunk0 = Simd::<u8, LANES>::from_slice(&bytes[i..i + LANES]);
        let chunk1 = Simd::<u8, LANES>::from_slice(&bytes[i + 1..i + LANES + 1]);
        let chunk2 = Simd::<u8, LANES>::from_slice(&bytes[i + 2..i + LANES + 2]);

        let mask0 = match pattern[0] {
                Some(b) => chunk0.simd_eq(Simd::splat(b)),
                None => Mask::<i8, LANES>::splat(true),
            };
            let mask1 = match pattern[1] {
                Some(b) => chunk1.simd_eq(Simd::splat(b)),
                None => Mask::<i8, LANES>::splat(true),
            };
            let mask2 = match pattern[2] {
                Some(b) => chunk2.simd_eq(Simd::splat(b)),
                None => Mask::<i8, LANES>::splat(true),
            };

        let resultant = mask0 & mask1 & mask2;

        return resultant.to_bitmask();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_keys() {
        let start = std::time::Instant::now();
        parse_keys_simd_bracketonly();
        let duration = start.elapsed();
        println!("parse_keys_simd_bracketonly took: {:?}", duration);

        let start = std::time::Instant::now();
        parse_keys_simd_all();
        let duration = start.elapsed();
        println!("parse_keys_simd_all took: {:?}", duration);
    }
}