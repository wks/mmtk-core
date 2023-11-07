//! This module contains code useful for tracing,
//! i.e. visiting the reachable objects by traversing all or part of an object graph.

use crate::scheduler::gc_work::{EdgeOf, ProcessEdgesWork, ProcessSlicesWork, SliceOf};
use crate::scheduler::{GCWorker, WorkBucketStage};
use crate::util::ObjectReference;
use crate::vm::EdgeVisitor;

/// This trait represents an object queue to enqueue objects during tracing.
pub trait ObjectQueue {
    /// Enqueue an object into the queue.
    fn enqueue(&mut self, object: ObjectReference);
}

pub type VectorObjectQueue = VectorQueue<ObjectReference>;

/// An implementation of `ObjectQueue` using a `Vec`.
///
/// This can also be used as a buffer. For example, the mark stack or the write barrier mod-buffer.
pub struct VectorQueue<T> {
    /// Enqueued nodes.
    buffer: Vec<T>,
}

impl<T> VectorQueue<T> {
    /// Reserve a capacity of this on first enqueue to avoid frequent resizing.
    const CAPACITY: usize = 4096;

    /// Create an empty `VectorObjectQueue`.
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    /// Return `true` if the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Return the contents of the underlying vector.  It will empty the queue.
    pub fn take(&mut self) -> Vec<T> {
        std::mem::take(&mut self.buffer)
    }

    /// Consume this `VectorObjectQueue` and return its underlying vector.
    pub fn into_vec(self) -> Vec<T> {
        self.buffer
    }

    /// Check if the buffer size reaches `CAPACITY`.
    pub fn is_full(&self) -> bool {
        self.buffer.len() >= Self::CAPACITY
    }

    pub fn push(&mut self, v: T) {
        if self.buffer.is_empty() {
            self.buffer.reserve(Self::CAPACITY);
        }
        self.buffer.push(v);
    }
}

impl<T> Default for VectorQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl ObjectQueue for VectorQueue<ObjectReference> {
    fn enqueue(&mut self, v: ObjectReference) {
        self.push(v);
    }
}

/// A transitive closure visitor to collect all the edges of an object.
pub struct ObjectsClosure<'a, E: ProcessEdgesWork> {
    edge_buffer: Vec<EdgeOf<E>>,
    slice_buffer: Vec<SliceOf<E>>,
    slice_buffer_edge_count: usize,
    pub(crate) worker: &'a mut GCWorker<E::VM>,
    bucket: WorkBucketStage,
}

impl<'a, E: ProcessEdgesWork> ObjectsClosure<'a, E> {
    pub const DEFAULT_EDGES_PER_PACKET: usize = 4096;

    pub fn new(worker: &'a mut GCWorker<E::VM>, bucket: WorkBucketStage) -> Self {
        Self {
            edge_buffer: Vec::new(),
            slice_buffer: Vec::new(),
            slice_buffer_edge_count: 0,
            worker,
            bucket,
        }
    }

    fn flush(&mut self) {
        self.flush_edges();
        self.flush_slices();
    }

    fn edge_buffer_is_full(&self) -> bool {
        self.edge_buffer.len() >= E::CAPACITY
    }

    fn slice_buffer_is_full(&self) -> bool {
        self.slice_buffer_edge_count >= E::CAPACITY
    }

    fn flush_edges(&mut self) {
        if !self.edge_buffer.is_empty() {
            let buf = std::mem::take(&mut self.edge_buffer);
            self.worker.add_work(
                self.bucket,
                E::new(buf, false, self.worker.mmtk, self.bucket),
            );
        }
    }

    fn flush_slices(&mut self) {
        if !self.slice_buffer.is_empty() {
            let buf = std::mem::take(&mut self.slice_buffer);
            self.worker.add_work(
                self.bucket,
                ProcessSlicesWork::<E>::new(buf, false, self.worker.mmtk, self.bucket),
            );
        }
    }
}

impl<'a, E: ProcessEdgesWork> EdgeVisitor<E::VM> for ObjectsClosure<'a, E> {
    fn visit_edge(&mut self, slot: EdgeOf<E>) {
        #[cfg(debug_assertions)]
        {
            use crate::vm::edge_shape::Edge;
            trace!(
                "(ObjectsClosure) Visit edge {:?} (pointing to {})",
                slot,
                slot.load()
            );
        }
        self.edge_buffer.push(slot);
        if self.edge_buffer_is_full() {
            self.flush();
        }
    }

    fn visit_slice(&mut self, slice: SliceOf<E>) {
        #[cfg(debug_assertions)]
        {
            trace!(
                "(ObjectsClosure) Visit slice {:?}",
                slice,
            );
        }

        todo!("Need a way to split slice into sub-slices")
    }
}

impl<'a, E: ProcessEdgesWork> Drop for ObjectsClosure<'a, E> {
    fn drop(&mut self) {
        self.flush();
    }
}
