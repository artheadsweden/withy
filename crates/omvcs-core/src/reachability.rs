//! Partial historical metadata reachability from currently supported roots.
//!
//! Core §62 has additional required root classes deliberately absent here. This
//! module cannot establish global unreachability, authorize deletion, or report
//! repository completeness. It never accesses Resource bytes or operational
//! Working State fields beyond their explicit historical safety references.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use omvcs_model::project_state::{AdmittedAdapterStateResolver, ComponentStateResolver};
use omvcs_model::revision::{AdmittedProjectStateResolver, AdmittedRevisionResolver};
use omvcs_model::{
    AdapterStateId, ComponentStateId, CreativeComponentId, LineId, ProjectId, ProjectStateId,
    ReleaseId, ResourceId, RevisionId,
};

use crate::line::{LineEnumerationBoundary, LineOperationError};
use crate::release::{ReleaseEnumerationBoundary, ReleaseOperationError};
use crate::working_state::{
    WorkingState, WorkingStateEnumerationBoundary, WorkingStateEnumerationError,
};

/// Trusted projection of the generic Resource edges of an admitted Adapter State.
///
/// Like `AdmittedAdapterStateResolver`, this consumes admission by generic Core
/// checks and the unique exact applicable Adapter schema/authority (Core §12;
/// DAW Adapter §§20–22). It does not define an Adapter State body or admission.
/// The returned identity MUST be the requested exact admitted object, and
/// `resources` MUST enumerate all its historical Resource references under that
/// exact schema. Implementations MUST NOT guess edges from opaque field names,
/// installed/latest Adapter versions, storage, or native working files.
pub trait AdmittedAdapterStateResourceResolver: AdmittedAdapterStateResolver {
    /// Returns the exact admitted object's identity and complete Resource IDs.
    ///
    /// `None` means its admitted metadata/edge projection is unresolved, NOT
    /// that it has no resources. An empty list is an affirmative no-edge result.
    /// No Resource bytes, Replicas, or materialisation are required.
    fn resolve_resource_ids(&self, id: AdapterStateId)
    -> Option<(AdapterStateId, Vec<ResourceId>)>;
}

/// Referenced historical object identity, ordered by type and then typed ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HistoricalId {
    /// Revision metadata.
    Revision(RevisionId),
    /// Project State metadata.
    ProjectState(ProjectStateId),
    /// Component State metadata.
    ComponentState(ComponentStateId),
    /// Adapter State metadata.
    AdapterState(AdapterStateId),
}

/// Enumerated root whose directly referenced Revision could not be resolved.
///
/// This records the existing Line target or admitted Release-to-Revision
/// association; it does not introduce a metadata edge kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReachabilityRoot {
    /// A retained Line and its current target Revision.
    Line(LineId),
    /// An admitted Release and its referenced Revision.
    Release(ReleaseId),
}

/// Context identifying an explicit historical reference in a current
/// persisted Working State. Project identity is context only; it is not a
/// fabricated historical Working State Identifier or edge kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum WorkingStateRoot {
    /// The Working State's present Base Revision.
    BaseRevision {
        /// Project owning the current Working State record.
        project_id: ProjectId,
    },
    /// A present source entry for one Creative Component.
    ComponentSource {
        /// Project owning the current Working State record.
        project_id: ProjectId,
        /// Component whose source entry contains the historical target.
        component_id: CreativeComponentId,
    },
}

/// One explicit historical safety reference from a currently persisted
/// Working State record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct WorkingStateRootReference {
    /// Project and operational field context supplying the reference.
    pub root: WorkingStateRoot,
    /// Referenced historical identifier.
    pub target: HistoricalId,
}

/// A root's direct Revision target whose admitted metadata was unresolved.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct UnresolvedRootReference {
    /// Enumerated Line or admitted Release providing the root.
    pub root: ReachabilityRoot,
    /// The directly referenced Revision identifier.
    pub target: RevisionId,
}

/// A current Working State safety-reference target whose admitted metadata
/// was unresolved. This records root context without a metadata edge kind or
/// a referring historical Identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct UnresolvedWorkingStateRootReference {
    /// Project and operational field context supplying the root reference.
    pub root: WorkingStateRoot,
    /// Referenced historical Identifier, retained even when metadata is absent.
    pub target: HistoricalId,
}

/// A defect is not an unresolved lookup and does not make an ID unreachable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReachabilityDefect {
    /// Resolver returned metadata under the wrong key; its edges were not followed.
    IdentifierMismatch {
        /// Requested historical ID.
        requested: HistoricalId,
        /// Returned historical ID.
        resolved: HistoricalId,
    },
    /// A Revision or asserted Component State parent path revisited an active node.
    Cycle {
        /// Historical ID revisited on the active path.
        object: HistoricalId,
    },
}

/// Explicitly partial result; every collection is sorted and deduplicated.
///
/// Reached IDs include unresolved or defective referenced IDs: an edge reaches
/// an identifier even if its metadata cannot be resolved. Such objects' unknown
/// outgoing edges cannot be followed. `unresolved` means only missing/unavailable
/// admitted historical metadata, never missing Resource bytes. Defects are
/// separate. Absence from these lists has NO global unreachability meaning.
///
/// Included roots are enumerated retained Lines, admitted Releases, and the
/// present Base Revision/component-source references in current persisted
/// Working State records. Contributions, configured archival pins, and
/// pending publication transactions remain excluded, even when every lookup
/// succeeds. Enumerations are individual reads, not a cross-boundary snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PartialReachability {
    /// Included retained Line roots; Default Line adds no second root.
    pub lines: Vec<LineId>,
    /// Included admitted Release roots.
    pub releases: Vec<ReleaseId>,
    /// Present Base Revision/source roots from current Working State records.
    pub working_state_roots: Vec<WorkingStateRootReference>,
    /// Revision IDs reached by roots and Revision parent edges.
    pub revisions: Vec<RevisionId>,
    /// Project State IDs referenced by reached resolved Revisions.
    pub project_states: Vec<ProjectStateId>,
    /// Component State IDs from Project States and asserted Component parents.
    pub component_states: Vec<ComponentStateId>,
    /// Adapter State IDs referenced by Project States.
    pub adapter_states: Vec<AdapterStateId>,
    /// Resource IDs from resolved Component and Adapter States; bytes unchecked.
    pub resources: Vec<ResourceId>,
    /// Referenced IDs whose admitted metadata could not be resolved.
    pub unresolved: Vec<HistoricalId>,
    /// Direct Line/Release root targets whose Revision metadata is unresolved.
    pub unresolved_root_references: Vec<UnresolvedRootReference>,
    /// Direct Working State safety-root targets whose metadata is unresolved.
    pub unresolved_working_state_root_references: Vec<UnresolvedWorkingStateRootReference>,
    /// Identity and graph defects, not unresolved references.
    pub defects: Vec<ReachabilityDefect>,
}

/// Failure to enumerate an included root class completely.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReachabilityError {
    /// Retained Line enumeration failed.
    Lines(LineOperationError),
    /// Admitted Release enumeration failed.
    Releases(ReleaseOperationError),
    /// Current persisted Working State enumeration failed.
    WorkingStates(WorkingStateEnumerationError),
}

impl fmt::Display for ReachabilityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lines(error) => write!(f, "Line enumeration failed: {error}"),
            Self::Releases(error) => write!(f, "Release enumeration failed: {error}"),
            Self::WorkingStates(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for ReachabilityError {}

/// Traverses approved historical edges from retained Lines, admitted Releases,
/// and current persisted Working State safety references.
///
/// Consumes trusted admitted immutable metadata, not import-validation candidates.
/// Uses the iterative active-path ancestry strategy from WORK-0009, but collects
/// independent branches and defects instead of the fail-fast `revision_ancestors`
/// API: a missing parent must not hide another parent's resources.
/// Only asserted Component State parentage is followed; omitted lineage is not
/// inferred. Resource IDs are terminal edges, with no availability lookup.
/// Operational Working State fields other than its Base Revision and present
/// source map entries are not read as roots.
///
/// # Errors
///
/// Returns an error if any included root class cannot be enumerated completely.
/// Metadata lookup failures and defects are retained in the partial result.
#[allow(clippy::too_many_lines)]
pub fn partial_repository_reachability(
    lines: &dyn LineEnumerationBoundary,
    releases: &dyn ReleaseEnumerationBoundary,
    working_states: &dyn WorkingStateEnumerationBoundary,
    revisions: &dyn AdmittedRevisionResolver,
    project_states: &dyn AdmittedProjectStateResolver,
    component_states: &dyn ComponentStateResolver,
    adapter_states: &dyn AdmittedAdapterStateResourceResolver,
) -> Result<PartialReachability, ReachabilityError> {
    let lines = lines.retained_lines().map_err(ReachabilityError::Lines)?;
    let releases = releases
        .admitted_releases()
        .map_err(ReachabilityError::Releases)?;
    let working_states = working_states
        .current_working_states()
        .map_err(ReachabilityError::WorkingStates)?;
    let root_references = collect_root_references(&lines, &releases);
    let working_state_root_references = collect_working_state_root_references(&working_states);
    let mut roots: BTreeSet<_> = root_references.iter().map(|(_, target)| *target).collect();
    let mut working_state_component_roots = BTreeSet::new();
    for (_, target) in &working_state_root_references {
        match target {
            WorkingStateRootTarget::BaseRevision(id) => {
                roots.insert(*id);
            }
            WorkingStateRootTarget::ComponentSource(id) => {
                working_state_component_roots.insert(*id);
            }
        }
    }
    let mut unresolved = BTreeSet::new();
    let mut defects = BTreeSet::new();
    let revision_graph = walk(
        roots,
        |id| {
            revisions
                .resolve_admitted(id)
                .map(|node| (node.revision_id(), node.parents().to_vec(), node))
        },
        HistoricalId::Revision,
        &mut unresolved,
        &mut defects,
    );
    let mut reached_projects = BTreeSet::new();
    let mut reached_components = working_state_component_roots;
    let mut reached_adapters = BTreeSet::new();
    for revision in revision_graph.nodes.values() {
        reached_projects.insert(revision.project_state_id());
    }
    for &id in &reached_projects {
        let Some(state) = project_states.resolve_admitted(id) else {
            unresolved.insert(HistoricalId::ProjectState(id));
            continue;
        };
        if state.project_state_id() != id {
            defects.insert(ReachabilityDefect::IdentifierMismatch {
                requested: HistoricalId::ProjectState(id),
                resolved: HistoricalId::ProjectState(state.project_state_id()),
            });
            continue;
        }
        reached_components.extend(state.components().values().copied());
        reached_adapters.insert(state.adapter_state_id());
    }
    let component_graph = walk(
        reached_components,
        |id| {
            component_states.resolve_admitted(id).map(|node| {
                (
                    node.component_state_id(),
                    node.parents().unwrap_or_default().to_vec(),
                    node,
                )
            })
        },
        HistoricalId::ComponentState,
        &mut unresolved,
        &mut defects,
    );
    let mut resources = BTreeSet::new();
    for state in component_graph.nodes.values() {
        resources.extend(
            state
                .resources()
                .iter()
                .map(omvcs_model::resource::ResourceReference::resource_id),
        );
    }
    collect_adapter_resources(
        &reached_adapters,
        adapter_states,
        &mut resources,
        &mut unresolved,
        &mut defects,
    );
    let unresolved_root_references = root_references
        .iter()
        .filter(|(_, target)| unresolved.contains(&HistoricalId::Revision(*target)))
        .map(|(root, target)| UnresolvedRootReference {
            root: *root,
            target: *target,
        })
        .collect();
    let unresolved_working_state_root_references = working_state_root_references
        .iter()
        .filter_map(|(root, target)| {
            let target_id = target.historical_id();
            unresolved
                .contains(&target_id)
                .then_some(UnresolvedWorkingStateRootReference {
                    root: *root,
                    target: target_id,
                })
        })
        .collect();
    let working_state_roots = working_state_root_references
        .iter()
        .map(|(root, target)| WorkingStateRootReference {
            root: *root,
            target: target.historical_id(),
        })
        .collect();
    Ok(PartialReachability {
        lines: sorted(lines.iter().map(crate::line::Line::line_id)),
        releases: sorted(releases.iter().map(omvcs_model::Release::release_id)),
        working_state_roots,
        revisions: revision_graph.reached.into_iter().collect(),
        project_states: reached_projects.into_iter().collect(),
        component_states: component_graph.reached.into_iter().collect(),
        adapter_states: reached_adapters.into_iter().collect(),
        resources: resources.into_iter().collect(),
        unresolved: unresolved.into_iter().collect(),
        unresolved_root_references,
        unresolved_working_state_root_references,
        defects: defects.into_iter().collect(),
    })
}

fn collect_root_references(
    lines: &[crate::line::Line],
    releases: &[omvcs_model::Release],
) -> Vec<(ReachabilityRoot, RevisionId)> {
    lines
        .iter()
        .map(|line| {
            (
                ReachabilityRoot::Line(line.line_id()),
                line.target_revision(),
            )
        })
        .chain(releases.iter().map(|release| {
            (
                ReachabilityRoot::Release(release.release_id()),
                release.revision_id(),
            )
        }))
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum WorkingStateRootTarget {
    BaseRevision(RevisionId),
    ComponentSource(ComponentStateId),
}

impl WorkingStateRootTarget {
    const fn historical_id(self) -> HistoricalId {
        match self {
            Self::BaseRevision(id) => HistoricalId::Revision(id),
            Self::ComponentSource(id) => HistoricalId::ComponentState(id),
        }
    }
}

fn collect_working_state_root_references(
    working_states: &[WorkingState],
) -> Vec<(WorkingStateRoot, WorkingStateRootTarget)> {
    let mut roots = BTreeSet::new();
    for working_state in working_states {
        if let Some(id) = working_state.base_revision_id() {
            roots.insert((
                WorkingStateRoot::BaseRevision {
                    project_id: working_state.project_id(),
                },
                WorkingStateRootTarget::BaseRevision(id),
            ));
        }
        for (&component_id, source) in working_state.component_sources() {
            if let Some(id) = source {
                roots.insert((
                    WorkingStateRoot::ComponentSource {
                        project_id: working_state.project_id(),
                        component_id,
                    },
                    WorkingStateRootTarget::ComponentSource(*id),
                ));
            }
        }
    }
    roots.into_iter().collect()
}

fn collect_adapter_resources(
    reached: &BTreeSet<AdapterStateId>,
    adapters: &dyn AdmittedAdapterStateResourceResolver,
    resources: &mut BTreeSet<ResourceId>,
    unresolved: &mut BTreeSet<HistoricalId>,
    defects: &mut BTreeSet<ReachabilityDefect>,
) {
    for &id in reached {
        // The projection must also meet the existing admitted-identity contract.
        let Some(admitted) = adapters.resolve_admitted(id) else {
            unresolved.insert(HistoricalId::AdapterState(id));
            continue;
        };
        if admitted != id {
            defects.insert(ReachabilityDefect::IdentifierMismatch {
                requested: HistoricalId::AdapterState(id),
                resolved: HistoricalId::AdapterState(admitted),
            });
            continue;
        }
        match adapters.resolve_resource_ids(id) {
            None => {
                unresolved.insert(HistoricalId::AdapterState(id));
            }
            Some((resolved, _)) if resolved != id => {
                defects.insert(ReachabilityDefect::IdentifierMismatch {
                    requested: HistoricalId::AdapterState(id),
                    resolved: HistoricalId::AdapterState(resolved),
                });
            }
            Some((_, ids)) => resources.extend(ids),
        }
    }
}

fn sorted<T: Ord>(ids: impl Iterator<Item = T>) -> Vec<T> {
    ids.collect::<BTreeSet<_>>().into_iter().collect()
}

struct Graph<I, N> {
    reached: BTreeSet<I>,
    nodes: BTreeMap<I, N>,
}

// Enter/exit events avoid recursive stack growth. Only active-path repetition
// is a cycle; convergence onto a completed node is normal. Missing/mismatched
// IDs are still visited once, so independent branches continue and terminate.
fn walk<I: Ord + Copy, N>(
    roots: BTreeSet<I>,
    mut resolve: impl FnMut(I) -> Option<(I, Vec<I>, N)>,
    historical: impl Fn(I) -> HistoricalId,
    unresolved: &mut BTreeSet<HistoricalId>,
    defects: &mut BTreeSet<ReachabilityDefect>,
) -> Graph<I, N> {
    let mut reached = BTreeSet::new();
    let mut nodes = BTreeMap::new();
    let mut active = BTreeSet::new();
    let mut stack: Vec<_> = roots.into_iter().rev().map(|id| (id, false)).collect();
    while let Some((id, exit)) = stack.pop() {
        if exit {
            active.remove(&id);
            continue;
        }
        if active.contains(&id) {
            defects.insert(ReachabilityDefect::Cycle {
                object: historical(id),
            });
            continue;
        }
        if !reached.insert(id) {
            continue;
        }
        let Some((resolved, parents, node)) = resolve(id) else {
            unresolved.insert(historical(id));
            continue;
        };
        if resolved != id {
            defects.insert(ReachabilityDefect::IdentifierMismatch {
                requested: historical(id),
                resolved: historical(resolved),
            });
            continue;
        }
        nodes.insert(id, node);
        active.insert(id);
        stack.push((id, true));
        // Stable parent order also makes cycle diagnostics deterministic.
        stack.extend(
            parents
                .into_iter()
                .collect::<BTreeSet<_>>()
                .into_iter()
                .rev()
                .map(|parent| (parent, false)),
        );
    }
    Graph { reached, nodes }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Synthetic graph inputs isolate termination: cycles cannot be constructed
    // through the admitted content-addressed Revision API.
    #[test]
    fn malformed_parent_cycles_terminate_and_leave_independent_branches_reached() {
        let first = RevisionId::from_digest([1; 32]);
        let second = RevisionId::from_digest([2; 32]);
        let independent = RevisionId::from_digest([3; 32]);
        let nodes = BTreeMap::from([
            (first, vec![second, independent]),
            (second, vec![first]),
            (independent, vec![]),
        ]);
        let mut unresolved = BTreeSet::new();
        let mut defects = BTreeSet::new();
        let graph = walk(
            BTreeSet::from([first]),
            |id| nodes.get(&id).map(|parents| (id, parents.clone(), ())),
            HistoricalId::Revision,
            &mut unresolved,
            &mut defects,
        );
        assert_eq!(graph.reached, BTreeSet::from([first, second, independent]));
        assert!(unresolved.is_empty());
        assert_eq!(
            defects,
            BTreeSet::from([ReachabilityDefect::Cycle {
                object: HistoricalId::Revision(first),
            }])
        );
    }

    #[test]
    fn malformed_component_parent_cycles_terminate_without_fabricating_unresolved_ids() {
        let first = ComponentStateId::from_digest([1; 32]);
        let second = ComponentStateId::from_digest([2; 32]);
        let mut unresolved = BTreeSet::new();
        let mut defects = BTreeSet::new();
        let graph = walk(
            BTreeSet::from([first]),
            |id| Some((id, vec![if id == first { second } else { first }], ())),
            HistoricalId::ComponentState,
            &mut unresolved,
            &mut defects,
        );
        assert_eq!(graph.reached.len(), 2);
        assert_eq!(graph.nodes.len(), 2);
        assert!(unresolved.is_empty());
        assert_eq!(
            defects,
            BTreeSet::from([ReachabilityDefect::Cycle {
                object: HistoricalId::ComponentState(first),
            }])
        );
    }

    #[test]
    fn deep_parent_history_is_traversed_without_recursive_stack_growth() {
        let id = |number: u32| {
            let mut digest = [0; 32];
            digest[..4].copy_from_slice(&number.to_be_bytes());
            RevisionId::from_digest(digest)
        };
        let mut unresolved = BTreeSet::new();
        let mut defects = BTreeSet::new();
        let graph = walk(
            BTreeSet::from([id(0)]),
            |requested| {
                let mut bytes = [0; 4];
                bytes.copy_from_slice(&requested.digest()[..4]);
                let number = u32::from_be_bytes(bytes);
                Some((
                    requested,
                    if number < 9_999 {
                        vec![id(number + 1)]
                    } else {
                        vec![]
                    },
                    (),
                ))
            },
            HistoricalId::Revision,
            &mut unresolved,
            &mut defects,
        );
        assert_eq!(graph.reached.len(), 10_000);
        assert!(unresolved.is_empty());
        assert!(defects.is_empty());
    }
}
