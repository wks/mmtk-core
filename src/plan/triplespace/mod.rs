//! Plan: triplespace: Three-space generational GC

pub(super) mod global;
pub(super) mod mutator;
pub(super) mod gc_work;

pub use self::global::TripleSpace;
pub use self::global::TS_CONSTRAINTS;
