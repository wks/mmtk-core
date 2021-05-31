use crate::mmtk::MMTK;
use crate::plan::global::{BasePlan, NoCopy};
use super::mutator::ALLOCATOR_MAPPING;
use crate::plan::AllocationSemantics;
use crate::plan::Plan;
use crate::plan::PlanConstraints;
use crate::policy::space::Space;
use crate::scheduler::GCWorkerLocal;
use crate::scheduler::GCWorkerLocalPtr;
use crate::scheduler::MMTkScheduler;
use crate::util::alloc::allocators::AllocatorSelector;
use crate::util::heap::layout::heap_layout::Mmapper;
use crate::util::heap::layout::heap_layout::VMMap;
use crate::util::heap::layout::vm_layout_constants::{HEAP_END, HEAP_START};
use crate::util::heap::HeapMeta;
#[allow(unused_imports)]
use crate::util::heap::VMRequest;
use crate::util::opaque_pointer::*;
use crate::util::options::UnsafeOptionsWrapper;
use crate::util::metadata::side_metadata::{SideMetadataContext, SideMetadataSanity};
use crate::vm::VMBinding;
use enum_map::EnumMap;
use std::sync::Arc;

use crate::policy::immortalspace::ImmortalSpace as TripleSpaceImmortalSpace;

pub struct TripleSpace<VM: VMBinding> {
    pub base: BasePlan<VM>,
    pub immortal_space: TripleSpaceImmortalSpace<VM>,
}

pub const TS_CONSTRAINTS: PlanConstraints = PlanConstraints::default();

impl<VM: VMBinding> Plan for TripleSpace<VM> {
    type VM = VM;

    fn constraints(&self) -> &'static PlanConstraints {
        &TS_CONSTRAINTS
    }

    fn create_worker_local(
        &self,
        tls: VMWorkerThread,
        mmtk: &'static MMTK<Self::VM>,
    ) -> GCWorkerLocalPtr {
        let mut c = NoCopy::new(mmtk);
        c.init(tls);
        GCWorkerLocalPtr::new(c)
    }

    fn gc_init(
        &mut self,
        heap_size: usize,
        vm_map: &'static VMMap,
        scheduler: &Arc<MMTkScheduler<VM>>,
    ) {
        self.base.gc_init(heap_size, vm_map, scheduler);

        // FIXME correctly initialize spaces based on options
        self.immortal_space.init(&vm_map);
    }

    fn collection_required(&self, space_full: bool, space: &dyn Space<Self::VM>) -> bool {
        self.base.collection_required(self, space_full, space)
    }

    fn base(&self) -> &BasePlan<VM> {
        &self.base
    }

    fn prepare(&mut self, _tls: VMWorkerThread) {
        unreachable!()
    }

    fn release(&mut self, _tls: VMWorkerThread) {
        unreachable!()
    }

    fn get_allocator_mapping(&self) -> &'static EnumMap<AllocationSemantics, AllocatorSelector> {
        &*ALLOCATOR_MAPPING
    }

    fn schedule_collection(&'static self, _scheduler: &MMTkScheduler<VM>) {
        unreachable!("GC triggered in the incomplete triplespace")
    }

    fn get_pages_used(&self) -> usize {
        self.immortal_space.reserved_pages()
    }

    fn handle_user_collection_request(&self, _tls: VMMutatorThread, _force: bool) {
        println!("Warning: User attempted a collection request, but it is not supported in TripleSpace. The request is ignored.");
    }
}

impl<VM: VMBinding> TripleSpace<VM> {
    pub fn new(
        vm_map: &'static VMMap,
        mmapper: &'static Mmapper,
        options: Arc<UnsafeOptionsWrapper>,
    ) -> Self {
        let mut heap = HeapMeta::new(HEAP_START, HEAP_END);

        let global_specs = SideMetadataContext::new_global_specs(&[]);

        let immortal_space = TripleSpaceImmortalSpace::new(
            "immortal_space",
            true,
            VMRequest::discontiguous(),
            global_specs.clone(),
            vm_map,
            mmapper,
            &mut heap,
            &TS_CONSTRAINTS,
        );

        let res = TripleSpace {
            immortal_space,
            base: BasePlan::new(
                vm_map,
                mmapper,
                options,
                heap,
                &TS_CONSTRAINTS,
                global_specs,
            ),
        };

        let mut side_metadata_sanity_checker = SideMetadataSanity::new();
        res.base
            .verify_side_metadata_sanity(&mut side_metadata_sanity_checker);
        res.immortal_space
            .verify_side_metadata_sanity(&mut side_metadata_sanity_checker);

        res
    }
}
