use crate::bits::*;
use crate::memcmp::*;

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

#[cfg(target_feature = "avx2")]
pub unsafe fn strstr_avx2_rust_simple(haystack: &[u8], needle: &[u8]) -> bool {
    let first = _mm256_set1_epi8(needle[0] as i8);
    let last = _mm256_set1_epi8(needle[needle.len() - 1] as i8);
    let mut block_pad: [u8; 32] = [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
    ];
    for chunk in haystack.chunks(32) {
        let i = chunk.as_ptr() as usize - haystack.as_ptr() as usize;
        let block_first = if chunk.len() == 32 {
            _mm256_loadu_si256(chunk.as_ptr() as *const __m256i)
        } else {
            block_pad[..chunk.len()].copy_from_slice(chunk);
            _mm256_loadu_si256(block_pad.as_ptr() as *const __m256i)
        };
        let block_last = if i + 31 + needle.len() <= haystack.len() {
            _mm256_loadu_si256(chunk[(needle.len() - 1)..].as_ptr() as *const __m256i)
        } else {
            let start = &haystack[(i + needle.len() - 1)..];
            block_pad[..start.len()].copy_from_slice(start);
            _mm256_loadu_si256(block_pad.as_ptr() as *const __m256i)
        };

        let eq_first = _mm256_cmpeq_epi8(first, block_first);
        let eq_last = _mm256_cmpeq_epi8(last, block_last);

        let mut mask = std::mem::transmute::<i32, u32>(_mm256_movemask_epi8(_mm256_and_si256(
            eq_first, eq_last,
        )));
        while mask != 0 {
            let bitpos = mask.trailing_zeros() as usize;
            let startpos = i + bitpos;
            if startpos + needle.len() <= haystack.len()
                && haystack[(startpos + 1)..(startpos + needle.len() - 1)]
                    == needle[1..needle.len() - 1]
            {
                return true;
            }
            mask = clear_leftmost_set(mask);
        }
    }

    false
}

#[cfg(target_feature = "avx2")]
pub unsafe fn strstr_avx2_rust_simple_2(haystack: &[u8], needle: &[u8]) -> bool {
    let first = _mm256_set1_epi8(needle[0] as i8);
    let last = _mm256_set1_epi8(needle[needle.len() - 1] as i8);
    let mut block_pad: [u8; 32] = [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
    ];
    let mut chunks = haystack.chunks_exact(32);
    while let Some(chunk) = chunks.next() {
        let i = chunk.as_ptr() as usize - haystack.as_ptr() as usize;
        let block_first = _mm256_loadu_si256(chunk.as_ptr() as *const __m256i);
        let block_last = if i + 31 + needle.len() <= haystack.len() {
            _mm256_loadu_si256(chunk[(needle.len() - 1)..].as_ptr() as *const __m256i)
        } else {
            let start = &haystack[(i + needle.len() - 1)..];
            block_pad[..start.len()].copy_from_slice(start);
            _mm256_loadu_si256(block_pad.as_ptr() as *const __m256i)
        };

        let eq_first = _mm256_cmpeq_epi8(first, block_first);
        let eq_last = _mm256_cmpeq_epi8(last, block_last);

        let mut mask = std::mem::transmute::<i32, u32>(_mm256_movemask_epi8(_mm256_and_si256(
            eq_first, eq_last,
        )));
        while mask != 0 {
            let bitpos = mask.trailing_zeros() as usize;
            let startpos = i + bitpos;
            if startpos + needle.len() <= haystack.len()
                && haystack[(startpos + 1)..(startpos + needle.len() - 1)]
                    == needle[1..needle.len() - 1]
            {
                return true;
            }
            mask = clear_leftmost_set(mask);
        }
    }

    let chunk = chunks.remainder();

    if needle.len() <= chunk.len() {
        let i = chunk.as_ptr() as usize - haystack.as_ptr() as usize;
        block_pad[..chunk.len()].copy_from_slice(chunk);
        let block_first = _mm256_loadu_si256(block_pad.as_ptr() as *const __m256i);
        let start = &haystack[(i + needle.len() - 1)..];
        block_pad[..start.len()].copy_from_slice(start);
        let block_last = _mm256_loadu_si256(block_pad.as_ptr() as *const __m256i);

        let eq_first = _mm256_cmpeq_epi8(first, block_first);
        let eq_last = _mm256_cmpeq_epi8(last, block_last);

        let mut mask = std::mem::transmute::<i32, u32>(_mm256_movemask_epi8(_mm256_and_si256(
            eq_first, eq_last,
        )));
        while mask != 0 {
            let bitpos = mask.trailing_zeros() as usize;
            let startpos = i + bitpos;
            if startpos + needle.len() <= haystack.len()
                && haystack[(startpos + 1)..(startpos + needle.len() - 1)]
                    == needle[1..needle.len() - 1]
            {
                return true;
            }
            mask = clear_leftmost_set(mask);
        }
    }

    false
}

#[cfg(target_feature = "avx2")]
#[inline(always)]
unsafe fn strstr_avx2_rust_fast_memcmp(
    haystack: &[u8],
    needle: &[u8],
    memcmp: unsafe fn(&[u8], &[u8]) -> bool,
) -> bool {
    let first = _mm256_set1_epi8(needle[0] as i8);
    let last = _mm256_set1_epi8(needle[needle.len() - 1] as i8);
    let mut block_first_pad: [u8; 32] = [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
    ];
    let mut block_last_pad: [u8; 32] = [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
    ];
    for chunk in haystack.chunks(32) {
        let i = chunk.as_ptr() as usize - haystack.as_ptr() as usize;
        let block_first = if chunk.len() == 32 {
            _mm256_loadu_si256(chunk.as_ptr() as *const __m256i)
        } else {
            block_first_pad[..chunk.len()].copy_from_slice(chunk);
            _mm256_loadu_si256(block_first_pad.as_ptr() as *const __m256i)
        };
        let block_last = if i + 31 + needle.len() <= haystack.len() {
            _mm256_loadu_si256(chunk[(needle.len() - 1)..].as_ptr() as *const __m256i)
        } else {
            let start = &haystack[(i + needle.len() - 1)..];
            block_last_pad[..start.len()].copy_from_slice(start);
            _mm256_loadu_si256(block_last_pad.as_ptr() as *const __m256i)
        };

        let eq_first = _mm256_cmpeq_epi8(first, block_first);
        let eq_last = _mm256_cmpeq_epi8(last, block_last);

        let mut mask = std::mem::transmute::<i32, u32>(_mm256_movemask_epi8(_mm256_and_si256(
            eq_first, eq_last,
        )));
        while mask != 0 {
            let bitpos = mask.trailing_zeros() as usize;
            let startpos = i + bitpos;
            if startpos + needle.len() <= haystack.len()
                && memcmp(
                    &haystack[(startpos + 1)..(startpos + needle.len() - 1)],
                    &needle[1..needle.len() - 1],
                )
            {
                return true;
            }
            mask = clear_leftmost_set(mask);
        }
    }

    false
}

#[cfg(target_feature = "avx2")]
pub unsafe fn strstr_avx2_rust_fast(haystack: &[u8], needle: &[u8]) -> bool {
    match needle.len() {
        0 => true,
        1 => memchr::memchr(needle[0], haystack).is_some(),
        2 => strstr_avx2_rust_fast_memcmp(haystack, needle, memcmp0),
        3 => strstr_avx2_rust_fast_memcmp(haystack, needle, memcmp1),
        4 => strstr_avx2_rust_fast_memcmp(haystack, needle, memcmp2),
        5 => strstr_avx2_rust_fast_memcmp(haystack, needle, memcmp4),
        6 => strstr_avx2_rust_fast_memcmp(haystack, needle, memcmp4),
        7 => strstr_avx2_rust_fast_memcmp(haystack, needle, memcmp5),
        8 => strstr_avx2_rust_fast_memcmp(haystack, needle, memcmp6),
        9 => strstr_avx2_rust_fast_memcmp(haystack, needle, memcmp8),
        10 => strstr_avx2_rust_fast_memcmp(haystack, needle, memcmp8),
        11 => strstr_avx2_rust_fast_memcmp(haystack, needle, memcmp9),
        12 => strstr_avx2_rust_fast_memcmp(haystack, needle, memcmp10),
        13 => strstr_avx2_rust_fast_memcmp(haystack, needle, memcmp11),
        14 => strstr_avx2_rust_fast_memcmp(haystack, needle, memcmp12),
        _ => strstr_avx2_rust_fast_memcmp(haystack, needle, memcmp),
    }
}

#[cfg(target_feature = "avx2")]
#[inline(always)]
unsafe fn strstr_avx2_rust_fast_2_memcmp(
    haystack: &[u8],
    needle: &[u8],
    memcmp: unsafe fn(&[u8], &[u8]) -> bool,
) -> bool {
    if haystack.len() < 32 {
        return strstr_rabin_karp(haystack, needle);
    }
    let first = _mm256_set1_epi8(needle[0] as i8);
    let last = _mm256_set1_epi8(needle[needle.len() - 1] as i8);
    let mut chunks = haystack[..=(haystack.len() - needle.len())].chunks_exact(32);
    while let Some(chunk) = chunks.next() {
        let i = chunk.as_ptr() as usize - haystack.as_ptr() as usize;
        let block_first = _mm256_loadu_si256(chunk.as_ptr() as *const __m256i);
        let block_last = _mm256_loadu_si256(chunk.as_ptr().add(needle.len() - 1) as *const __m256i);

        let eq_first = _mm256_cmpeq_epi8(first, block_first);
        let eq_last = _mm256_cmpeq_epi8(last, block_last);

        let mut mask = std::mem::transmute::<i32, u32>(_mm256_movemask_epi8(_mm256_and_si256(
            eq_first, eq_last,
        )));
        while mask != 0 {
            let bitpos = mask.trailing_zeros() as usize;
            let startpos = i + bitpos;
            if startpos + needle.len() <= haystack.len()
                && memcmp(
                    &haystack[(startpos + 1)..(startpos + needle.len() - 1)],
                    &needle[1..needle.len() - 1],
                )
            {
                return true;
            }
            mask = clear_leftmost_set(mask);
        }
    }

    let chunk = chunks.remainder();
    let i = chunk.as_ptr() as usize - haystack.as_ptr() as usize;
    let chunk = &haystack[i..];
    if needle.len() <= chunk.len() {
        strstr_rabin_karp(chunk, needle)
    } else {
        false
    }
}

#[inline(always)]
fn strstr_rabin_karp(haystack: &[u8], needle: &[u8]) -> bool {
    let mut needle_sum = 0;
    for c in needle {
        needle_sum += c;
    }

    let mut haystack_sum = 0;
    for c in &haystack[..needle.len() - 1] {
        haystack_sum += c;
    }

    let mut i = needle.len() - 1;
    while i < haystack.len() {
        haystack_sum += unsafe { haystack.get_unchecked(i) };
        i += 1;
        if haystack_sum == needle_sum && &haystack[(i - needle.len())..i] == needle {
            return true;
        }
        haystack_sum -= unsafe { haystack.get_unchecked(i - needle.len()) };
    }

    return false;
}

#[cfg(target_feature = "avx2")]
pub fn strstr_avx2_rust_fast_2(haystack: &[u8], needle: &[u8]) -> bool {
    match needle.len() {
        0 => true,
        1 => memchr::memchr(needle[0], haystack).is_some(),
        2 => unsafe { strstr_avx2_rust_fast_2_memcmp(haystack, needle, memcmp0) },
        3 => unsafe { strstr_avx2_rust_fast_2_memcmp(haystack, needle, memcmp1) },
        4 => unsafe { strstr_avx2_rust_fast_2_memcmp(haystack, needle, memcmp2) },
        5 => unsafe { strstr_avx2_rust_fast_2_memcmp(haystack, needle, memcmp4) },
        6 => unsafe { strstr_avx2_rust_fast_2_memcmp(haystack, needle, memcmp4) },
        7 => unsafe { strstr_avx2_rust_fast_2_memcmp(haystack, needle, memcmp5) },
        8 => unsafe { strstr_avx2_rust_fast_2_memcmp(haystack, needle, memcmp6) },
        9 => unsafe { strstr_avx2_rust_fast_2_memcmp(haystack, needle, memcmp8) },
        10 => unsafe { strstr_avx2_rust_fast_2_memcmp(haystack, needle, memcmp8) },
        11 => unsafe { strstr_avx2_rust_fast_2_memcmp(haystack, needle, memcmp9) },
        12 => unsafe { strstr_avx2_rust_fast_2_memcmp(haystack, needle, memcmp10) },
        13 => unsafe { strstr_avx2_rust_fast_2_memcmp(haystack, needle, memcmp11) },
        14 => unsafe { strstr_avx2_rust_fast_2_memcmp(haystack, needle, memcmp12) },
        _ => unsafe { strstr_avx2_rust_fast_2_memcmp(haystack, needle, memcmp) },
    }
}

#[cfg(target_feature = "avx2")]
pub struct StrStrAVX2Searcher {
    needle: Box<[u8]>,
    sse_first: __m128i,
    sse_last: __m128i,
    avx2_first: __m256i,
    avx2_last: __m256i,
    needle_sum: usize,
}

#[cfg(target_feature = "avx2")]
impl StrStrAVX2Searcher {
    pub fn new(needle: &[u8]) -> Self {
        let mut needle_sum = 0_usize;
        for &c in needle {
            needle_sum += c as usize;
        }
        StrStrAVX2Searcher {
            needle: needle.to_vec().into_boxed_slice(),
            sse_first: unsafe { _mm_set1_epi8(needle[0] as i8) },
            sse_last: unsafe { _mm_set1_epi8(needle[needle.len() - 1] as i8) },
            avx2_first: unsafe { _mm256_set1_epi8(needle[0] as i8) },
            avx2_last: unsafe { _mm256_set1_epi8(needle[needle.len() - 1] as i8) },
            needle_sum,
        }
    }

    pub fn search_in(&self, haystack: &[u8]) -> bool {
        if haystack.len() < self.needle.len() {
            return false;
        }
        match self.needle.len() {
            0 => true,
            1 => memchr::memchr(self.needle[0], haystack).is_some(),
            2 => unsafe { self.avx2_memcmp(haystack, memcmp0) },
            3 => unsafe { self.avx2_memcmp(haystack, memcmp1) },
            4 => unsafe { self.avx2_memcmp(haystack, memcmp2) },
            5 => unsafe { self.avx2_memcmp(haystack, memcmp4) },
            6 => unsafe { self.avx2_memcmp(haystack, memcmp4) },
            7 => unsafe { self.avx2_memcmp(haystack, memcmp5) },
            8 => unsafe { self.avx2_memcmp(haystack, memcmp6) },
            9 => unsafe { self.avx2_memcmp(haystack, memcmp8) },
            10 => unsafe { self.avx2_memcmp(haystack, memcmp8) },
            11 => unsafe { self.avx2_memcmp(haystack, memcmp9) },
            12 => unsafe { self.avx2_memcmp(haystack, memcmp10) },
            13 => unsafe { self.avx2_memcmp(haystack, memcmp11) },
            14 => unsafe { self.avx2_memcmp(haystack, memcmp12) },
            _ => unsafe { self.avx2_memcmp(haystack, memcmp) },
        }
    }

    #[inline(always)]
    unsafe fn sse_memcmp(&self, haystack: &[u8], memcmp: unsafe fn(&[u8], &[u8]) -> bool) -> bool {
        if haystack.len() < 16 {
            return self.rabin_karp(haystack);
        }
        let mut chunks = haystack[..=(haystack.len() - self.needle.len())].chunks_exact(16);
        while let Some(chunk) = chunks.next() {
            let i = chunk.as_ptr() as usize - haystack.as_ptr() as usize;
            let block_first = _mm_loadu_si128(chunk.as_ptr() as *const __m128i);
            let block_last =
                _mm_loadu_si128(chunk.as_ptr().add(self.needle.len() - 1) as *const __m128i);

            let eq_first = _mm_cmpeq_epi8(self.sse_first, block_first);
            let eq_last = _mm_cmpeq_epi8(self.sse_last, block_last);

            let mut mask = std::mem::transmute::<i32, u32>(_mm_movemask_epi8(_mm_and_si128(
                eq_first, eq_last,
            )));
            while mask != 0 {
                let bitpos = mask.trailing_zeros() as usize;
                let startpos = i + bitpos;
                if startpos + self.needle.len() <= haystack.len()
                    && memcmp(
                        &haystack[(startpos + 1)..(startpos + self.needle.len() - 1)],
                        &self.needle[1..self.needle.len() - 1],
                    )
                {
                    return true;
                }
                mask = clear_leftmost_set(mask);
            }
        }

        let chunk = chunks.remainder();
        let i = chunk.as_ptr() as usize - haystack.as_ptr() as usize;
        let chunk = &haystack[i..];
        if !chunk.is_empty() {
            self.rabin_karp(chunk)
        } else {
            false
        }
    }

    #[inline(always)]
    unsafe fn avx2_memcmp(&self, haystack: &[u8], memcmp: unsafe fn(&[u8], &[u8]) -> bool) -> bool {
        if haystack.len() < 32 {
            return self.sse_memcmp(haystack, memcmp);
        }
        let mut chunks = haystack[..=(haystack.len() - self.needle.len())].chunks_exact(32);
        while let Some(chunk) = chunks.next() {
            let i = chunk.as_ptr() as usize - haystack.as_ptr() as usize;
            let block_first = _mm256_loadu_si256(chunk.as_ptr() as *const __m256i);
            let block_last =
                _mm256_loadu_si256(chunk.as_ptr().add(self.needle.len() - 1) as *const __m256i);

            let eq_first = _mm256_cmpeq_epi8(self.avx2_first, block_first);
            let eq_last = _mm256_cmpeq_epi8(self.avx2_last, block_last);

            let mut mask = std::mem::transmute::<i32, u32>(_mm256_movemask_epi8(_mm256_and_si256(
                eq_first, eq_last,
            )));
            while mask != 0 {
                let bitpos = mask.trailing_zeros() as usize;
                let startpos = i + bitpos;
                if startpos + self.needle.len() <= haystack.len()
                    && memcmp(
                        &haystack[(startpos + 1)..(startpos + self.needle.len() - 1)],
                        &self.needle[1..self.needle.len() - 1],
                    )
                {
                    return true;
                }
                mask = clear_leftmost_set(mask);
            }
        }

        let chunk = chunks.remainder();
        let i = chunk.as_ptr() as usize - haystack.as_ptr() as usize;
        let chunk = &haystack[i..];
        if !chunk.is_empty() {
            self.sse_memcmp(chunk, memcmp)
        } else {
            false
        }
    }

    #[inline(always)]
    unsafe fn rabin_karp(&self, haystack: &[u8]) -> bool {
        let mut haystack_sum = 0_usize;
        for &c in &haystack[..self.needle.len() - 1] {
            haystack_sum += c as usize;
        }

        let mut i = self.needle.len() - 1;
        while i < haystack.len() {
            haystack_sum += *haystack.get_unchecked(i) as usize;
            i += 1;
            if haystack_sum == self.needle_sum
                && &haystack[(i - self.needle.len())..i] == &*self.needle
            {
                return true;
            }
            haystack_sum -= *haystack.get_unchecked(i - self.needle.len()) as usize;
        }

        return false;
    }
}

#[repr(align(32))]
struct AlignedVector([u8; 32]);

#[cfg(target_feature = "avx2")]
pub unsafe fn strstr_avx2_rust_aligned(haystack: &[u8], needle: &[u8]) -> bool {
    let first = _mm256_set1_epi8(needle[0] as i8);
    let last = _mm256_set1_epi8(needle[needle.len() - 1] as i8);
    let mut block_pad = AlignedVector([
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
    ]);
    for chunk in haystack.chunks(32) {
        let i = chunk.as_ptr() as usize - haystack.as_ptr() as usize;
        block_pad.0[..chunk.len()].copy_from_slice(chunk);
        let block_first = _mm256_load_si256(block_pad.0.as_ptr() as *const __m256i);
        let start = if i + 31 + needle.len() <= haystack.len() {
            &haystack[(i + needle.len() - 1)..(i + 32 + needle.len() - 1)]
        } else {
            &haystack[(i + needle.len() - 1)..]
        };
        block_pad.0[..start.len()].copy_from_slice(start);
        let block_last = _mm256_load_si256(block_pad.0.as_ptr() as *const __m256i);

        let eq_first = _mm256_cmpeq_epi8(first, block_first);
        let eq_last = _mm256_cmpeq_epi8(last, block_last);

        let mut mask = std::mem::transmute::<i32, u32>(_mm256_movemask_epi8(_mm256_and_si256(
            eq_first, eq_last,
        )));
        while mask != 0 {
            let bitpos = mask.trailing_zeros() as usize;
            let startpos = i + bitpos;
            if startpos + needle.len() <= haystack.len()
                && haystack[(startpos + 1)..(startpos + needle.len() - 1)]
                    == needle[1..needle.len() - 1]
            {
                return true;
            }
            mask = clear_leftmost_set(mask);
        }
    }

    false
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_needle_length_3() {
        use super::strstr_avx2_rust_fast_2;

        let mut input = [0; 32];

        for i in 0..=(input.len() - 3) {
            input = [b'A'; 32];
            input[i..(i + 3)].copy_from_slice(&[b'B'; 3]);
            assert_eq!(
                strstr_avx2_rust_fast_2(&input[..], b"BBB"),
                true,
                "{:?} should contain {:?}",
                &input[..],
                b"BBB"
            );
        }

        let mut input = [0; 63];

        for i in 0..=(input.len() - 3) {
            input = [b'A'; 63];
            input[i..(i + 3)].copy_from_slice(&[b'B'; 3]);
            assert_eq!(
                strstr_avx2_rust_fast_2(&input[..], b"BBB"),
                true,
                "{:?} should contain {:?}",
                &input[..],
                b"BBB"
            );
        }
    }
}