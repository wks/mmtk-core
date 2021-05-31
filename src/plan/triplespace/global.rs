use crate::mmtk::MMTK;
use crate::plan::global::{BasePlan, CommonPlan, NoCopy};
use super::mutator::ALLOCATOR_MAPPING;
use crate::plan::AllocationSemantics;
use crate::plan::Plan;
use crate::plan::PlanConstraints;
use crate::policy::space::Space;
use crate::policy::copyspace::CopySpace;
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
use std::sync::atomic::{AtomicBool, Ordering};

pub struct TripleSpace<VM: VMBinding> {
    pub hi: AtomicBool,
    pub copyspace0: CopySpace<VM>,
    pub copyspace1: CopySpace<VM>,
    pub youngspace: CopySpace<VM>,
    pub common: CommonPlan<VM>,
}

pub const TS_CONSTRAINTS: PlanConstraints = PlanConstraints {
    gc_header_bits: 2,
    gc_header_words: 0,
    moves_objects: true,
    num_specialized_scans: 1,
    ..PlanConstraints::default()
};

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
        self.common.gc_init(heap_size, vm_map, scheduler);

        self.copyspace0.init(&vm_map);
        self.copyspace1.init(&vm_map);
        self.youngspace.init(&vm_map);
    }

    fn collection_required(&self, space_full: bool, space: &dyn Space<Self::VM>) -> bool {
        self.base().collection_required(self, space_full, space)
    }

    fn base(&self) -> &BasePlan<VM> {
        &self.common().base
    }

    fn common(&self) -> &CommonPlan<VM> {
        &self.common
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

    fn get_collection_reserve(&self) -> usize {
        self.tospace().reserved_pages() +
            self.youngspace.reserved_pages()
    }

    fn get_pages_used(&self) -> usize {
        self.tospace().reserved_pages() +
            self.youngspace.reserved_pages() +
            self.common.get_pages_used()
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

        let global_metadata_specs = SideMetadataContext::new_global_specs(&[]);

        let res = TripleSpace {
            hi: AtomicBool::new(false),
            copyspace0: CopySpace::new(
                "copyspace0",
                false,
                true,
                VMRequest::discontiguous(),
                global_metadata_specs.clone(),
                vm_map,
                mmapper,
                &mut heap,
            ),
            copyspace1: CopySpace::new(
                "copyspace1",
                true,
                true,
                VMRequest::discontiguous(),
                global_metadata_specs.clone(),
                vm_map,
                mmapper,
                &mut heap,
            ),
            youngspace: CopySpace::new(
                "youngspace",
                false,
                true,
                VMRequest::discontiguous(),
                global_metadata_specs.clone(),
                vm_map,
                mmapper,
                &mut heap,
            ),
            common: CommonPlan::new(
                vm_map,
                mmapper,
                options,
                heap,
                &TS_CONSTRAINTS,
                global_metadata_specs,
            ),
        };

        let mut side_metadata_sanity_checker = SideMetadataSanity::new();
        res.common
            .verify_side_metadata_sanity(&mut side_metadata_sanity_checker);
        res.copyspace0
            .verify_side_metadata_sanity(&mut side_metadata_sanity_checker);
        res.copyspace1
            .verify_side_metadata_sanity(&mut side_metadata_sanity_checker);
        res.youngspace
            .verify_side_metadata_sanity(&mut side_metadata_sanity_checker);

        res
    }

    pub fn tospace(&self) -> &CopySpace<VM> {
        if self.hi.load(Ordering::SeqCst) {
            &self.copyspace1
        } else {
            &self.copyspace0
        }
    }

    pub fn fromspace(&self) -> &CopySpace<VM> {
        if self.hi.load(Ordering::SeqCst) {
            &self.copyspace0
        } else {
            &self.copyspace1
        }
    }
}
