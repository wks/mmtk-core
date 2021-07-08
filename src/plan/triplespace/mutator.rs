use crate::MMTK;
use crate::plan::barriers::ObjectRememberingBarrier;
use crate::plan::mutator_context::Mutator;
use crate::plan::mutator_context::MutatorConfig;
use crate::plan::triplespace::gc_work::TSProcessEdges;
use super::TripleSpace;
use crate::plan::AllocationSemantics as AllocationType;
use crate::plan::Plan;
use crate::util::alloc::allocators::{AllocatorSelector, Allocators};
use crate::util::alloc::BumpAllocator;
use crate::util::{VMMutatorThread, VMWorkerThread};
use crate::vm::{ObjectModel, VMBinding};
use enum_map::enum_map;
use enum_map::EnumMap;

pub fn ts_mutator_prepare<VM: VMBinding>(_mutator: &mut Mutator<VM>, _tls: VMWorkerThread) {
    // Do nothing
}

pub fn ts_mutator_release<VM: VMBinding>(mutator: &mut Mutator<VM>, _tls: VMWorkerThread) {
    // rebind the allocation bump pointer to the appropriate semispace
    let bump_allocator = unsafe {
        mutator
            .allocators
            .get_allocator_mut(mutator.config.allocator_mapping[AllocationType::Default])
    }
    .downcast_mut::<BumpAllocator<VM>>()
    .unwrap();
    bump_allocator.reset();
}

lazy_static! {
    pub static ref ALLOCATOR_MAPPING: EnumMap<AllocationType, AllocatorSelector> = enum_map! {
        AllocationType::Default => AllocatorSelector::BumpPointer(0),
        AllocationType::Immortal | AllocationType::Code | AllocationType::ReadOnly => AllocatorSelector::BumpPointer(1),
        AllocationType::Los => AllocatorSelector::LargeObject(0),
    };
}

pub fn create_ts_mutator<VM: VMBinding>(
    mutator_tls: VMMutatorThread,
    plan: &'static dyn Plan<VM = VM>,
    mmtk: &'static MMTK<VM>,
) -> Mutator<VM> {
    let ts_plan = &plan.downcast_ref::<TripleSpace<VM>>().unwrap();
    let config = MutatorConfig {
        allocator_mapping: &*ALLOCATOR_MAPPING,
        space_mapping: box vec![
            (AllocatorSelector::BumpPointer(0), &ts_plan.youngspace),
            (AllocatorSelector::BumpPointer(1), ts_plan.common.get_immortal()),
            (AllocatorSelector::LargeObject(0), ts_plan.common.get_los()),
        ],
        prepare_func: &ts_mutator_prepare,
        release_func: &ts_mutator_release,
    };

    Mutator {
        allocators: Allocators::<VM>::new(mutator_tls, plan, &config.space_mapping),
        barrier: Box::new(ObjectRememberingBarrier::<TSProcessEdges<VM, false>>::new(
            mmtk,
            VM::VMObjectModel::GLOBAL_LOG_BIT_SPEC,
        )),
        mutator_tls,
        config,
        plan,
    }
}
