//! Data types for visiting metadata ranges at different granularities

use crate::util::Address;

/// The type for bit offset in a byte, word or a SIMD vector.
///
/// We use usize because it is generic and we may use AVX-512 some day, where u8 (256 max) is not
/// big enough.
pub type BitOffset = usize;

/// A range of bytes or bits within a byte.  It is the unit of visiting a contiguous bit range of a
/// side metadata.
///
/// In general, a bit range of a bitmap starts with multiple bits in the byte, followed by many
/// whole bytes, and ends with multiple bits in the last byte.
///
/// A range is never empty.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitByteRange {
    /// A range of whole bytes.
    Bytes {
        /// The starting address (inclusive) of the bytes.
        start: Address,
        /// The ending address (exclusive) of the bytes.
        end: Address,
    },
    /// A range of bits within a byte.
    BitsInByte {
        /// The address of the byte.
        addr: Address,
        /// The starting bit index (inclusive), starting with zero from the low-order bit.
        bit_start: BitOffset,
        /// The ending bit index (exclusive),  starting with zero from the low-order bit.  This may
        /// be 8 which means the range includes the highest bit.  Be careful when shifting a `u8`
        /// value because shifting an `u8` by 8 is considered an overflow in Rust.
        bit_end: BitOffset,
    },
}

/// Iterate over a range of bits in a bitmap.
///
/// This method is primarily used for iterating side metadata for a data address range. As we cannot
/// guarantee that the data address range can be mapped to whole metadata bytes, we have to deal
/// with visiting only a bit range in a metadata byte.
///
/// The bit range starts at the bit at index `bit_start` in the byte at address `byte_start`, and
/// ends at (but does not include) the bit at index `bit_end` in the byte at address `byte_end`.
///
/// Arguments:
/// * `forwards`: If true, we iterate forwards (from start/low address to end/high address).
///               Otherwise, we iterate backwards (from end/high address to start/low address).
/// * `visitor`: The callback that visits ranges of bits or bytes.  It returns whether the itertion
///   is early terminated.
///
/// Returns true if we iterate through every bits in the range. Return false if we abort iteration
/// early.
pub fn iterate_meta_bits<V>(
    byte_start: Address,
    bit_start: u8,
    byte_end: Address,
    bit_end: u8,
    forwards: bool,
    visitor: &mut V,
) -> bool
where
    V: FnMut(BitByteRange) -> bool,
{
    trace!(
        "iterate_meta_bits: {} {}, {} {}",
        byte_start,
        bit_start,
        byte_end,
        bit_end
    );

    // Start/end is the same, we don't need to do anything.
    if byte_start == byte_end && bit_start == bit_end {
        return false;
    }

    // visit whole bytes
    if bit_start == 0 && bit_end == 0 {
        return visitor(BitByteRange::Bytes {
            start: byte_start,
            end: byte_end,
        });
    }

    if byte_start == byte_end {
        // Visit bits in the same byte between start and end bit
        return visitor(BitByteRange::BitsInByte {
            addr: byte_start,
            bit_start: bit_start as usize,
            bit_end: bit_end as usize,
        });
    } else if byte_start + 1usize == byte_end && bit_end == 0 {
        // Visit bits in the same byte after the start bit (between start bit and 8)
        return visitor(BitByteRange::BitsInByte {
            addr: byte_start,
            bit_start: bit_start as usize,
            bit_end: 8usize,
        });
    } else {
        // We cannot let multiple closures capture `visitor` mutably at the same time, so we
        // pass the visitor in as `v` every time.

        // update bits in the first byte
        let visit_start = |v: &mut V| {
            v(BitByteRange::BitsInByte {
                addr: byte_start,
                bit_start: bit_start as usize,
                bit_end: 8usize,
            })
        };

        // update bytes in the middle
        let visit_middle = |v: &mut V| {
            let start = byte_start + 1usize;
            let end = byte_end;
            if start < end {
                // non-empty middle range
                v(BitByteRange::Bytes { start, end })
            } else {
                // empty middle range
                false
            }
        };

        // update bits in the last byte
        let visit_end = |v: &mut V| {
            v(BitByteRange::BitsInByte {
                addr: byte_end,
                bit_start: 0 as usize,
                bit_end: bit_end as usize,
            })
        };

        // Update each segments.
        if forwards {
            visit_start(visitor) || visit_middle(visitor) || visit_end(visitor)
        } else {
            visit_end(visitor) || visit_middle(visitor) || visit_start(visitor)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ByteWordRange {
    Words { start: Address, end: Address },
    BytesInWord { start: Address, end: Address },
}
