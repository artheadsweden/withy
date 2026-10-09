//! OMVCS Core reference implementation.
//!
//! Production semantics must come from the normative Specs and approved work packages.

#![forbid(unsafe_code)]

pub mod line;
pub mod reachability;
pub mod release;
pub mod revision_graph;
pub mod working_state;
