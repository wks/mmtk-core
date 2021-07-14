use super::gc_work::{TSCopyContext, TSProcessEdges};
use crate::mmtk::MMTK;
use crate::plan::global::{BasePlan, CommonPlan};
use crate::plan::global::GcStatus;
use super::mutator::ALLOCATOR_MAPPING;
use crate::plan::AllocationSemantics;
use crate::plan::Plan;
use crate::plan::PlanConstraints;
use crate::policy::space::Space;
use crate::policy::copyspace::CopySpace;
use crate::scheduler::gc_work::*;
use crate::{BarrierSelector, scheduler::*};
use crate::util::alloc::allocators::AllocatorSelector;
use crate::util::heap::layout::heap_layout::Mmapper;
use crate::util::heap::layout::heap_layout::VMMap;
use crate::util::heap::layout::vm_layout_constants::{HEAP_END, HEAP_START};
use crate::util::heap::HeapMeta;
#[allow(unused_imports)]
use crate::util::heap::VMRequest;
use crate::util::{conversions, metadata, opaque_pointer::*};
use crate::util::options::UnsafeOptionsWrapper;
use crate::util::metadata::side_metadata::SideMetadataSanity;
use crate::vm::*;
use enum_map::EnumMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(feature = "sanity")]
use crate::util::sanity::sanity_checker::*;

pub const ALLOC_TS: AllocationSemantics = AllocationSemantics::Default;

pub struct TripleSpace<VM: VMBinding> {
    // State fields
    pub hi: AtomicBool,
    gc_full_heap: AtomicBool, // Set by schedule, used by prepare/release
    next_gc_full_heap: AtomicBool, // Set by GC trigger/release, used by schedule

    // Spaces
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
    barrier: BarrierSelector::ObjectBarrier,
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
        let mut c = TSCopyContext::new(mmtk);
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
        let nursery_full = self.youngspace.reserved_pages()
            >= (conversions::bytes_to_pages_up(self.base().options.max_nursery));
        if nursery_full {
            info!("Nursery is full. Nursery size: {}", self.base().options.max_nursery);
            return true;
        }

        if space_full && space.common().descriptor != self.youngspace.common().descriptor {
            self.next_gc_full_heap.store(true, Ordering::SeqCst);
        }

        self.base().collection_required(self, space_full, space)
    }

    fn base(&self) -> &BasePlan<VM> {
        &self.common().base
    }

    fn common(&self) -> &CommonPlan<VM> {
        &self.common
    }

    fn prepare(&mut self, tls: VMWorkerThread) {
        self.common.prepare(tls, true);
        self.youngspace.prepare(true);

        let gc_full_heap = self.gc_full_heap.load(Ordering::SeqCst);
        let hi = {
            let old_hi = self.hi.load(Ordering::SeqCst);
            info!("[triplespace] old_hi = {}", old_hi);
            if gc_full_heap {
                let new_hi = !old_hi;
                self.hi.store(new_hi, Ordering::SeqCst); // flip the semi-spaces
                new_hi
            } else {
                old_hi
            }
        };
        info!("[triplespace] hi = {}", hi);

        self.copyspace0.prepare(hi);
        self.copyspace1.prepare(!hi);
    }

    fn release(&mut self, tls: VMWorkerThread) {
        self.common.release(tls, true);
        self.youngspace.release();

        let gc_full_heap = self.gc_full_heap.load(Ordering::SeqCst);

        if gc_full_heap {
            // release the collected region only after full GC
            self.fromspace().release();
        }

        self.next_gc_full_heap.store(
            self.get_pages_avail()
                < conversions::bytes_to_pages_up(self.base().options.min_nursery),
            Ordering::SeqCst,
        );

    }

    fn get_allocator_mapping(&self) -> &'static EnumMap<AllocationSemantics, AllocatorSelector> {
        &*ALLOCATOR_MAPPING
    }

    fn schedule_collection(&'static self, scheduler: &MMTkScheduler<VM>) {
        let is_full_heap = self.request_full_heap_collection();
        //let is_full_heap = true;
        self.gc_full_heap.store(is_full_heap, Ordering::SeqCst);

        self.base().set_collection_kind();
        self.base().set_gc_status(GcStatus::GcPrepare);

        fn schedule_process_edges<VM: VMBinding, const IS_FULL_HEAP: bool>(common: &CommonPlan<VM>, scheduler: &MMTkScheduler<VM>) {
            info!("[triplespace] IS_FULL_HEAP={}", IS_FULL_HEAP);
            common.schedule_common::<TSProcessEdges<VM, IS_FULL_HEAP>>(&TS_CONSTRAINTS, scheduler);
            // Stop & scan mutators (mutator scanning can happen before STW)
            scheduler.work_buckets[WorkBucketStage::Unconstrained]
                .add(StopMutators::<TSProcessEdges<VM, IS_FULL_HEAP>>::new());
        }

        if !is_full_heap {
            info!("[triplespace] Nursery GC");
            schedule_process_edges::<VM, false>(self.common(), scheduler);
        } else {
            info!("[triplespace] Full heap GC");
            schedule_process_edges::<VM, true>(self.common(), scheduler);
        }

        // Prepare global/collectors/mutators
        scheduler.work_buckets[WorkBucketStage::Prepare]
            .add(Prepare::<Self, TSCopyContext<VM>>::new(self));
        // Release global/collectors/mutators
        scheduler.work_buckets[WorkBucketStage::Release]
            .add(Release::<Self, TSCopyContext<VM>>::new(self));
        // Scheduling all the gc hooks of analysis routines. It is generally recommended
        // to take advantage of the scheduling system we have in place for more performance
        #[cfg(feature = "analysis")]
        scheduler.work_buckets[WorkBucketStage::Unconstrained].add(GcHookWork);
        // Resume mutators
        #[cfg(feature = "sanity")]
        scheduler.work_buckets[WorkBucketStage::Final]
            .add(ScheduleSanityGC::<Self, TSCopyContext<VM>>::new(self));
        scheduler.set_finalizer(Some(EndOfGC));
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

    /// Return the number of pages avilable for allocation. Assuming all future allocations goes to nursery.
    fn get_pages_avail(&self) -> usize {
        // super.get_pages_avail() / 2 to reserve pages for copying
        (self.get_total_pages() - self.get_pages_reserved()) >> 1
    }

    fn is_current_gc_nursery(&self) -> bool {
        !self.gc_full_heap.load(Ordering::SeqCst)
    }
}

impl<VM: VMBinding> TripleSpace<VM> {
    pub fn new(
        vm_map: &'static VMMap,
        mmapper: &'static Mmapper,
        options: Arc<UnsafeOptionsWrapper>,
    ) -> Self {
        let mut heap = HeapMeta::new(HEAP_START, HEAP_END);

        let global_metadata_specs = metadata::extract_side_metadata(&[
            *VM::VMObjectModel::GLOBAL_LOG_BIT_SPEC,
        ]);

        let res = TripleSpace {
            hi: AtomicBool::new(false),
            gc_full_heap: AtomicBool::default(),
            next_gc_full_heap: AtomicBool::new(false),
            youngspace: CopySpace::new(
                "youngspace",
                false,
                true,
                VMRequest::fixed_extent(options.max_nursery, false),
                global_metadata_specs.clone(),
                vm_map,
                mmapper,
                &mut heap,
            ),
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

    fn request_full_heap_collection(&self) -> bool {
        if self.base().user_triggered_collection.load(Ordering::SeqCst)
            && self.base().options.full_heap_system_gc
        {
            return true;
        }

        if self.next_gc_full_heap.load(Ordering::SeqCst)
            || self.base().cur_collection_attempts.load(Ordering::SeqCst) > 1
        {
            // Forces full heap collection
            return true;
        }

        self.get_total_pages() <= self.get_pages_reserved()
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
