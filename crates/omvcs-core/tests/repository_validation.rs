#![allow(clippy::expect_used)]

use std::cell::Cell;
use std::collections::BTreeMap;

use omvcs_core::reachability::{
    HistoricalId, PartialReachability, ReachabilityRoot, UnresolvedRootReference,
};
use omvcs_core::repository_validation::{
    CoverageStatus, DeclaredBoundaryLookup, DeclaredBoundaryQuery, HistoryCompleteness,
    InvocationError, MetadataAdmissionStatus, MetadataBodyFailure, MetadataCandidate,
    MetadataEdgeKind, MetadataEnumeration, MetadataIdentifier, MetadataIntegrity, ProviderFailure,
    RepositoryValidationBoundary, RequiredMetadataReference, ResolvedMetadata, ResourceState,
    ResourceVerification, ResourceVerificationDepth, RootProvider, ValidationFinding,
    ValidationRequest, ValidationScope, VerificationStrength, validate_repository,
};
use omvcs_model::{LineId, ProjectId, ProjectStateId, ReleaseId, ResourceId, RevisionId};

struct Fixture {
    enumeration: Result<MetadataEnumeration, ProviderFailure>,
    resolved: BTreeMap<MetadataIdentifier, Result<Option<ResolvedMetadata>, ProviderFailure>>,
    boundaries: BTreeMap<DeclaredBoundaryQuery, Result<DeclaredBoundaryLookup, ProviderFailure>>,
    resources: BTreeMap<ResourceId, Result<ResourceVerification, ProviderFailure>>,
    reachability: Result<PartialReachability, ProviderFailure>,
    context_error: Option<InvocationError>,
    verification_calls: Cell<usize>,
    boundary_queries: Cell<usize>,
}

fn line_id() -> LineId {
    "019cc17d-1b22-7a41-9fe9-c345c468f82f"
        .parse()
        .expect("LineId")
}

#[test]
fn unresolved_line_target_is_reported_and_blocks_valid_complete() {
    let target = RevisionId::from_digest([12; 32]);
    let root = line_id();
    let mut fixture = Fixture::new(vec![candidate(
        revision(13),
        Some(project(1)),
        Vec::new(),
        Vec::new(),
    )]);
    fixture.reachability = Ok(PartialReachability {
        lines: vec![root],
        revisions: vec![target],
        unresolved: vec![HistoricalId::Revision(target)],
        unresolved_root_references: vec![UnresolvedRootReference {
            root: ReachabilityRoot::Line(root),
            target,
        }],
        ..PartialReachability::default()
    });

    let report = fixture
        .validate(
            ValidationScope::Repository,
            ResourceVerificationDepth::MetadataOnly,
        )
        .expect("unresolved root remains a completed report");
    assert_eq!(report.metadata_integrity, MetadataIntegrity::Indeterminate);
    assert_eq!(report.history_completeness, HistoryCompleteness::Unresolved);
    assert!(report.findings.iter().any(|finding| matches!(
        finding,
        ValidationFinding::UnresolvedRootReference {
            root: ReachabilityRoot::Line(actual_root),
            target: MetadataIdentifier::Revision(actual_target),
        } if *actual_root == root && *actual_target == target
    )));
    assert!(!report.findings.iter().any(|finding| matches!(
        finding,
        ValidationFinding::UnresolvedReachabilityMetadata {
            target: MetadataIdentifier::Revision(actual_target),
        } if *actual_target == target
    )));
}

#[test]
fn unresolved_release_target_is_reported_at_the_report_level() {
    let target = RevisionId::from_digest([15; 32]);
    let root = ReleaseId::from_digest([16; 32]);
    let mut fixture = Fixture::new(vec![candidate(
        revision(17),
        Some(project(1)),
        Vec::new(),
        Vec::new(),
    )]);
    fixture.reachability = Ok(PartialReachability {
        releases: vec![root],
        revisions: vec![target],
        unresolved: vec![HistoricalId::Revision(target)],
        unresolved_root_references: vec![UnresolvedRootReference {
            root: ReachabilityRoot::Release(root),
            target,
        }],
        ..PartialReachability::default()
    });

    let report = fixture
        .validate(
            ValidationScope::Repository,
            ResourceVerificationDepth::MetadataOnly,
        )
        .expect("unresolved Release root is a report finding");
    assert_eq!(report.metadata_integrity, MetadataIntegrity::Indeterminate);
    assert_eq!(report.history_completeness, HistoryCompleteness::Unresolved);
    assert!(report.findings.iter().any(|finding| matches!(
        finding,
        ValidationFinding::UnresolvedRootReference {
            root: ReachabilityRoot::Release(actual_root),
            target: MetadataIdentifier::Revision(actual_target),
        } if *actual_root == root && *actual_target == target
    )));
}

#[test]
fn unresolved_reachability_without_root_context_has_a_generic_report_finding() {
    let target = omvcs_model::ComponentStateId::from_digest([18; 32]);
    let mut fixture = Fixture::new(vec![candidate(
        revision(19),
        Some(project(1)),
        Vec::new(),
        Vec::new(),
    )]);
    fixture.reachability = Ok(PartialReachability {
        component_states: vec![target],
        unresolved: vec![HistoricalId::ComponentState(target)],
        ..PartialReachability::default()
    });

    let report = fixture
        .validate(
            ValidationScope::Repository,
            ResourceVerificationDepth::MetadataOnly,
        )
        .expect("unresolved reachability is a report finding");
    assert_eq!(report.metadata_integrity, MetadataIntegrity::Indeterminate);
    assert_eq!(report.history_completeness, HistoryCompleteness::Unresolved);
    assert!(report.findings.iter().any(|finding| matches!(
        finding,
        ValidationFinding::UnresolvedReachabilityMetadata {
            target: MetadataIdentifier::ComponentState(actual),
        } if *actual == target
    )));
}

#[test]
fn partial_required_root_coverage_cannot_claim_complete_history() {
    let report = Fixture::new(vec![candidate(
        revision(14),
        Some(project(1)),
        Vec::new(),
        Vec::new(),
    )])
    .validate(
        ValidationScope::Repository,
        ResourceVerificationDepth::MetadataOnly,
    )
    .expect("partial root coverage remains a completed report");

    assert!(
        report
            .coverage
            .required_roots
            .iter()
            .any(|provider| provider.status != CoverageStatus::Complete)
    );
    assert_ne!(report.history_completeness, HistoryCompleteness::Complete);
    assert_eq!(
        report.history_completeness,
        HistoryCompleteness::NotAssessed
    );
    // Partial root coverage does not invalidate otherwise-checked metadata.
    assert_eq!(report.metadata_integrity, MetadataIntegrity::Valid);
}

impl Fixture {
    fn new(objects: Vec<MetadataCandidate>) -> Self {
        Self {
            enumeration: Ok(MetadataEnumeration {
                objects,
                coverage: CoverageStatus::Complete,
                unavailable_capabilities: Vec::new(),
            }),
            resolved: BTreeMap::new(),
            boundaries: BTreeMap::new(),
            resources: BTreeMap::new(),
            reachability: Ok(PartialReachability::default()),
            context_error: None,
            verification_calls: Cell::new(0),
            boundary_queries: Cell::new(0),
        }
    }

    fn with_reference(
        mut self,
        requested: MetadataIdentifier,
        resolved: Result<Option<ResolvedMetadata>, ProviderFailure>,
    ) -> Self {
        self.resolved.insert(requested, resolved);
        self
    }

    fn with_boundary(
        mut self,
        query: DeclaredBoundaryQuery,
        result: Result<DeclaredBoundaryLookup, ProviderFailure>,
    ) -> Self {
        self.boundaries.insert(query, result);
        self
    }

    fn validate(
        &self,
        scope: ValidationScope,
        depth: ResourceVerificationDepth,
    ) -> Result<omvcs_core::repository_validation::ValidationReport, InvocationError> {
        validate_repository(
            ValidationRequest {
                scope,
                resource_verification_depth: depth,
            },
            self,
        )
    }
}

impl RepositoryValidationBoundary for Fixture {
    fn establish_context(&self, _scope: ValidationScope) -> Result<(), InvocationError> {
        self.context_error.map_or(Ok(()), Err)
    }

    fn enumerate_metadata(
        &self,
        _scope: ValidationScope,
    ) -> Result<MetadataEnumeration, ProviderFailure> {
        self.enumeration.clone()
    }

    fn resolve_admitted_metadata(
        &self,
        identifier: MetadataIdentifier,
    ) -> Result<Option<ResolvedMetadata>, ProviderFailure> {
        self.resolved.get(&identifier).cloned().unwrap_or(Ok(None))
    }

    fn lookup_declared_boundary(
        &self,
        query: DeclaredBoundaryQuery,
    ) -> Result<DeclaredBoundaryLookup, ProviderFailure> {
        self.boundary_queries.set(self.boundary_queries.get() + 1);
        self.boundaries
            .get(&query)
            .cloned()
            .unwrap_or(Ok(DeclaredBoundaryLookup::NoMatch))
    }

    fn verify_resource(
        &self,
        resource: ResourceId,
        _depth: ResourceVerificationDepth,
    ) -> Result<ResourceVerification, ProviderFailure> {
        self.verification_calls
            .set(self.verification_calls.get() + 1);
        self.resources.get(&resource).cloned().unwrap_or(Ok(
            ResourceVerification::UnavailableOrNotLocallyMaterialised,
        ))
    }

    fn partial_line_release_reachability(
        &self,
        _scope: ValidationScope,
    ) -> Result<PartialReachability, ProviderFailure> {
        self.reachability.clone()
    }
}

fn project(byte: u8) -> ProjectId {
    match byte {
        1 => "019cc17d-1b22-7a41-9fe9-c345c468f82c"
            .parse()
            .expect("ProjectId"),
        2 => "019cc17d-1b22-7a41-9fe9-c345c468f82d"
            .parse()
            .expect("ProjectId"),
        _ => "019cc17d-1b22-7a41-9fe9-c345c468f82e"
            .parse()
            .expect("ProjectId"),
    }
}

const fn revision(byte: u8) -> MetadataIdentifier {
    MetadataIdentifier::Revision(RevisionId::from_digest([byte; 32]))
}

const fn project_state(byte: u8) -> MetadataIdentifier {
    MetadataIdentifier::ProjectState(ProjectStateId::from_digest([byte; 32]))
}

fn candidate(
    identifier: MetadataIdentifier,
    project_id: Option<ProjectId>,
    refs: Vec<RequiredMetadataReference>,
    resources: Vec<ResourceId>,
) -> MetadataCandidate {
    let admission = if refs.is_empty() {
        MetadataAdmissionStatus::Admitted
    } else {
        MetadataAdmissionStatus::NotAdmitted("a required target has not been admitted".to_owned())
    };
    MetadataCandidate {
        identifier,
        project_id,
        body_identifier: Ok(identifier),
        admission,
        required_references: refs,
        resources,
    }
}

const fn reference(
    edge_kind: MetadataEdgeKind,
    target: MetadataIdentifier,
    same_project_required: bool,
) -> RequiredMetadataReference {
    RequiredMetadataReference {
        edge_kind,
        target,
        same_project_required,
    }
}

const fn query(
    referring: MetadataIdentifier,
    edge_kind: MetadataEdgeKind,
    target: MetadataIdentifier,
) -> DeclaredBoundaryQuery {
    DeclaredBoundaryQuery {
        referring,
        edge_kind,
        target,
    }
}

const fn resource(byte: u8) -> ResourceId {
    ResourceId::from_digest([byte; 32])
}

#[test]
fn requested_scopes_and_exact_depth_names_are_echoed() {
    assert_eq!(
        "metadata_only".parse::<ResourceVerificationDepth>(),
        Ok(ResourceVerificationDepth::MetadataOnly)
    );
    assert_eq!(
        "verify_available_resources".parse::<ResourceVerificationDepth>(),
        Ok(ResourceVerificationDepth::VerifyAvailableResources)
    );
    assert_eq!(
        "deep_resources".parse::<ResourceVerificationDepth>(),
        Ok(ResourceVerificationDepth::DeepResources)
    );
    assert_eq!(
        "deep".parse::<ResourceVerificationDepth>(),
        Err(InvocationError::InvalidDepth)
    );

    let target_project = project(1);
    let object = candidate(revision(1), Some(target_project), Vec::new(), Vec::new());
    let fixture = Fixture::new(vec![object]);
    for (scope, depth) in [
        (
            ValidationScope::Repository,
            ResourceVerificationDepth::MetadataOnly,
        ),
        (
            ValidationScope::Project(target_project),
            ResourceVerificationDepth::VerifyAvailableResources,
        ),
        (
            ValidationScope::Project(target_project),
            ResourceVerificationDepth::DeepResources,
        ),
    ] {
        let report = fixture.validate(scope, depth).expect("completed report");
        assert_eq!(report.requested_scope, scope);
        assert_eq!(report.resource_verification_depth, depth);
        assert_eq!(report.coverage.requested_scope, scope);
    }
}

#[test]
fn complete_declared_unresolved_and_not_assessed_are_distinct() {
    let referrer = revision(2);
    let target = project_state(3);
    let edge = MetadataEdgeKind::RevisionProjectState;
    let mut object = candidate(
        referrer,
        Some(project(1)),
        vec![reference(edge, target, false)],
        Vec::new(),
    );
    object.admission = MetadataAdmissionStatus::Admitted;

    let complete = Fixture::new(vec![object.clone()]).with_reference(
        target,
        Ok(Some(ResolvedMetadata {
            identifier: target,
            project_id: Some(project(1)),
        })),
    );
    assert_eq!(
        complete
            .validate(
                ValidationScope::Repository,
                ResourceVerificationDepth::MetadataOnly
            )
            .expect("report")
            .history_completeness,
        HistoryCompleteness::NotAssessed
    );

    let mut missing_object = object;
    missing_object.admission =
        MetadataAdmissionStatus::NotAdmitted("required target is absent".to_owned());
    let declared = Fixture::new(vec![missing_object.clone()]).with_boundary(
        query(referrer, edge, target),
        Ok(DeclaredBoundaryLookup::Match),
    );
    let declared_report = declared
        .validate(
            ValidationScope::Repository,
            ResourceVerificationDepth::MetadataOnly,
        )
        .expect("negative finding is a completed report");
    assert_eq!(
        declared_report.history_completeness,
        HistoryCompleteness::DeclaredIncomplete
    );
    assert_eq!(
        declared_report.metadata_integrity,
        MetadataIntegrity::Indeterminate
    );
    assert!(declared_report.findings.iter().any(|finding| matches!(
        finding,
        ValidationFinding::DeclaredHistoryBoundary {
            referring: actual_referrer,
            edge_kind: actual_edge,
            target: actual_target,
        } if *actual_referrer == referrer && *actual_edge == edge && *actual_target == target
    )));

    let unresolved = Fixture::new(vec![missing_object]);
    assert_eq!(
        unresolved
            .validate(
                ValidationScope::Repository,
                ResourceVerificationDepth::MetadataOnly
            )
            .expect("report")
            .history_completeness,
        HistoryCompleteness::Unresolved
    );

    let mut partial = Fixture::new(Vec::new());
    partial.enumeration = Ok(MetadataEnumeration {
        objects: vec![candidate(
            revision(4),
            Some(project(1)),
            Vec::new(),
            Vec::new(),
        )],
        coverage: CoverageStatus::Partial,
        unavailable_capabilities: vec!["metadata page omitted".to_owned()],
    });
    let partial_report = partial
        .validate(
            ValidationScope::Repository,
            ResourceVerificationDepth::MetadataOnly,
        )
        .expect("partial coverage remains a report");
    assert_eq!(
        partial_report.history_completeness,
        HistoryCompleteness::NotAssessed
    );
    assert_eq!(
        partial_report.metadata_integrity,
        MetadataIntegrity::Indeterminate
    );
}

#[test]
fn exact_boundary_tuple_is_required_and_provider_errors_are_not_no_match() {
    let referrer = revision(5);
    let target = project_state(6);
    let edge = MetadataEdgeKind::RevisionProjectState;
    let object = candidate(
        referrer,
        Some(project(1)),
        vec![reference(edge, target, false)],
        Vec::new(),
    );
    let exact = query(referrer, edge, target);
    let nonmatching_edge = query(referrer, MetadataEdgeKind::RevisionParent, target);
    let fixture = Fixture::new(vec![object.clone()])
        .with_boundary(nonmatching_edge, Ok(DeclaredBoundaryLookup::Match))
        .with_boundary(exact, Ok(DeclaredBoundaryLookup::NoMatch));
    let report = fixture
        .validate(
            ValidationScope::Repository,
            ResourceVerificationDepth::MetadataOnly,
        )
        .expect("report");
    assert_eq!(report.history_completeness, HistoryCompleteness::Unresolved);
    assert_eq!(fixture.boundary_queries.get(), 1);

    let resolution_failed = Fixture::new(vec![object.clone()]).with_reference(
        target,
        Err(ProviderFailure::new(
            "metadata_resolution",
            "resolver read failed",
        )),
    );
    let resolution_report = resolution_failed
        .validate(
            ValidationScope::Repository,
            ResourceVerificationDepth::MetadataOnly,
        )
        .expect("resolver failure is a completed report");
    assert_eq!(
        resolution_report.coverage.metadata_resolution,
        CoverageStatus::Partial
    );
    assert_eq!(resolution_failed.boundary_queries.get(), 0);
    assert_eq!(
        resolution_report.history_completeness,
        HistoryCompleteness::NotAssessed
    );

    let failed = Fixture::new(vec![object]).with_boundary(
        exact,
        Err(ProviderFailure::new(
            "declared_boundary_lookup",
            "provider read failed",
        )),
    );
    let failed_report = failed
        .validate(
            ValidationScope::Repository,
            ResourceVerificationDepth::MetadataOnly,
        )
        .expect("provider error becomes a finding");
    assert_eq!(
        failed_report.coverage.declared_boundaries,
        CoverageStatus::Partial
    );
    assert_eq!(
        failed_report.history_completeness,
        HistoryCompleteness::NotAssessed
    );
    assert!(failed_report.findings.iter().any(|finding| matches!(
        finding,
        ValidationFinding::ProviderFailure {
            capability,
            message,
        } if capability == "declared_boundary_lookup" && message == "provider read failed"
    )));
    assert!(
        !failed_report
            .findings
            .iter()
            .any(|finding| matches!(finding, ValidationFinding::UnresolvedMetadata { .. }))
    );
    assert_eq!(failed.boundary_queries.get(), 1);
}

#[test]
fn valid_body_identity_does_not_resolve_or_admit_a_missing_target() {
    let referrer = revision(7);
    let target = project_state(8);
    let edge = MetadataEdgeKind::RevisionProjectState;
    let object = candidate(
        referrer,
        Some(project(1)),
        vec![reference(edge, target, false)],
        Vec::new(),
    );
    assert_eq!(object.body_identifier, Ok(referrer));
    let fixture = Fixture::new(vec![object]).with_boundary(
        query(referrer, edge, target),
        Ok(DeclaredBoundaryLookup::Match),
    );
    let report = fixture
        .validate(
            ValidationScope::Repository,
            ResourceVerificationDepth::MetadataOnly,
        )
        .expect("report");
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| matches!(finding, ValidationFinding::IdentityMismatch { .. }))
    );
    assert_eq!(
        report.history_completeness,
        HistoryCompleteness::DeclaredIncomplete
    );
    assert_eq!(report.metadata_integrity, MetadataIntegrity::Indeterminate);
}

#[test]
fn wrong_body_hash_schema_failure_and_resolver_identity_are_findings() {
    let expected = revision(9);
    let calculated = revision(10);
    let mut bad_hash = candidate(expected, Some(project(1)), Vec::new(), Vec::new());
    bad_hash.body_identifier = Ok(calculated);
    let bad_schema = MetadataCandidate {
        identifier: project_state(11),
        project_id: Some(project(1)),
        body_identifier: Err(MetadataBodyFailure::InvalidBody(
            "closed schema rejected extra member".to_owned(),
        )),
        admission: MetadataAdmissionStatus::Indeterminate("not independently admitted".to_owned()),
        required_references: Vec::new(),
        resources: Vec::new(),
    };
    let report = Fixture::new(vec![bad_hash, bad_schema])
        .validate(
            ValidationScope::Repository,
            ResourceVerificationDepth::MetadataOnly,
        )
        .expect("report");
    assert_eq!(report.metadata_integrity, MetadataIntegrity::Invalid);
    assert!(report.findings.iter().any(|finding| matches!(
        finding,
        ValidationFinding::IdentityMismatch {
            expected,
            actual,
        } if *expected == revision(9) && *actual == revision(10)
    )));
    assert!(report.findings.iter().any(|finding| matches!(
        finding,
        ValidationFinding::MetadataIntegrityFailure { reason, .. }
            if reason == "closed schema rejected extra member"
    )));

    let referrer = revision(12);
    let target = project_state(13);
    let object = candidate(
        referrer,
        Some(project(1)),
        vec![reference(
            MetadataEdgeKind::RevisionProjectState,
            target,
            false,
        )],
        Vec::new(),
    );
    let wrong_resolver = Fixture::new(vec![object]).with_reference(
        target,
        Ok(Some(ResolvedMetadata {
            identifier: project_state(14),
            project_id: Some(project(1)),
        })),
    );
    let resolved_report = wrong_resolver
        .validate(
            ValidationScope::Repository,
            ResourceVerificationDepth::MetadataOnly,
        )
        .expect("report");
    assert_eq!(
        resolved_report.metadata_integrity,
        MetadataIntegrity::Invalid
    );
    assert!(resolved_report.findings.iter().any(|finding| matches!(
        finding,
        ValidationFinding::IdentityMismatch { expected, actual }
            if *expected == target && *actual == project_state(14)
    )));
}

#[test]
fn same_project_admission_and_cycles_are_checked() {
    let source_project = project(1);
    let other_project = project(2);
    let referrer = revision(15);
    let target = revision(16);
    let source = candidate(
        referrer,
        Some(source_project),
        vec![reference(MetadataEdgeKind::RevisionParent, target, true)],
        Vec::new(),
    );
    let crossing = Fixture::new(vec![source]).with_reference(
        target,
        Ok(Some(ResolvedMetadata {
            identifier: target,
            project_id: Some(other_project),
        })),
    );
    let crossing_report = crossing
        .validate(
            ValidationScope::Repository,
            ResourceVerificationDepth::MetadataOnly,
        )
        .expect("report");
    assert_eq!(
        crossing_report.metadata_integrity,
        MetadataIntegrity::Invalid
    );
    assert!(crossing_report.findings.iter().any(|finding| matches!(
        finding,
        ValidationFinding::ProjectMismatch {
            referring,
            target: actual_target,
        } if *referring == referrer && *actual_target == target
    )));

    let cycle = Fixture::new(vec![
        candidate(
            revision(17),
            Some(source_project),
            vec![reference(
                MetadataEdgeKind::RevisionParent,
                revision(18),
                true,
            )],
            Vec::new(),
        ),
        candidate(
            revision(18),
            Some(source_project),
            vec![reference(
                MetadataEdgeKind::RevisionParent,
                revision(17),
                true,
            )],
            Vec::new(),
        ),
    ]);
    let cycle_report = cycle
        .validate(
            ValidationScope::Repository,
            ResourceVerificationDepth::MetadataOnly,
        )
        .expect("report");
    assert!(
        cycle_report
            .findings
            .iter()
            .any(|finding| matches!(finding, ValidationFinding::Cycle { .. }))
    );
    assert_eq!(cycle_report.metadata_integrity, MetadataIntegrity::Invalid);
}

#[test]
fn unavailable_schema_is_indeterminate_not_invalid_and_target_is_not_admitted() {
    let source = revision(19);
    let mut unknown_schema = candidate(
        source,
        Some(project(1)),
        vec![reference(
            MetadataEdgeKind::RevisionProjectState,
            project_state(21),
            false,
        )],
        Vec::new(),
    );
    unknown_schema.body_identifier = Err(MetadataBodyFailure::SchemaUnavailable);
    let ref_target = project_state(21);
    let source_candidate = candidate(
        revision(22),
        Some(project(1)),
        vec![reference(
            MetadataEdgeKind::RevisionProjectState,
            ref_target,
            false,
        )],
        Vec::new(),
    );
    let report = Fixture::new(vec![
        unknown_schema,
        candidate(ref_target, Some(project(1)), Vec::new(), Vec::new()),
        source_candidate,
    ])
    .validate(
        ValidationScope::Repository,
        ResourceVerificationDepth::MetadataOnly,
    )
    .expect("report");
    assert_eq!(report.metadata_integrity, MetadataIntegrity::Indeterminate);
    assert!(report.findings.iter().any(|finding| matches!(
        finding,
        ValidationFinding::SchemaUnavailable { object } if *object == source
    )));
    assert!(report.findings.iter().any(|finding| matches!(
        finding,
        ValidationFinding::SchemaOrAdmissionFailure { object, .. } if *object == ref_target
    )));
    assert_eq!(
        report.history_completeness,
        HistoryCompleteness::NotAssessed
    );
}

#[test]
#[allow(clippy::too_many_lines)]
fn resource_sparsity_is_independent_and_verification_states_stay_distinct() {
    let metadata = revision(23);
    let resource_available = resource(1);
    let resource_unavailable = resource(2);
    let resource_corrupt = resource(3);
    let object = candidate(
        metadata,
        Some(project(1)),
        Vec::new(),
        vec![resource_available, resource_unavailable, resource_corrupt],
    );

    let metadata_only = Fixture::new(vec![object.clone()]);
    let report = metadata_only
        .validate(
            ValidationScope::Repository,
            ResourceVerificationDepth::MetadataOnly,
        )
        .expect("metadata-only report");
    assert_eq!(report.metadata_integrity, MetadataIntegrity::Valid);
    assert_eq!(
        report.history_completeness,
        HistoryCompleteness::NotAssessed
    );
    assert_eq!(report.coverage.resource_verification, None);
    assert!(
        report
            .resources
            .iter()
            .all(|result| result.state == ResourceState::NotChecked)
    );
    assert_eq!(metadata_only.verification_calls.get(), 0);

    let mut available_shallow = Fixture::new(vec![candidate(
        revision(31),
        Some(project(1)),
        Vec::new(),
        vec![resource_available],
    )]);
    available_shallow.resources.insert(
        resource_available,
        Ok(ResourceVerification::Available {
            method: "provider-check".to_owned(),
            strength: VerificationStrength::ProviderSpecific("available-content".to_owned()),
        }),
    );
    let shallow_report = available_shallow
        .validate(
            ValidationScope::Repository,
            ResourceVerificationDepth::VerifyAvailableResources,
        )
        .expect("available-resource report");
    assert_eq!(shallow_report.resources[0].state, ResourceState::Available);
    assert_eq!(
        shallow_report.resources[0].strength,
        Some(VerificationStrength::ProviderSpecific(
            "available-content".to_owned()
        ))
    );

    let mut verified = Fixture::new(vec![object]);
    verified.resources.insert(
        resource_available,
        Ok(ResourceVerification::Available {
            method: "sha256".to_owned(),
            strength: VerificationStrength::FullContent,
        }),
    );
    verified.resources.insert(
        resource_unavailable,
        Ok(ResourceVerification::UnavailableOrNotLocallyMaterialised),
    );
    verified.resources.insert(
        resource_corrupt,
        Ok(ResourceVerification::Corrupt {
            method: "sha256".to_owned(),
            strength: VerificationStrength::FullContent,
        }),
    );
    let verified_report = verified
        .validate(
            ValidationScope::Repository,
            ResourceVerificationDepth::DeepResources,
        )
        .expect("deep resource report");
    assert_eq!(verified_report.metadata_integrity, MetadataIntegrity::Valid);
    assert_eq!(verified.verification_calls.get(), 3);
    assert_eq!(
        verified_report
            .resources
            .iter()
            .find(|result| result.identifier == resource_available)
            .expect("available resource")
            .state,
        ResourceState::Available
    );
    assert_eq!(
        verified_report
            .resources
            .iter()
            .find(|result| result.identifier == resource_unavailable)
            .expect("unavailable resource")
            .state,
        ResourceState::UnavailableOrNotLocallyMaterialised
    );
    assert_eq!(
        verified_report
            .resources
            .iter()
            .find(|result| result.identifier == resource_corrupt)
            .expect("corrupt resource")
            .state,
        ResourceState::Corrupt
    );
    assert_eq!(
        verified_report
            .resources
            .iter()
            .find(|result| result.identifier == resource_corrupt)
            .and_then(|result| result.strength.as_ref()),
        Some(&VerificationStrength::FullContent)
    );

    let mut provider_failed = Fixture::new(vec![candidate(
        revision(32),
        Some(project(1)),
        Vec::new(),
        vec![resource(4)],
    )]);
    provider_failed.resources.insert(
        resource(4),
        Err(ProviderFailure::new(
            "resource_verification",
            "verification provider failed",
        )),
    );
    let failure_report = provider_failed
        .validate(
            ValidationScope::Repository,
            ResourceVerificationDepth::DeepResources,
        )
        .expect("provider failure is a completed report");
    assert_eq!(
        failure_report.coverage.resource_verification,
        Some(CoverageStatus::Partial)
    );
    assert_eq!(failure_report.resources[0].state, ResourceState::NotChecked);
    assert!(failure_report.findings.iter().any(|finding| matches!(
        finding,
        ValidationFinding::ProviderFailure {
            capability,
            message,
        } if capability == "resource_verification" && message == "verification provider failed"
    )));
}

#[test]
fn project_scope_filters_outside_objects_and_reports_partial_coverage() {
    let selected = project(1);
    let other = project(2);
    let fixture = Fixture::new(vec![
        candidate(revision(24), Some(selected), Vec::new(), Vec::new()),
        candidate(revision(25), Some(other), Vec::new(), Vec::new()),
    ]);
    let report = fixture
        .validate(
            ValidationScope::Project(selected),
            ResourceVerificationDepth::MetadataOnly,
        )
        .expect("report");
    assert_eq!(
        report.coverage.requested_scope,
        ValidationScope::Project(selected)
    );
    assert_eq!(report.coverage.object_enumeration, CoverageStatus::Partial);
    assert!(!report.findings.iter().any(|finding| matches!(
        finding,
        ValidationFinding::IdentityMismatch {
            expected: MetadataIdentifier::Revision(id),
            ..
        } if *id == RevisionId::from_digest([25; 32])
    )));
    assert_eq!(report.metadata_integrity, MetadataIntegrity::Indeterminate);
}

#[test]
fn provider_errors_are_completed_reports_with_partial_coverage() {
    let object = candidate(revision(26), Some(project(1)), Vec::new(), Vec::new());
    let mut failed_enumeration = Fixture::new(vec![object.clone()]);
    failed_enumeration.enumeration = Err(ProviderFailure::new(
        "metadata_enumeration",
        "enumeration interrupted",
    ));
    let enumeration_report = failed_enumeration
        .validate(
            ValidationScope::Repository,
            ResourceVerificationDepth::MetadataOnly,
        )
        .expect("failure after report construction is a report");
    assert_eq!(
        enumeration_report.coverage.object_enumeration,
        CoverageStatus::Unavailable
    );
    assert_eq!(
        enumeration_report.history_completeness,
        HistoryCompleteness::NotAssessed
    );
    assert!(enumeration_report.findings.iter().any(|finding| matches!(
        finding,
        ValidationFinding::ProviderFailure { capability, .. }
            if capability == "metadata_enumeration"
    )));

    let mut failed_roots = Fixture::new(vec![object]);
    failed_roots.reachability = Err(ProviderFailure::new(
        "line_release_roots",
        "root enumeration failed",
    ));
    let root_report = failed_roots
        .validate(
            ValidationScope::Repository,
            ResourceVerificationDepth::MetadataOnly,
        )
        .expect("root error is a completed report");
    assert!(root_report.findings.iter().any(|finding| matches!(
        finding,
        ValidationFinding::ProviderFailure { capability, .. }
            if capability == "line_release_roots"
    )));
    assert!(root_report.coverage.required_roots.iter().any(|coverage| {
        coverage.provider == RootProvider::LinesAndReleases
            && coverage.status == CoverageStatus::Unavailable
    }));
    assert_eq!(
        root_report
            .coverage
            .required_roots
            .iter()
            .filter(|coverage| coverage.status != CoverageStatus::Complete)
            .count(),
        5
    );
    assert!(
        root_report
            .coverage
            .partial_line_release_reachability
            .is_none()
    );
}

#[test]
fn invocation_errors_are_distinct_from_reports_and_read_only_boundaries() {
    let fixture = Fixture::new(Vec::new());
    let mut unavailable = Fixture::new(Vec::new());
    unavailable.context_error = Some(InvocationError::RepositoryUnavailable);
    assert_eq!(
        unavailable.validate(
            ValidationScope::Repository,
            ResourceVerificationDepth::MetadataOnly
        ),
        Err(InvocationError::RepositoryUnavailable)
    );
    let mut unavailable_project = Fixture::new(Vec::new());
    unavailable_project.context_error = Some(InvocationError::ProjectUnavailable);
    assert_eq!(
        unavailable_project.validate(
            ValidationScope::Project(project(3)),
            ResourceVerificationDepth::MetadataOnly,
        ),
        Err(InvocationError::ProjectUnavailable)
    );
    assert!(
        fixture
            .validate(
                ValidationScope::Project(project(3)),
                ResourceVerificationDepth::MetadataOnly,
            )
            .is_ok()
    );

    // Every operation exposed by the provider is a shared read. The boundary
    // has no repair, declaration mutation, fetch, or materialisation method;
    // repeated invocations perform only these observations.
    let snapshot = (
        fixture.enumeration.clone(),
        fixture.boundaries.clone(),
        fixture.resources.clone(),
    );
    for _ in 0..2 {
        fixture
            .validate(
                ValidationScope::Repository,
                ResourceVerificationDepth::MetadataOnly,
            )
            .expect("read-only invocation");
    }
    assert_eq!(
        (
            fixture.enumeration.clone(),
            fixture.boundaries.clone(),
            fixture.resources.clone(),
        ),
        snapshot
    );
    assert_eq!(fixture.verification_calls.get(), 0);
}

#[test]
fn partial_reachability_never_exposes_a_global_unreachable_claim() {
    let mut partial = PartialReachability::default();
    partial.revisions.push(RevisionId::from_digest([27; 32]));
    let mut fixture = Fixture::new(vec![candidate(
        revision(28),
        Some(project(1)),
        Vec::new(),
        Vec::new(),
    )]);
    fixture.reachability = Ok(partial.clone());
    let report = fixture
        .validate(
            ValidationScope::Repository,
            ResourceVerificationDepth::MetadataOnly,
        )
        .expect("report");
    assert_eq!(
        report.coverage.partial_line_release_reachability,
        Some(partial)
    );
    assert_eq!(
        report
            .coverage
            .required_roots
            .iter()
            .find(|coverage| coverage.provider == RootProvider::LinesAndReleases)
            .expect("Line/Release provider")
            .status,
        CoverageStatus::Partial
    );
    assert_eq!(
        report
            .coverage
            .required_roots
            .iter()
            .filter(|coverage| coverage.status == CoverageStatus::Unavailable)
            .count(),
        4
    );
    assert!(!report.findings.iter().any(|finding| matches!(
        finding,
        ValidationFinding::IncompleteCoverage { capability, .. }
            if capability.contains("unreachable")
    )));
}

#[test]
fn resolved_target_ignores_a_retained_boundary_and_is_checked_normally() {
    let referrer = revision(29);
    let target = project_state(30);
    let edge = MetadataEdgeKind::RevisionProjectState;
    let mut object = candidate(
        referrer,
        Some(project(1)),
        vec![reference(edge, target, false)],
        Vec::new(),
    );
    object.admission = MetadataAdmissionStatus::Admitted;
    let fixture = Fixture::new(vec![object])
        .with_reference(
            target,
            Ok(Some(ResolvedMetadata {
                identifier: target,
                project_id: Some(project(1)),
            })),
        )
        .with_boundary(
            query(referrer, edge, target),
            Ok(DeclaredBoundaryLookup::Match),
        );
    let report = fixture
        .validate(
            ValidationScope::Repository,
            ResourceVerificationDepth::MetadataOnly,
        )
        .expect("report");
    assert_eq!(fixture.boundary_queries.get(), 0);
    assert_eq!(
        report.history_completeness,
        HistoryCompleteness::NotAssessed
    );
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| matches!(finding, ValidationFinding::DeclaredHistoryBoundary { .. }))
    );
}
