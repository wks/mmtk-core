use super::global::TripleSpace;
use crate::plan::CopyContext;
use crate::plan::PlanConstraints;
use crate::policy::space::Space;
use crate::policy::copyspace::CopySpace;
use crate::scheduler::gc_work::*;
use crate::scheduler::WorkerLocal;
use crate::util::alloc::{Allocator, BumpAllocator};
use crate::util::object_forwarding;
use crate::util::opaque_pointer::*;
use crate::util::{Address, ObjectReference};
use crate::vm::VMBinding;
use crate::MMTK;
use std::ops::{Deref, DerefMut};

pub struct TSCopyContext<VM: VMBinding> {
    plan: &'static TripleSpace<VM>,
    bp: BumpAllocator<VM>,
}

impl<VM: VMBinding> CopyContext for TSCopyContext<VM> {
    type VM = VM;

    fn constraints(&self) -> &'static PlanConstraints {
        &super::global::TS_CONSTRAINTS
    }
    fn init(&mut self, tls: VMWorkerThread) {
        self.bp.tls = tls.0;
    }
    fn prepare(&mut self) {
        self.bp.rebind(self.plan.tospace());
    }
    fn release(&mut self) {
        // self.bp.rebind(Some(self.plan.tospace()));
    }
    #[inline(always)]
    fn alloc_copy(
        &mut self,
        _original: ObjectReference,
        bytes: usize,
        align: usize,
        offset: isize,
        _semantics: crate::AllocationSemantics,
    ) -> Address {
        self.bp.alloc(bytes, align, offset)
    }
    #[inline(always)]
    fn post_copy(
        &mut self,
        obj: ObjectReference,
        _tib: Address,
        _bytes: usize,
        _semantics: crate::AllocationSemantics,
    ) {
        object_forwarding::clear_forwarding_bits::<VM>(obj);
    }
}

impl<VM: VMBinding> TSCopyContext<VM> {
    pub fn new(mmtk: &'static MMTK<VM>) -> Self {
        let plan = &mmtk.plan.downcast_ref::<TripleSpace<VM>>().unwrap();
        Self {
            plan,
            // it doesn't matter which space we bind with the copy allocator. We will rebind to a proper space in prepare().
            bp: BumpAllocator::new(VMThread::UNINITIALIZED, plan.tospace(), &*mmtk.plan),
        }
    }
}

impl<VM: VMBinding> WorkerLocal for TSCopyContext<VM> {
    fn init(&mut self, tls: VMWorkerThread) {
        CopyContext::init(self, tls);
    }
}

pub struct TSProcessEdges<VM: VMBinding> {
    // Use a static ref to the specific plan to avoid overhead from dynamic dispatch or
    // downcast for each traced object.
    plan: &'static TripleSpace<VM>,
    base: ProcessEdgesBase<TSProcessEdges<VM>>,
}

impl<VM: VMBinding> TSProcessEdges<VM> {
    fn ts(&self) -> &'static TripleSpace<VM> {
        self.plan
    }

    #[inline]
    fn try_trace_object_in(
        &mut self,
        space: &CopySpace<VM>,
        object: ObjectReference
    ) -> Option<ObjectReference> {
        if space.in_space(object) {
            Some(space.trace_object::<Self, TSCopyContext<VM>>(
                self,
                object,
                super::global::ALLOC_TS,
                unsafe { self.worker().local::<TSCopyContext<VM>>() },
            ))
        } else {
            None
        }
    }
}

impl<VM: VMBinding> ProcessEdgesWork for TSProcessEdges<VM> {
    type VM = VM;
    fn new(edges: Vec<Address>, _roots: bool, mmtk: &'static MMTK<VM>) -> Self {
        let base = ProcessEdgesBase::new(edges, mmtk);
        let plan = base.plan().downcast_ref::<TripleSpace<VM>>().unwrap();
        Self { plan, base }
    }

    #[inline]
    fn trace_object(&mut self, object: ObjectReference) -> ObjectReference {
        if object.is_null() {
            return object;
        }
        self.try_trace_object_in(self.ts().tospace(), object)
            .or_else(|| {
                self.try_trace_object_in(self.ts().fromspace(), object)
            })
            .or_else(|| {
                self.try_trace_object_in(&self.ts().youngspace, object)
            })
            .unwrap_or_else(|| {
                self.ts()
                    .common
                    .trace_object::<Self, TSCopyContext<VM>>(self, object)
            })
    }
}

impl<VM: VMBinding> Deref for TSProcessEdges<VM> {
    type Target = ProcessEdgesBase<Self>;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl<VM: VMBinding> DerefMut for TSProcessEdges<VM> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
