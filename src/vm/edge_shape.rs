use atomic::Atomic;

use crate::util::{Address, ObjectReference};

/// An abstract edge.  An edge holds an object reference.  When we load from it, we get an
/// ObjectReference; we can also store an ObjectReference into it.
///
/// This intends to abstract out the differences of reference field representation among different
/// VMs.  If the VM represent a reference field as a word that holds the pointer to the object, it
/// can use the default `SimpleEdge` we provide.  In some cases, the VM need to implement its own
/// `Edge` instances:
///
/// For example:
/// -   The VM uses compressed pointer (Compressed OOP in OpenJDK's terminology), where the heap
///     size is limited, and a 64-bit pointer is stored in a 32-bit slot.
/// -   The VM uses tagged pointer, where some bits of a word are used as metadata while the rest
///     are used as pointer.
/// -   The VM does not expose the offset of object fields, but calls a function for each reference
///     field **value**, and assigns the returned value back to the field.
///
/// Note: this trait only concerns the representation (i.e. the shape) of the edge, not its
/// semantics, such as whether it holds strong or weak references.  If a VM holds a weak reference
/// in a word as a pointer, it can also use `SimpleEdge` for weak reference fields.
pub trait Edge: Copy + Send {
    /// Load object reference from the edge.
    fn load(&self) -> ObjectReference;

    /// Store the object reference `object` into the edge.
    fn store(&self, object: ObjectReference);

    /// Prefetch the edge so that a subsequent `load` will be faster.
    fn prefetch_load(&self) {
        // no-op by default
    }

    /// Prefetch the edge so that a subsequent `store` will be faster.
    fn prefetch_store(&self) {
        // no-op by default
    }
}

/// A simple edge implementation that represents a word-sized slot where an ObjectReference value
/// is stored as is.  It is the default edge type, and should be suitable for most VMs.
#[derive(Clone, Copy)]
pub struct SimpleEdge {
    slot_addr: *mut Atomic<ObjectReference>,
}

impl SimpleEdge {
    /// Create a simple edge from an address.
    ///
    /// Arguments:
    /// *   `address`: The address in memory where an `ObjectReference` is stored.
    pub fn from_address(address: Address) -> Self {
        Self {
            slot_addr: address.to_mut_ptr(),
        }
    }

    /// Get the address of the edge.
    ///
    /// Return the address at which the `ObjectReference` is stored.
    pub fn as_address(&self) -> Address {
        Address::from_mut_ptr(self.slot_addr)
    }
}

unsafe impl Send for SimpleEdge {}

impl Edge for SimpleEdge {
    fn load(&self) -> ObjectReference {
        unsafe { (*self.slot_addr).load(atomic::Ordering::Relaxed) }
    }

    fn store(&self, object: ObjectReference) {
        unsafe { (*self.slot_addr).store(object, atomic::Ordering::Relaxed) }
    }
}

/// For backword compatibility.
/// We let Address implement Edge so existing bindings that use `Address` to represent an edge can
/// continue to work.
impl Edge for Address {
    fn load(&self) -> ObjectReference {
        unsafe { Address::load(*self) }
    }

    fn store(&self, object: ObjectReference) {
        unsafe { Address::store(*self, object) }
    }
}
