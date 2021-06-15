use std::fmt;
use std::io::Result;

use crate::util::constants::BYTES_IN_PAGE;
use crate::util::heap::layout::vm_layout_constants::BYTES_IN_CHUNK;
use crate::util::metadata::side_metadata::*;
use crate::util::Address;

/// This struct stores the specification of a side metadata bit-set.
/// It is used as an input to the (inline) functions provided by the side metadata module.
///
/// Each plan or policy which uses a metadata bit-set, needs to create an instance of this struct.
///
/// For performance reasons, objects of this struct should be constants.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct MetadataSpec {
    /// Shows whether the side metadata is on side (`true`) or in object header (`false`).
    pub is_side_metadata: bool,
    /// Shows whether the metadata is global, or policy-specific
    pub is_global: bool,
    /// A multi-purpose field:
    ///  - For contiguous side metadata, this field represents the absolute starting address.
    ///  - For chunked side metadata, this field represents the offset (in bytes) from the start of the metadata chunk.
    ///  - For in-header metadata, this is the offset (in bits) from the object reference.
    pub offset: isize,
    /// The number of bits included in this metadata.
    /// For side metadata, this must be a power of two (2^n)
    pub num_of_bits: usize,
    /// Log2 of the minimum object size
    pub log_min_obj_size: usize,
}

impl fmt::Debug for MetadataSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "MetadataSpec {{ \
            **side?: {}, \
            **global: {} \
            **offset: 0x{:x} \
            **num_of_bits: 0x{:x} \
            **log_min_obj_size: 0x{:x} \
            }}",
            self.is_side_metadata,
            self.is_global,
            self.offset,
            self.num_of_bits,
            self.log_min_obj_size
        ))
    }
}

/// This struct stores all the side metadata specs for a policy. Generally a policy needs to know its own
/// side metadata spec as well as the plan's specs.
pub struct MetadataContext {
    // For plans
    pub global: Vec<MetadataSpec>,
    // For policies
    pub local: Vec<MetadataSpec>,
}

impl MetadataContext {
    pub fn new_global_specs(specs: &[MetadataSpec]) -> Vec<MetadataSpec> {
        let mut ret = vec![];
        ret.extend_from_slice(specs);
        // if cfg!(feature = "side_gc_header") {
        //     ret.push(crate::util::gc_byte::SIDE_GC_BYTE_SPEC);
        // }
        ret
    }
}

pub struct SideMetadata {
    context: MetadataContext,
}

impl SideMetadata {
    pub fn new(context: MetadataContext) -> SideMetadata {
        Self { context }
    }

    pub fn get_context(&self) -> &MetadataContext {
        &self.context
    }

    pub fn get_local_specs(&self) -> &[MetadataSpec] {
        &self.context.local
    }

    /// Return the pages reserved for side metadata based on the data pages we used.
    // We used to use PageAccouting to count pages used in side metadata. However,
    // that means we always count pages while we may reserve less than a page each time.
    // This could lead to overcount. I think the easier way is to not account
    // when we allocate for sidemetadata, but to calculate the side metadata usage based on
    // how many data pages we use when reporting.
    pub fn calculate_reserved_pages(&self, data_pages: usize) -> usize {
        let mut total = 0;
        for spec in self.context.global.iter() {
            let rshift = addr_rshift(spec);
            total += (data_pages + ((1 << rshift) - 1)) >> rshift;
        }
        for spec in self.context.local.iter() {
            let rshift = addr_rshift(spec);
            total += (data_pages + ((1 << rshift) - 1)) >> rshift;
        }
        total
    }

    pub fn reset(&self) {}

    // ** NOTE: **
    //  Regardless of the number of bits in a metadata unit, we always represent its content as a word.

    /// Tries to map the required metadata space and returns `true` is successful.
    /// This can be called at page granularity.
    pub fn try_map_metadata_space(&self, start: Address, size: usize) -> Result<()> {
        debug!(
            "try_map_metadata_space({}, 0x{:x}, {}, {})",
            start,
            size,
            self.context.global.len(),
            self.context.local.len()
        );
        // Page aligned
        debug_assert!(start.is_aligned_to(BYTES_IN_PAGE));
        debug_assert!(size % BYTES_IN_PAGE == 0);
        self.map_metadata_internal(start, size, false)
    }

    /// Tries to map the required metadata address range, without reserving swap-space/physical memory for it.
    /// This will make sure the address range is exclusive to the caller. This should be called at chunk granularity.
    ///
    /// NOTE: Accessing addresses in this range will produce a segmentation fault if swap-space is not mapped using the `try_map_metadata_space` function.
    pub fn try_map_metadata_address_range(&self, start: Address, size: usize) -> Result<()> {
        debug!(
            "try_map_metadata_address_range({}, 0x{:x}, {}, {})",
            start,
            size,
            self.context.global.len(),
            self.context.local.len()
        );
        // Chunk aligned
        debug_assert!(start.is_aligned_to(BYTES_IN_CHUNK));
        debug_assert!(size % BYTES_IN_CHUNK == 0);
        self.map_metadata_internal(start, size, true)
    }

    /// The internal function to mmap metadata
    ///
    /// # Arguments
    /// * `start` - The starting address of the source data.
    /// * `size` - The size of the source data (in bytes).
    /// * `no_reserve` - whether to invoke mmap with a noreserve flag (we use this flag to quanrantine address range)
    fn map_metadata_internal(&self, start: Address, size: usize, no_reserve: bool) -> Result<()> {
        for spec in self.context.global.iter() {
            match try_mmap_contiguous_metadata_space(start, size, spec, no_reserve) {
                Ok(_) => {}
                Err(e) => return Result::Err(e),
            }
        }

        #[cfg(target_pointer_width = "32")]
        let mut lsize: usize = 0;

        for spec in self.context.local.iter() {
            // For local side metadata, we always have to reserve address space for all
            // local metadata required by all policies in MMTk to be able to calculate a constant offset for each local metadata at compile-time
            // (it's like assigning an ID to each policy).
            // As the plan is chosen at run-time, we will never know which subset of policies will be used during run-time.
            // We can't afford this much address space in 32-bits.
            // So, we switch to the chunk-based approach for this specific case.
            //
            // The global metadata is different in that for each plan, we can calculate its constant base addresses at compile-time.
            // Using the chunk-based approach will need the same address space size as the current not-chunked approach.
            #[cfg(target_pointer_width = "64")]
            {
                match try_mmap_contiguous_metadata_space(start, size, spec, no_reserve) {
                    Ok(_) => {}
                    Err(e) => return Result::Err(e),
                }
            }
            #[cfg(target_pointer_width = "32")]
            {
                lsize += metadata_bytes_per_chunk(spec.log_min_obj_size, spec.num_of_bits);
            }
        }

        #[cfg(target_pointer_width = "32")]
        if lsize > 0 {
            let max = BYTES_IN_CHUNK >> LOG_LOCAL_SIDE_METADATA_WORST_CASE_RATIO;
            debug_assert!(
                lsize <= max,
                "local side metadata per chunk (0x{:x}) must be less than (0x{:x})",
                lsize,
                max
            );
            match try_map_per_chunk_metadata_space(start, size, lsize, no_reserve) {
                Ok(_) => {}
                Err(e) => return Result::Err(e),
            }
        }

        Ok(())
    }

    /// Unmap the corresponding metadata space or panic.
    ///
    /// Note-1: This function is only used for test and debug right now.
    ///
    /// Note-2: This function uses munmap() which works at page granularity.
    ///     If the corresponding metadata space's size is not a multiple of page size,
    ///     the actual unmapped space will be bigger than what you specify.
    #[cfg(test)]
    pub fn ensure_unmap_metadata_space(&self, start: Address, size: usize) {
        trace!("ensure_unmap_metadata_space({}, 0x{:x})", start, size);
        debug_assert!(start.is_aligned_to(BYTES_IN_PAGE));
        debug_assert!(size % BYTES_IN_PAGE == 0);

        for spec in self.context.global.iter() {
            ensure_munmap_contiguos_metadata_space(start, size, spec);
        }

        for spec in self.context.local.iter() {
            #[cfg(target_pointer_width = "64")]
            {
                ensure_munmap_contiguos_metadata_space(start, size, spec);
            }
            #[cfg(target_pointer_width = "32")]
            {
                ensure_munmap_chunked_metadata_space(start, size, spec);
            }
        }
    }
}
