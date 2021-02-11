use crate::util::{constants, conversions, side_metadata::{ensure_metadata_chunk_is_mmaped, load_atomic, meta_bytes_per_chunk, store_atomic, try_mmap_metadata_chunk}};
use crate::util::side_metadata::SideMetadataSpec;
use crate::util::side_metadata::SideMetadataScope;
use crate::util::Address;
use crate::util::ObjectReference;
use conversions::chunk_align_down;
use std::{collections::HashSet, sync::Mutex};
use std::sync::RwLock;

lazy_static! {
    pub static ref ACTIVE_CHUNKS: RwLock<HashSet<Address>> = RwLock::default();
}

pub const ALLOC_METADATA_SPEC: SideMetadataSpec = SideMetadataSpec { 
    scope: SideMetadataScope::PolicySpecific,
    offset: 0,
    log_num_of_bits: 0, 
    log_min_obj_size: constants::LOG_BYTES_IN_WORD as usize };
    
pub const MARKING_METADATA_SPEC: SideMetadataSpec = SideMetadataSpec { 
    scope: SideMetadataScope::PolicySpecific, 
    offset: ALLOC_METADATA_SPEC.offset + meta_bytes_per_chunk(ALLOC_METADATA_SPEC.log_min_obj_size, ALLOC_METADATA_SPEC.log_num_of_bits), 
    log_num_of_bits: 0, 
    log_min_obj_size: constants::LOG_BYTES_IN_WORD as usize };

pub fn meta_space_mapped(address: Address) -> bool {
    let chunk_start = chunk_align_down(address);
    ACTIVE_CHUNKS.read().unwrap().contains(&chunk_start)
}

pub unsafe fn map_meta_space_for_chunk(chunk_start: Address) {
    let mut active_chunks = ACTIVE_CHUNKS.write().unwrap();
    if active_chunks.contains(&chunk_start) {
        return
    }
    active_chunks.insert(chunk_start);
    try_mmap_metadata_chunk(chunk_start,
        0,
        meta_bytes_per_chunk(ALLOC_METADATA_SPEC.log_min_obj_size, ALLOC_METADATA_SPEC.log_num_of_bits) + meta_bytes_per_chunk(MARKING_METADATA_SPEC.log_min_obj_size, MARKING_METADATA_SPEC.log_num_of_bits));
}

// Check if a given object was allocated by malloc
pub fn is_malloced(object: ObjectReference) -> bool {
    let address = object.to_address();
    unsafe {
        meta_space_mapped(address)
            && load_atomic(ALLOC_METADATA_SPEC, address) == 1
    }
}

// check if a given object is marked
pub fn is_marked(object: ObjectReference) -> bool {
    let address = object.to_address();
    debug_assert!(meta_space_mapped(address));
    unsafe { load_atomic(MARKING_METADATA_SPEC, address) == 1 } 
}

pub fn set_alloc_bit(address: Address) {
    debug_assert!(meta_space_mapped(address));
    ensure_metadata_chunk_is_mmaped(ALLOC_METADATA_SPEC, address);
    unsafe {
        store_atomic(ALLOC_METADATA_SPEC, address, 1);
    }
}

pub fn set_mark_bit(address: Address) {
    debug_assert!(meta_space_mapped(address));
    unsafe {
        store_atomic(MARKING_METADATA_SPEC, address, 1);
    }
}

pub fn unset_alloc_bit(address: Address) {
    debug_assert!(meta_space_mapped(address));
    ensure_metadata_chunk_is_mmaped(ALLOC_METADATA_SPEC, address);
    unsafe {
        store_atomic(ALLOC_METADATA_SPEC, address, 0);
    }
}

pub fn unset_mark_bit(address: Address) {
    debug_assert!(meta_space_mapped(address));
    unsafe {
        store_atomic(MARKING_METADATA_SPEC, address, 0);
    }
}