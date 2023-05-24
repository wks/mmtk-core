//! This module contains data structures for recording events in a heap traversal.

use crate::util::ObjectReference;

/// An event in tracing (i.e. computing transitive closure from roots).
#[derive(Debug)]
pub enum Record {
    /// Visiting an object from a root.
    Root {
        /// The object.
        objref: ObjectReference,
        /// If the root pins the object.
        pinning: bool,
        /// `false` if `to` does not point to a valid object.
        valid: bool,
    },
    /// Visiting an object.
    Node {
        /// The object reference of the object, in the to-space.
        objref: ObjectReference,
        /// If the object is actually pinned.
        pinned: bool,
        /// Optional string to represent the type.
        type_string: Option<String>,
        /// Optional string to represent detail information.
        comment: Option<String>,
    },
    /// Visiting a reference field of an object.
    Edge {
        /// The object that contains the field.
        from: ObjectReference,
        /// The content of the field.
        to: ObjectReference,
        /// `false` if `to` does not point to a valid object.
        valid: bool,
    },
    /// An object is moved from `from` to `to`.
    Forward {
        /// The old address (in the from-space).
        from: ObjectReference,
        /// The new address (in the to-space).
        to: ObjectReference,
    },
    /// An object is resurrected due to weak reference or finalization processing.
    Resurrect {
        /// The object (in the from-space).  If an object is resurrected, it will also generate
        /// a `Node` record for the to-space object and a `Forward` event if the object is moved.
        objref: ObjectReference,
    },
}
