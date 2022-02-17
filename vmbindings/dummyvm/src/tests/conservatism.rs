pub use crate::api::*;
pub use mmtk::util::opaque_pointer::*;
pub use mmtk::util::*;
pub use mmtk::AllocationSemantics;

#[allow(dead_code)]
pub struct Fixture {
    addr: Address,
    objref: ObjectReference,
}

static mut FIXTURE: *mut Fixture = std::ptr::null_mut();
static INIT: std::sync::Once = std::sync::Once::new();

pub fn with_fixture<F: Fn(&Fixture)>(func: F) {
    INIT.call_once(|| {
        const MB: usize = 1024 * 1024;
        // 1MB heap
        mmtk_gc_init(MB);
        mmtk_initialize_collection(VMThread::UNINITIALIZED);
        // Make sure GC does not run during test.
        mmtk_disable_collection();
        let handle = mmtk_bind_mutator(VMMutatorThread(VMThread::UNINITIALIZED));
    
        let size = 8;
        let semantics = AllocationSemantics::Default;
    
        // A relatively small object, typical for Ruby.
        let addr = mmtk_alloc(handle, size, 8, 0, semantics);
        assert!(!addr.is_zero());
    
        let objref = unsafe { addr.to_object_reference() };
        mmtk_post_alloc(handle, objref, size, semantics);
    
        unsafe {
            FIXTURE = Box::into_raw(Box::new(Fixture {
                addr,
                objref,
            }));
        }
    });

    func(unsafe { &mut *FIXTURE });
}

#[cfg(feature = "global_alloc_bit")]
mod global_alloc_bit {
    use super::*;

    use mmtk::memory_manager::is_alloced as tested_function;
  
    #[test]
    pub fn null() {
        with_fixture(|_fixture| {
            assert!(!tested_function(unsafe { Address::ZERO.to_object_reference() }));
        }); 
    }  

    #[test]
    pub fn max() {
        with_fixture(|_fixture| {
            assert!(!tested_function(unsafe { Address::MAX.to_object_reference() }));
        }); 
    }
    
    #[test]
    pub fn direct_hit() {
        with_fixture(|fixture| {
            assert!(tested_function(fixture.objref));
        }); 
    }
        
    #[test]
    pub fn small_offset_aligned() {
        with_fixture(|fixture| {
            assert!(!tested_function(unsafe {
                fixture.addr.add(8).to_object_reference()
            }));
        }); 
    }
        
    #[test]
    pub fn small_offset_unaligned() {
        with_fixture(|fixture| {
            assert!(!tested_function(unsafe {
                fixture.addr.add(1).to_object_reference()
            }));
        }); 
    }
        
    #[test]
    pub fn medium_offset_aligned_4k() {
        with_fixture(|fixture| {
            assert!(!tested_function(unsafe {
                fixture.addr.add(4 * 1024).to_object_reference()
            }));
        }); 
    }
        
    #[test]
    pub fn medium_offset_aligned_64k() {
        with_fixture(|fixture| {
            assert!(!tested_function(unsafe {
                fixture.addr.add(64 * 1024).to_object_reference()
            }));
        }); 
    }
        
    #[test]
    pub fn medium_offset_aligned_1m() {
        with_fixture(|fixture| {
            assert!(!tested_function(unsafe {
                fixture.addr.add(1024 * 1024).to_object_reference()
            }));
        }); 
    }
        
    #[test]
    pub fn medium_offset_aligned_32m() {
        with_fixture(|fixture| {
            assert!(!tested_function(unsafe {
                fixture.addr.add(32 * 1024 * 1024).to_object_reference()
            }));
        }); 
    }
        
    #[test]
    pub fn medium_offset_aligned_1g() {
        with_fixture(|fixture| {
            assert!(!tested_function(unsafe {
                fixture.addr.add(1024 * 1024 * 1024).to_object_reference()
            }));
        }); 
    }
        
    #[test]
    pub fn medium_offset_aligned_32g() {
        with_fixture(|fixture| {
            assert!(!tested_function(unsafe {
                fixture.addr.add(32 * 1024 * 1024 * 1024).to_object_reference()
            }));
        }); 
    }
}

mod is_mapped_object {
    use super::*;

    use mmtk::memory_manager::is_mapped_object as tested_function;

  
    #[test]
    pub fn null() {
        with_fixture(|_fixture| {
            assert!(!tested_function(unsafe { Address::ZERO.to_object_reference() }));
        }); 
    }  

    #[test]
    pub fn max() {
        with_fixture(|_fixture| {
            assert!(!tested_function(unsafe { Address::MAX.to_object_reference() }));
        }); 
    }
    
    #[test]
    pub fn direct_hit() {
        with_fixture(|fixture| {
            assert!(tested_function(fixture.objref));
        }); 
    }
        
    #[test]
    pub fn small_offset_aligned() {
        with_fixture(|fixture| {
            assert!(!tested_function(unsafe {
                fixture.addr.add(8).to_object_reference()
            }));
        }); 
    }
        
    #[test]
    pub fn small_offset_unaligned() {
        with_fixture(|fixture| {
            assert!(!tested_function(unsafe {
                fixture.addr.add(1).to_object_reference()
            }));
        }); 
    }
        
    #[test]
    pub fn medium_offset_aligned_4k() {
        with_fixture(|fixture| {
            assert!(!tested_function(unsafe {
                fixture.addr.add(4 * 1024).to_object_reference()
            }));
        }); 
    }
        
    #[test]
    pub fn medium_offset_aligned_64k() {
        with_fixture(|fixture| {
            assert!(!tested_function(unsafe {
                fixture.addr.add(64 * 1024).to_object_reference()
            }));
        }); 
    }
        
    #[test]
    pub fn medium_offset_aligned_1m() {
        with_fixture(|fixture| {
            assert!(!tested_function(unsafe {
                fixture.addr.add(1024 * 1024).to_object_reference()
            }));
        }); 
    }
        
    #[test]
    pub fn medium_offset_aligned_32m() {
        with_fixture(|fixture| {
            assert!(!tested_function(unsafe {
                fixture.addr.add(32 * 1024 * 1024).to_object_reference()
            }));
        }); 
    }
        
    #[test]
    pub fn medium_offset_aligned_1g() {
        with_fixture(|fixture| {
            assert!(!tested_function(unsafe {
                fixture.addr.add(1024 * 1024 * 1024).to_object_reference()
            }));
        }); 
    }
        
    #[test]
    pub fn medium_offset_aligned_32g() {
        with_fixture(|fixture| {
            assert!(!tested_function(unsafe {
                fixture.addr.add(32 * 1024 * 1024 * 1024).to_object_reference()
            }));
        }); 
    }
}
