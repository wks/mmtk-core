pub(crate) mod closure;
pub(crate) mod root;
pub(crate) mod weakref;

use std::marker::PhantomData;

use crate::{
    plan::{
        tracing::{
            gc_work::{
                closure::ProcessNodes,
                weakref::{VMForwardWeakRefs, VMPostForwarding, VMProcessWeakRefs},
            },
            Trace,
        },
        VectorObjectQueue,
    },
    scheduler::{
        gc_work::{Prepare, Release, StopMutators},
        GCWorkContext, GCWorkScheduler, GCWorker, WorkBucketStage,
    },
    util::ObjectReference,
    vm::{ObjectTracer, ObjectTracerContext},
    Plan,
};

/// This implementation of [`ObjectTracerContext`] creates the [`DefaultObjectTracer`] to expand the
/// transitive closure during a stop-the-world tracing GC or the final mark pause of a concurrent
/// GC.  It is used during object scanning as well as weak reference processing.
#[derive(Clone)]
pub(crate) struct DefaultObjectTracerContext<T: Trace> {
    stage: WorkBucketStage,
    phantom_data: PhantomData<T>,
}

impl<T: Trace> DefaultObjectTracerContext<T> {
    pub fn new(stage: WorkBucketStage) -> Self {
        Self {
            stage,
            phantom_data: PhantomData,
        }
    }
}

impl<T: Trace> ObjectTracerContext<T::VM> for DefaultObjectTracerContext<T> {
    type TracerType<'w> = DefaultObjectTracer<'w, T>;

    fn with_tracer<'w, R, F>(&self, worker: &'w mut GCWorker<T::VM>, func: F) -> R
    where
        F: FnOnce(&mut Self::TracerType<'w>) -> R,
    {
        let mmtk = worker.mmtk;

        // Create the callback tracer.
        let mut tracer = DefaultObjectTracer::new(worker, T::from_mmtk(mmtk), self.stage);

        // The caller can use the tracer here.
        let result = func(&mut tracer);

        // Flush the queued nodes.
        tracer.flush_if_not_empty();

        result
    }
}

/// This implementation of [`ObjectTracer`] queues newly visited objects and create the
/// [`ProcessNodes`] work packets to scan and trace objects.
pub(crate) struct DefaultObjectTracer<'w, T: Trace> {
    worker: &'w mut GCWorker<T::VM>,
    trace: T,
    queue: VectorObjectQueue,
    stage: WorkBucketStage,
}

impl<T: Trace> ObjectTracer for DefaultObjectTracer<'_, T> {
    /// Forward the `trace_object` call to the underlying `Trace`,
    /// and flush as soon as `self.queue` is full.
    fn trace_object(&mut self, object: ObjectReference) -> ObjectReference {
        let result = self
            .trace
            .trace_object(self.worker, object, &mut self.queue);
        self.flush_if_full();
        result
    }
}

impl<'w, T: Trace> DefaultObjectTracer<'w, T> {
    fn new(worker: &'w mut GCWorker<T::VM>, trace: T, stage: WorkBucketStage) -> Self {
        Self {
            worker,
            trace,
            queue: VectorObjectQueue::new(),
            stage,
        }
    }

    fn flush_if_full(&mut self) {
        if self.queue.is_full() {
            self.flush();
        }
    }

    pub fn flush_if_not_empty(&mut self) {
        if !self.queue.is_empty() {
            self.flush();
        }
    }

    fn flush(&mut self) {
        let next_nodes = self.queue.take();
        assert!(!next_nodes.is_empty());
        let work_packet = ProcessNodes::<T>::new(next_nodes, self.stage);
        self.worker.scheduler().work_buckets[self.stage].add(work_packet);
    }
}

pub(crate) fn schedule_common_work<C: GCWorkContext>(
    scheduler: &GCWorkScheduler<C::VM>,
    plan: &'static <C as GCWorkContext>::PlanType,
) {
    // Stop & scan mutators (mutator scanning can happen before STW)
    scheduler.work_buckets[WorkBucketStage::Unconstrained].add(StopMutators::<C>::new());

    // Prepare global/collectors/mutators
    scheduler.work_buckets[WorkBucketStage::Prepare].add(Prepare::<C>::new(plan));

    // Release global/collectors/mutators
    scheduler.work_buckets[WorkBucketStage::Release].add(Release::<C>::new(plan));

    // Analysis GC work
    #[cfg(feature = "analysis")]
    {
        use crate::util::analysis::GcHookWork;
        scheduler.work_buckets[WorkBucketStage::Unconstrained].add(GcHookWork);
    }

    // Sanity
    #[cfg(feature = "sanity")]
    {
        use crate::util::sanity::sanity_checker::ScheduleSanityGC;
        scheduler.work_buckets[WorkBucketStage::Final]
            .add(ScheduleSanityGC::<C::PlanType>::new(plan));
    }

    // Reference processing
    if !*plan.base().options.no_reference_types {
        use crate::util::reference_processor::{
            PhantomRefProcessing, SoftRefProcessing, WeakRefProcessing,
        };
        scheduler.work_buckets[WorkBucketStage::SoftRefClosure]
            .add(SoftRefProcessing::<C::DefaultTrace>::new());
        scheduler.work_buckets[WorkBucketStage::WeakRefClosure]
            .add(WeakRefProcessing::<C::VM>::new());
        scheduler.work_buckets[WorkBucketStage::PhantomRefClosure]
            .add(PhantomRefProcessing::<C::VM>::new());

        use crate::util::reference_processor::RefForwarding;
        if plan.constraints().needs_forward_after_liveness {
            scheduler.work_buckets[WorkBucketStage::RefForwarding]
                .add(RefForwarding::<C::DefaultTrace>::new());
        }

        use crate::util::reference_processor::RefEnqueue;
        scheduler.work_buckets[WorkBucketStage::Release].add(RefEnqueue::<C::VM>::new());
    }

    // Finalization
    if !*plan.base().options.no_finalizer {
        use crate::util::finalizable_processor::{Finalization, ForwardFinalization};
        // finalization
        scheduler.work_buckets[WorkBucketStage::FinalRefClosure]
            .add(Finalization::<C::DefaultTrace>::new());
        // forward refs
        if plan.constraints().needs_forward_after_liveness {
            scheduler.work_buckets[WorkBucketStage::FinalizableForwarding]
                .add(ForwardFinalization::<C::DefaultTrace>::new());
        }
    }

    // We add the VM-specific weak ref processing work regardless of MMTK-side options,
    // including Options::no_finalizer and Options::no_reference_types.
    //
    // VMs need weak reference handling to function properly.  The VM may treat weak references
    // as strong references, but it is not appropriate to simply disable weak reference
    // handling from MMTk's side.  The VM, however, may choose to do nothing in
    // `Collection::process_weak_refs` if appropriate.
    //
    // It is also not sound for MMTk core to turn off weak
    // reference processing or finalization alone, because (1) not all VMs have the notion of
    // weak references or finalizers, so it may not make sence, and (2) the VM may
    // processing them together.

    // VM-specific weak ref processing
    // The `VMProcessWeakRefs` work packet is set as the sentinel so that it is executed when
    // the `VMRefClosure` bucket is drained.  The VM binding may spawn new work packets into
    // the `VMRefClosure` bucket, and request another `VMProcessWeakRefs` work packet to be
    // executed again after this bucket is drained again.  Strictly speaking, the first
    // `VMProcessWeakRefs` packet can be an ordinary packet (doesn't have to be a sentinel)
    // because there are no other packets in the bucket.  We set it as sentinel for
    // consistency.
    scheduler.work_buckets[WorkBucketStage::VMRefClosure]
        .set_sentinel(Box::new(VMProcessWeakRefs::<C::DefaultTrace>::new()));

    if plan.constraints().needs_forward_after_liveness {
        // VM-specific weak ref forwarding
        scheduler.work_buckets[WorkBucketStage::VMRefForwarding]
            .add(VMForwardWeakRefs::<C::DefaultTrace>::new());
    }

    scheduler.work_buckets[WorkBucketStage::Release].add(VMPostForwarding::<C::VM>::default());
}
