use crate::vm::VMBinding;

use super::space::{PlanCreateSpaceArgs, Space};

pub trait NonMovingSpace<VM: VMBinding>: Space<VM> {
    fn new_nonmoving_space(args: PlanCreateSpaceArgs<VM>) -> Self;
    fn prepare_nonmoving_space(&mut self, full_heap: bool);
    fn release_nonmoving_space(&mut self, full_heap: bool);
    fn end_of_gc_nonmoving_space(&mut self);
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
}

cfg_if::cfg_if! {
    if #[cfg(feature = "immortal_as_nonmoving")] {
        pub type ChosenNonMovingSpace<VM> = crate::policy::immortalspace::ImmortalSpace<VM>;
    } else if #[cfg(feature = "marksweep_as_nonmoving")] {
        pub type ChosenNonMovingSpace<VM> = crate::policy::marksweepspace::native_ms::MarkSweepSpace<VM>;
    } else {
        pub type ChosenNonMovingSpace<VM> = crate::policy::immix::ImmixSpace<VM>;
    }
}
