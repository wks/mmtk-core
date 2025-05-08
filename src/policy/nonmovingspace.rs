//! This module implements non-moving space.
//!
//! We can use Cargo features to choose one of `ImmixSpace` (default), `ImmortalSpace` and
//! `MarkSweepSpace` as the non-moving space.

use crate::{util::VMWorkerThread, vm::VMBinding, AllocationSemantics, Mutator};

use super::space::{PlanCreateSpaceArgs, Space};

/// A space implements this trait if it can be used as the non-moving space.
///
/// Methods of this trait should be called inste
pub trait NonMovingSpace<VM: VMBinding>: Space<VM> {
    /// Call this instead of `new` to create the space as the non-moving space.
    fn new_nonmoving_space(args: PlanCreateSpaceArgs<VM>) -> Self;
    /// Call this instead of `prepare` when used as the non-moving space.
    fn prepare_nonmoving_space(&mut self, full_heap: bool);
    /// Call this instead of `release` when used as the non-moving space.
    fn release_nonmoving_space(&mut self, full_heap: bool);
    /// Call this instead of `end_of_gc` when used as the non-moving space.
    fn end_of_gc_nonmoving_space(&mut self);
    /// Call this in `common_release_func` when using this space as the non-moving space.
    fn release_mutator_nonmoving_space(mutator: &mut Mutator<VM>, tls: VMWorkerThread);
}

impl<VM: VMBinding> NonMovingSpace<VM> for crate::policy::immortalspace::ImmortalSpace<VM> {
    fn new_nonmoving_space(space_args: PlanCreateSpaceArgs<VM>) -> Self {
        Self::new(space_args)
    }

    fn prepare_nonmoving_space(&mut self, _full_heap: bool) {
        self.prepare();
    }

    fn release_nonmoving_space(&mut self, _full_heap: bool) {
        self.release();
    }

    fn end_of_gc_nonmoving_space(&mut self) {
        // Do nothing
    }

    fn release_mutator_nonmoving_space(_mutator: &mut Mutator<VM>, _tls: VMWorkerThread) {
        // Do nothing
    }
}

impl<VM: VMBinding> NonMovingSpace<VM>
    for crate::policy::marksweepspace::native_ms::MarkSweepSpace<VM>
{
    fn new_nonmoving_space(space_args: PlanCreateSpaceArgs<VM>) -> Self {
        Self::new(space_args)
    }

    fn prepare_nonmoving_space(&mut self, full_heap: bool) {
        self.prepare(full_heap);
    }

    fn release_nonmoving_space(&mut self, _full_heap: bool) {
        self.release();
    }

    fn end_of_gc_nonmoving_space(&mut self) {
        self.end_of_gc();
    }

    fn release_mutator_nonmoving_space(mutator: &mut Mutator<VM>, _tls: VMWorkerThread) {
        use crate::util::alloc::FreeListAllocator;
        unsafe {
            mutator
                .allocators
                .get_allocator_mut(mutator.config.allocator_mapping[AllocationSemantics::NonMoving])
        }
        .downcast_mut::<FreeListAllocator<VM>>()
        .unwrap()
        .release();
    }
}

impl<VM: VMBinding> NonMovingSpace<VM> for crate::policy::immix::ImmixSpace<VM> {
    fn new_nonmoving_space(space_args: PlanCreateSpaceArgs<VM>) -> Self {
        Self::new(
            space_args,
            crate::policy::immix::ImmixSpaceArgs {
                unlog_object_when_traced: false,
                #[cfg(feature = "vo_bit")]
                mixed_age: false,
                never_move_objects: true,
            },
        )
    }

    fn prepare_nonmoving_space(&mut self, full_heap: bool) {
        self.prepare(full_heap, None);
    }

    fn release_nonmoving_space(&mut self, full_heap: bool) {
        self.release(full_heap);
    }

    fn end_of_gc_nonmoving_space(&mut self) {
        self.end_of_gc();
    }

    fn release_mutator_nonmoving_space(mutator: &mut Mutator<VM>, _tls: VMWorkerThread) {
        use crate::util::alloc::ImmixAllocator;
        unsafe {
            mutator
                .allocators
                .get_allocator_mut(mutator.config.allocator_mapping[AllocationSemantics::NonMoving])
        }
        .downcast_mut::<ImmixAllocator<VM>>()
        .unwrap()
        .reset();
    }
}

// Choose one concrete non-moving space implementation according to the enabled Cargo feature.
cfg_if::cfg_if! {
    if #[cfg(feature = "immortal_as_nonmoving")] {
        pub type ChosenNonMovingSpace<VM> = crate::policy::immortalspace::ImmortalSpace<VM>;
    } else if #[cfg(feature = "marksweep_as_nonmoving")] {
        pub type ChosenNonMovingSpace<VM> = crate::policy::marksweepspace::native_ms::MarkSweepSpace<VM>;
    } else {
        pub type ChosenNonMovingSpace<VM> = crate::policy::immix::ImmixSpace<VM>;
    }
}
