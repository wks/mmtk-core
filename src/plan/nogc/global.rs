use crate::plan::global::BasePlan;
use crate::plan::global::CreateGeneralPlanArgs;
use crate::plan::global::CreateSpecificPlanArgs;
use crate::plan::nogc::mutator::ALLOCATOR_MAPPING;
use crate::plan::AllocationSemantics;
use crate::plan::Plan;
use crate::plan::PlanConstraints;
use crate::policy::immortalspace::ImmortalSpace;
use crate::policy::space::Space;
use crate::scheduler::GCWorkScheduler;
use crate::util::alloc::allocators::AllocatorSelector;
use crate::util::heap::heap_meta::SpaceSpec;
use crate::util::metadata::side_metadata::SideMetadataContext;
use crate::util::opaque_pointer::*;
use crate::vm::VMBinding;
use enum_map::EnumMap;
use mmtk_macros::HasSpaces;

#[cfg(not(feature = "nogc_lock_free"))]
use crate::policy::immortalspace::ImmortalSpace as NoGCImmortalSpace;
#[cfg(feature = "nogc_lock_free")]
use crate::policy::lockfreeimmortalspace::LockFreeImmortalSpace as NoGCImmortalSpace;

#[derive(HasSpaces)]
pub struct NoGC<VM: VMBinding> {
    #[parent]
    pub base: BasePlan<VM>,
    #[space]
    pub nogc_space: NoGCImmortalSpace<VM>,
    #[space]
    pub immortal: ImmortalSpace<VM>,
    #[space]
    pub los: ImmortalSpace<VM>,
}

pub const NOGC_CONSTRAINTS: PlanConstraints = PlanConstraints {
    collects_garbage: false,
    ..PlanConstraints::default()
};

impl<VM: VMBinding> Plan for NoGC<VM> {
    fn constraints(&self) -> &'static PlanConstraints {
        &NOGC_CONSTRAINTS
    }

    fn collection_required(&self, space_full: bool, _space: Option<&dyn Space<Self::VM>>) -> bool {
        self.base().collection_required(self, space_full)
    }

    fn base(&self) -> &BasePlan<VM> {
        &self.base
    }

    fn base_mut(&mut self) -> &mut BasePlan<Self::VM> {
        &mut self.base
    }

    fn prepare(&mut self, _tls: VMWorkerThread) {
        unreachable!()
    }

    fn release(&mut self, _tls: VMWorkerThread) {
        unreachable!()
    }

    fn get_allocator_mapping(&self) -> &'static EnumMap<AllocationSemantics, AllocatorSelector> {
        &ALLOCATOR_MAPPING
    }

    fn schedule_collection(&'static self, _scheduler: &GCWorkScheduler<VM>) {
        unreachable!("GC triggered in nogc")
    }

    fn get_used_pages(&self) -> usize {
        self.nogc_space.reserved_pages()
            + self.immortal.reserved_pages()
            + self.los.reserved_pages()
            + self.base.get_used_pages()
    }
}

impl<VM: VMBinding> NoGC<VM> {
    pub fn new(args: CreateGeneralPlanArgs<VM>) -> Self {
        let mut plan_args = CreateSpecificPlanArgs {
            global_args: args,
            constraints: &NOGC_CONSTRAINTS,
            global_side_metadata_specs: SideMetadataContext::new_global_specs(&[]),
        };

        let nogc_space_spec = plan_args
            .global_args
            .heap
            .specify_space(SpaceSpec::DontCare);
        let immortal_spec = plan_args
            .global_args
            .heap
            .specify_space(SpaceSpec::DontCare);
        let los = plan_args
            .global_args
            .heap
            .specify_space(SpaceSpec::DontCare);

        // Spaces will eventually be placed by `BasePlan`.
        let base = BasePlan::new(&mut plan_args);

        let res = NoGC {
            nogc_space: NoGCImmortalSpace::new(plan_args.get_space_args(
                "nogc_space",
                cfg!(not(feature = "nogc_no_zeroing")),
                nogc_space_spec.unwrap(),
            )),
            immortal: ImmortalSpace::new(plan_args.get_space_args(
                "immortal",
                true,
                immortal_spec.unwrap(),
            )),
            los: ImmortalSpace::new(plan_args.get_space_args("los", true, los.unwrap())),
            base,
        };

        res.verify_side_metadata_sanity();

        res
    }
}
