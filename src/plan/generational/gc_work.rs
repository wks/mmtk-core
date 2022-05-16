use crate::plan::generational::global::Gen;
use crate::policy::space::{Space, TraceObjectResult};
use crate::scheduler::gc_work::*;
use crate::util::{Address, ObjectReference};
use crate::vm::*;
use crate::MMTK;
use std::ops::{Deref, DerefMut};

/// Process edges for a nursery GC. This type is provided if a generational plan does not use
/// [`crate::scheduler::gc_work::SFTProcessEdges`]. If a plan uses `SFTProcessEdges`,
/// it does not need to use this type.
pub struct GenNurseryProcessEdges<VM: VMBinding> {
    gen: &'static Gen<VM>,
    base: ProcessEdgesBase<VM>,
}

impl<VM: VMBinding> ProcessEdgesWork for GenNurseryProcessEdges<VM> {
    type VM = VM;

    fn new(edges: Vec<Address>, roots: bool, mmtk: &'static MMTK<VM>) -> Self {
        let base = ProcessEdgesBase::new(edges, roots, mmtk);
        let gen = base.plan().generational();
        Self { gen, base }
    }
    #[inline]
    fn trace_object(&mut self, object: ObjectReference) -> TraceObjectResult {
        if object.is_null() {
            return TraceObjectResult::not_forwarded(object);
        }
        self.gen.trace_object_nursery(self, object, self.worker())
    }
    #[inline]
    fn process_edge(&mut self, slot: Address) {
        let object = unsafe { slot.load::<ObjectReference>() };
        #[cfg(not(feature = "trace_object_result"))]
        {
            let new_object = self.trace_object(object);
            debug_assert!(!self.gen.nursery.in_space(new_object));
            unsafe { slot.store(new_object) };
        }
        #[cfg(feature = "trace_object_result")]
        {
            let TraceObjectResult { forwarded_ref } = self.trace_object(object);
            if let Some(new_object) = forwarded_ref {
                debug_assert!(!self.gen.nursery.in_space(new_object));
                unsafe { slot.store(new_object) };
            }
        }
    }
}

impl<VM: VMBinding> Deref for GenNurseryProcessEdges<VM> {
    type Target = ProcessEdgesBase<VM>;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl<VM: VMBinding> DerefMut for GenNurseryProcessEdges<VM> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
