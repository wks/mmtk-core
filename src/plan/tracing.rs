//! This module contains code useful for tracing,
//! i.e. visiting the reachable objects by traversing all or part of an object graph.

use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::mem;

use crate::policy::gc_work::TraceKind;
use crate::scheduler::gc_work::ProcessEdgesWork;
use crate::scheduler::{GCWork, GCWorker, WorkBucketStage};
use crate::util::batcher::Batcher;
use crate::util::{Address, ObjectReference};
use crate::vm::{EdgeVisitor, RootsWorkFactory, Scanning, VMBinding};
use crate::{memory_manager, Plan, MMTK};

use super::PlanTraceObject;

/// This trait represents an object queue to enqueue objects during tracing.
pub trait ObjectQueue {
    /// Enqueue an object into the queue.
    fn enqueue(&mut self, object: ObjectReference);
}

/// This allows us to use a closure as an ObjectQueue
impl<F: FnMut(ObjectReference)> ObjectQueue for F {
    fn enqueue(&mut self, object: ObjectReference) {
        self(object)
    }
}

/// An implementation of `ObjectQueue` using a `Vec`.
pub struct VectorObjectQueue {
    /// Enqueued nodes.
    nodes: Vec<ObjectReference>,
}

impl VectorObjectQueue {
    /// Reserve a capacity of this on first enqueue to avoid frequent resizing.
    const CAPACITY: usize = 4096;

    /// Create an empty `VectorObjectQueue`.
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    /// Return `true` if the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Return the contents of the underlying vector.  It will empty the queue.
    pub fn take(&mut self) -> Vec<ObjectReference> {
        std::mem::take(&mut self.nodes)
    }

    /// Consume this `VectorObjectQueue` and return its underlying vector.
    pub fn into_vec(self) -> Vec<ObjectReference> {
        self.nodes
    }
}

impl Default for VectorObjectQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl ObjectQueue for VectorObjectQueue {
    #[inline(always)]
    fn enqueue(&mut self, object: ObjectReference) {
        if self.nodes.is_empty() {
            self.nodes.reserve(Self::CAPACITY);
        }
        self.nodes.push(object);
    }
}

/// A transitive closure visitor to collect all the edges of an object.
pub struct ObjectsClosure<'a, E: ProcessEdgesWork> {
    buffer: Vec<Address>,
    worker: &'a mut GCWorker<E::VM>,
}

impl<'a, E: ProcessEdgesWork> ObjectsClosure<'a, E> {
    pub fn new(worker: &'a mut GCWorker<E::VM>) -> Self {
        Self {
            buffer: vec![],
            worker,
        }
    }

    fn flush(&mut self) {
        let mut new_edges = Vec::new();
        mem::swap(&mut new_edges, &mut self.buffer);
        self.worker.add_work(
            WorkBucketStage::Closure,
            E::new(new_edges, false, self.worker.mmtk),
        );
    }
}

impl<'a, E: ProcessEdgesWork> EdgeVisitor for ObjectsClosure<'a, E> {
    #[inline(always)]
    fn visit_edge(&mut self, slot: Address) {
        if self.buffer.is_empty() {
            self.buffer.reserve(E::CAPACITY);
        }
        self.buffer.push(slot);
        if self.buffer.len() >= E::CAPACITY {
            let mut new_edges = Vec::new();
            mem::swap(&mut new_edges, &mut self.buffer);
            self.worker.add_work(
                WorkBucketStage::Closure,
                E::new(new_edges, false, self.worker.mmtk),
            );
        }
    }
}

impl<'a, E: ProcessEdgesWork> Drop for ObjectsClosure<'a, E> {
    #[inline(always)]
    fn drop(&mut self) {
        self.flush();
    }
}

/// A generic trait suppposed to be suitable for most tracing GC algorithms,
/// including SemiSpace, MarkSweep, Immix, GenCopy and GenImmix.
trait TracingWorkContext {
    type VM: VMBinding;
    type PlanType: Plan<VM = Self::VM> + PlanTraceObject<Self::VM>;
    type RootsWorkFactoryType: RootsWorkFactory;
    type ProcessEdgesWorkType: GCWork<Self::VM>;
    type ProcessNodesWorkType: GCWork<Self::VM>;

    fn create_roots_work_factory(&self) -> Self::RootsWorkFactoryType;
    fn create_process_edges_work(&self, edges: Vec<Address>) -> Self::ProcessEdgesWorkType;
    fn create_process_nodes_work(&self, nodes: Vec<ObjectReference>) -> Self::ProcessNodesWorkType;
}

struct SimpleTracingWorkContext<
    VM: VMBinding,
    PlanType: Plan<VM = VM> + PlanTraceObject<VM>,
    const KIND: TraceKind,
> {
    mmtk: &'static MMTK<VM>,
    plan: &'static PlanType,
}

impl<VM: VMBinding, PlanType: Plan<VM = VM> + PlanTraceObject<VM>, const KIND: TraceKind> Clone
    for SimpleTracingWorkContext<VM, PlanType, KIND>
{
    fn clone(&self) -> Self {
        Self {
            mmtk: self.mmtk.clone(),
            plan: self.plan.clone(),
        }
    }
}

unsafe impl<VM: VMBinding, PlanType: Plan<VM = VM> + PlanTraceObject<VM>, const KIND: TraceKind>
    Send for SimpleTracingWorkContext<VM, PlanType, KIND>
{
}

impl<VM: VMBinding, PlanType: Plan<VM = VM> + PlanTraceObject<VM>, const KIND: TraceKind>
    TracingWorkContext for SimpleTracingWorkContext<VM, PlanType, KIND>
{
    type VM = VM;
    type PlanType = PlanType;
    type RootsWorkFactoryType = Self;
    type ProcessEdgesWorkType = SimpleTracingProcessEdges<VM, PlanType, KIND>;
    type ProcessNodesWorkType = SimpleTracingProcessEdges<VM, PlanType, KIND>;

    fn create_roots_work_factory(&self) -> Self::RootsWorkFactoryType {
        self.clone()
    }

    fn create_process_edges_work(&self, edges: Vec<Address>) -> Self::ProcessEdgesWorkType {
        SimpleTracingProcessEdges {
            context: self.clone(),
            edges,
        }
    }

    fn create_process_nodes_work(&self, nodes: Vec<ObjectReference>) -> Self::ProcessNodesWorkType {
        todo!()
    }
}

impl<VM: VMBinding, PlanType: Plan<VM = VM> + PlanTraceObject<VM>, const KIND: TraceKind>
    RootsWorkFactory for SimpleTracingWorkContext<VM, PlanType, KIND>
{
    fn create_process_edge_roots_work(&mut self, edges: Vec<Address>) {
        let work = TracingWorkContext::create_process_edges_work(self, edges);
        memory_manager::add_work_packet(self.mmtk, WorkBucketStage::Closure, work);
    }

    fn create_process_node_roots_work(&mut self, nodes: Vec<ObjectReference>) {
        let work = TracingWorkContext::create_process_nodes_work(self, nodes);
        memory_manager::add_work_packet(self.mmtk, WorkBucketStage::Closure, work);
    }
}

struct SimpleTracingProcessEdges<
    VM: VMBinding,
    PlanType: Plan<VM = VM> + PlanTraceObject<VM>,
    const KIND: TraceKind,
> {
    context: SimpleTracingWorkContext<VM, PlanType, KIND>,
    edges: Vec<Address>,
}

impl<VM: VMBinding, PlanType: Plan<VM = VM> + PlanTraceObject<VM>, const KIND: TraceKind>
    SimpleTracingProcessEdges<VM, PlanType, KIND>
{
    const SCAN_OBJECTS_IMMEDIATELY: bool = true;
}

impl<VM: VMBinding, PlanType: Plan<VM = VM> + PlanTraceObject<VM>, const KIND: TraceKind> GCWork<VM>
    for SimpleTracingProcessEdges<VM, PlanType, KIND>
{
    fn do_work(&mut self, worker: &mut GCWorker<VM>, mmtk: &'static MMTK<VM>) {
        trace!("SimpleTracingProcessEdges");

        // Process edges
        let mut object_queue = VectorObjectQueue::new();
        for edge in self.edges.iter() {
            let object = unsafe { edge.load::<ObjectReference>() };

            if object.is_null() {
                continue;
            }

            let new_object = self.context.plan.trace_object::<VectorObjectQueue, KIND>(
                &mut object_queue,
                object,
                worker,
            );

            if PlanType::may_move_objects::<KIND>() {
                unsafe { edge.store(new_object) };
            }
        }

        // Scan objects if any objects are enqueued, now or later.
        if !object_queue.is_empty() {
            let nodes = object_queue.into_vec();
            let process_nodes_work = self.context.create_process_nodes_work(nodes);
            if Self::SCAN_OBJECTS_IMMEDIATELY {
                // TODO: Why does worker need to be 'static?
                let worker_static: &'static mut GCWorker<VM> =
                    unsafe { std::mem::transmute(worker) };
                worker_static.do_work(process_nodes_work);
            } else {
                memory_manager::add_work_packet(mmtk, WorkBucketStage::Closure, process_nodes_work);
            }
        }

        #[cfg(feature = "sanity")]
        if self.roots {
            self.cache_roots_for_sanity_gc();
        }

        trace!("SimpleTracingProcessEdges End");
    }
}

struct SimpleTracingProcessNodes<
    VM: VMBinding,
    PlanType: Plan<VM = VM> + PlanTraceObject<VM>,
    const KIND: TraceKind,
> {
    context: SimpleTracingWorkContext<VM, PlanType, KIND>,
    nodes: Vec<ObjectReference>,
}

impl<VM: VMBinding, PlanType: Plan<VM = VM> + PlanTraceObject<VM>, const KIND: TraceKind> GCWork<VM>
    for SimpleTracingProcessNodes<VM, PlanType, KIND>
{
    fn do_work(&mut self, worker: &mut GCWorker<VM>, _mmtk: &'static MMTK<VM>) {
        trace!("SimpleTracingProcessNodes");
        let tls = worker.tls;

        // These nodes will be scanned later.
        let mut nodes_later = vec![];

        let mut edges_batcher = Batcher::new(4096, |edges| {
            worker.add_work(
                WorkBucketStage::Closure,
                self.context.create_process_edges_work(edges),
            );
        });

        for node in self.nodes.iter().copied() {
            if <VM as VMBinding>::VMScanning::support_edge_enqueuing(tls, node) {
                <VM as VMBinding>::VMScanning::scan_object(tls, node, &mut |edge| {
                    edges_batcher.push(edge);
                });
            } else {
                nodes_later.push(node);
            }
        }

        edges_batcher.finish();

        if !nodes_later.is_empty() {
            let mut new_nodes = VectorObjectQueue::new();

            for node in nodes_later {
                <VM as VMBinding>::VMScanning::scan_object_and_trace_edges(
                    tls,
                    node,
                    &mut |object| {
                        self.context
                            .plan
                            .trace_object::<_, KIND>(&mut new_nodes, object, worker)
                    },
                );
            }

            todo!("Create work packet for new_node");
        }

        trace!("SimpleTracingProcessNodes End");
    }
}
