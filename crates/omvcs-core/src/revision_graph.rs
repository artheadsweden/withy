//! Read-only traversal of admitted Revision ancestry.
//!
//! This operation follows only the typed parent references in admitted
//! Revision metadata. It neither resolves nor materializes Resource bytes.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use omvcs_model::revision::AdmittedRevisionResolver;
use omvcs_model::{Revision, RevisionId};

/// Returns each direct and transitive ancestor of `revision_id` once.
///
/// The requested identifier and every parent identifier are resolved through
/// the admitted Revision boundary. A missing object makes this traversal
/// unresolved; a resolver result with a different identifier or a cycle is an
/// invalid graph structure. WORK-0008 admission guarantees typed, unique,
/// same-Project parent references, so this function relies on that guarantee
/// while defensively checking the identity returned for each lookup.
///
/// The returned identifiers are sorted to make equivalent traversals stable.
/// Their order has no ancestry meaning. The starting Revision is not included.
/// No Resource-byte availability is checked or implied by success.
///
/// # Errors
///
/// Returns [`RevisionGraphError::UnresolvedRevision`] if the starting
/// Revision or any Revision in its requested parent closure cannot be
/// resolved. Returns [`RevisionGraphError::InvalidStructure`] for a cycle or
/// a resolver result whose identifier does not match the requested identifier.
pub fn revision_ancestors(
    revision_id: RevisionId,
    revisions: &dyn AdmittedRevisionResolver,
) -> Result<Vec<RevisionId>, RevisionGraphError> {
    traverse_ancestry(revision_id, |requested| {
        revisions
            .resolve_admitted(requested)
            .map(ResolvedNode::from)
    })
}

/// A graph-local outcome for ancestry traversal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RevisionGraphError {
    /// Required admitted Revision metadata could not be resolved.
    UnresolvedRevision(RevisionId),
    /// The resolved closure contains invalid graph structure.
    InvalidStructure(RevisionGraphDefect),
}

impl fmt::Display for RevisionGraphError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnresolvedRevision(id) => {
                write!(formatter, "Revision metadata is unresolved: {id}")
            }
            Self::InvalidStructure(defect) => {
                write!(formatter, "invalid Revision graph structure: {defect}")
            }
        }
    }
}

impl std::error::Error for RevisionGraphError {}

/// A detected defect in the requested Revision graph closure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RevisionGraphDefect {
    /// A resolver returned an admitted object whose identity differs from its key.
    IdentifierMismatch {
        /// Identifier requested from the resolver.
        requested: RevisionId,
        /// Identifier reported by the returned Revision.
        resolved: RevisionId,
    },
    /// A parent path returned to a Revision already active in the traversal.
    Cycle {
        /// Identifier whose active path was encountered again.
        revision_id: RevisionId,
    },
}

impl fmt::Display for RevisionGraphDefect {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IdentifierMismatch {
                requested,
                resolved,
            } => {
                write!(
                    formatter,
                    "requested Revision {requested}, resolver returned {resolved}"
                )
            }
            Self::Cycle { revision_id } => {
                write!(formatter, "parent path returns to Revision {revision_id}")
            }
        }
    }
}

#[derive(Debug, Clone)]
struct ResolvedNode {
    id: RevisionId,
    parents: Vec<RevisionId>,
}

impl From<&Revision> for ResolvedNode {
    fn from(revision: &Revision) -> Self {
        Self {
            id: revision.revision_id(),
            parents: revision.parents().to_vec(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VisitState {
    Visiting,
    Complete,
}

struct Frame {
    id: RevisionId,
    parents: Vec<RevisionId>,
    next_parent: usize,
}

fn traverse_ancestry(
    root: RevisionId,
    mut resolve: impl FnMut(RevisionId) -> Option<ResolvedNode>,
) -> Result<Vec<RevisionId>, RevisionGraphError> {
    let root_node = resolve_checked(root, &mut resolve)?;
    let mut states = BTreeMap::from([(root, VisitState::Visiting)]);
    let mut ancestors = BTreeSet::new();
    let mut stack = vec![Frame {
        id: root,
        parents: root_node.parents,
        next_parent: 0,
    }];

    while let Some(frame) = stack.last_mut() {
        if frame.next_parent == frame.parents.len() {
            let completed_id = frame.id;
            stack.pop();
            states.insert(completed_id, VisitState::Complete);
            if completed_id != root {
                ancestors.insert(completed_id);
            }
            continue;
        }

        let parent_id = frame.parents[frame.next_parent];
        frame.next_parent += 1;

        match states.get(&parent_id) {
            Some(VisitState::Visiting) => {
                return Err(RevisionGraphError::InvalidStructure(
                    RevisionGraphDefect::Cycle {
                        revision_id: parent_id,
                    },
                ));
            }
            Some(VisitState::Complete) => continue,
            None => {}
        }

        let parent = resolve_checked(parent_id, &mut resolve)?;
        states.insert(parent_id, VisitState::Visiting);
        stack.push(Frame {
            id: parent_id,
            parents: parent.parents,
            next_parent: 0,
        });
    }

    Ok(ancestors.into_iter().collect())
}

fn resolve_checked(
    requested: RevisionId,
    resolve: &mut impl FnMut(RevisionId) -> Option<ResolvedNode>,
) -> Result<ResolvedNode, RevisionGraphError> {
    let node = resolve(requested).ok_or(RevisionGraphError::UnresolvedRevision(requested))?;
    if node.id != requested {
        return Err(RevisionGraphError::InvalidStructure(
            RevisionGraphDefect::IdentifierMismatch {
                requested,
                resolved: node.id,
            },
        ));
    }
    Ok(node)
}

#[cfg(test)]
mod tests {
    use super::{ResolvedNode, RevisionGraphDefect, RevisionGraphError, traverse_ancestry};
    use omvcs_model::RevisionId;
    use std::collections::BTreeMap;

    fn id(value: u8) -> RevisionId {
        RevisionId::from_digest([value; 32])
    }

    // Synthetic edges isolate the cycle detector; this is not an admitted
    // Revision fixture and makes no claim that a cyclic Revision is admissible.
    #[test]
    fn cyclic_synthetic_graph_is_rejected_as_invalid_structure() {
        let first = id(1);
        let second = id(2);
        let nodes = BTreeMap::from([
            (
                first,
                ResolvedNode {
                    id: first,
                    parents: vec![second],
                },
            ),
            (
                second,
                ResolvedNode {
                    id: second,
                    parents: vec![first],
                },
            ),
        ]);

        assert_eq!(
            traverse_ancestry(first, |requested| nodes.get(&requested).cloned()),
            Err(RevisionGraphError::InvalidStructure(
                RevisionGraphDefect::Cycle { revision_id: first }
            ))
        );
    }

    #[test]
    fn resolver_identifier_mismatch_is_invalid_structure() {
        let requested = id(1);
        let resolved = id(2);

        assert_eq!(
            traverse_ancestry(requested, |_| {
                Some(ResolvedNode {
                    id: resolved,
                    parents: vec![],
                })
            }),
            Err(RevisionGraphError::InvalidStructure(
                RevisionGraphDefect::IdentifierMismatch {
                    requested,
                    resolved
                }
            ))
        );
    }
}
