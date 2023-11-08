//! This module contains code useful for tracing,
//! i.e. visiting the reachable objects by traversing all or part of an object graph.

use crate::scheduler::gc_work::{EdgeOf, ProcessEdgesWork, ProcessSlicesWork, SliceOf};
use crate::scheduler::{GCWorker, WorkBucketStage};
use crate::util::ObjectReference;
use crate::vm::edge_shape::MemorySlice;
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

/// A specialized buffer type for slices.  It counts the number of edges in all slices instead of
/// the number of slices themselves.
pub struct SliceBuffer<S: MemorySlice> {
    slices: Vec<S>,
    cur_edges: usize,
    max_edges: usize,
}

impl<S: MemorySlice> Default for SliceBuffer<S> {
    fn default() -> Self {
        Self::new(Self::DEFAULT_CAPACITY)
    }
}

impl<S: MemorySlice> SliceBuffer<S> {
    pub const DEFAULT_CAPACITY: usize = 4096;

    pub fn new(max_edges: usize) -> Self {
        Self {
            slices: Vec::new(),
            cur_edges: 0,
            max_edges,
        }
    }

    /// Add `slice` into the `SliceBuffer`.  If `slice` is larger than `max_edges`, it will be
    /// split into smaller slices not larger than `max_edges`.  This function may flush this
    /// `SliceBuffer` multiple times by calling the `flusher` callback, yielding one `Vec<S>` each
    /// time.  The total number of *edges* in each `Vec<S>` yielded will be between `max_edges / 2`
    /// and `max_edges`.
    ///
    /// The user usually creates a `ProcessSlicesWork` packet for the `Vec<S>` every time this
    /// function yields.  The yielding rule means the work packet may have between `max_edges / 2`
    /// and `max_edges` edges, making the time for executing the work packet bounded.
    pub fn push_with_flush(&mut self, slice: S, mut flusher: impl FnMut(Vec<S>)) {
        let max_edges = self.max_edges;
        let half_edges = max_edges / 2;

        for chunk in slice.chunks(self.max_edges) {
            let chunk_len = chunk.len();
            if chunk_len >= half_edges {
                // If the chunk (a sub-slice) is large enough, we make a dedicated work packet for
                // it by yielding a `Vec` containing a single slice (this `chunk`).
                // All chunks except the last one should be exactly `max_edges` in length.
                // If the last chunk is still at least `half_edges` in length,
                // we consider it large enough, and create a dedicate work packet for it, too.
                flusher(vec![chunk]);
            } else {
                // If the last chunk is not large enough, we try to save it in `self.slices`.
                if self.cur_edges + chunk_len >= max_edges {
                    // If it doesn't fit, then `self.slices` must have already had at least
                    // `half_edges` edges in total.  We flush it.
                    self.flush_inner(&mut flusher);
                }
                // Now `self.slices` must have enough room for the new chunk.
                self.slices.push(chunk);
                self.cur_edges += chunk_len;
            }
        }
    }

    /// Flush the `SliceBuffer`.  If it is empty, it will call `flusher` with all currently pushed
    /// slices.
    pub fn flush(&mut self, mut flusher: impl FnMut(Vec<S>)) {
        if !self.slices.is_empty() {
            self.flush_inner(&mut flusher);
        } else {
            debug_assert_eq!(self.cur_edges, 0);
        }
    }

    fn flush_inner(&mut self, flusher: &mut impl FnMut(Vec<S>)) {
        let slices = std::mem::take(&mut self.slices);
        flusher(slices);
        self.cur_edges = 0;
    }
}

/// A transitive closure visitor to collect all the slots (edges) of an object.
/// It also collect slices of slots.
pub struct ObjectsClosure<'a, E: ProcessEdgesWork> {
    edge_buffer: Vec<EdgeOf<E>>,
    slice_buffer: SliceBuffer<SliceOf<E>>,
    pub(crate) worker: &'a mut GCWorker<E::VM>,
    bucket: WorkBucketStage,
}

impl<'a, E: ProcessEdgesWork> ObjectsClosure<'a, E> {
    pub fn new(worker: &'a mut GCWorker<E::VM>, bucket: WorkBucketStage) -> Self {
        Self {
            edge_buffer: Vec::new(),
            slice_buffer: SliceBuffer::new(E::CAPACITY),
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
        self.slice_buffer.flush(|slices| {
            self.worker.add_work(
                self.bucket,
                ProcessSlicesWork::<E>::new(slices, false, self.worker.mmtk, self.bucket),
            );
        })
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
                "(ObjectsClosure) Visit slice {:?}, len: {}",
                slice,
                slice.len()
            );
        }

        let mut packets = vec![];

        self.slice_buffer.push_with_flush(slice, |slices| {
            let packet = ProcessSlicesWork::<E>::new(slices, false, self.worker.mmtk, self.bucket);
            packets.push(Box::new(packet) as _);
        });

        if !packets.is_empty() {
            self.worker.scheduler().work_buckets[self.bucket].bulk_add(packets);
        }
    }
}

impl<'a, E: ProcessEdgesWork> Drop for ObjectsClosure<'a, E> {
    fn drop(&mut self) {
        self.flush();
    }
}
