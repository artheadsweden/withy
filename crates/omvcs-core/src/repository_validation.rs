//! Read-only repository validation orchestration.
//!
//! Object-specific body verification and admission remain the responsibility
//! of the existing model validators and admitted-object boundaries. This
//! module combines their provider-neutral results without defining alternate
//! historical schemas or admission rules.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::str::FromStr;

use omvcs_model::{
    AdapterStateId, ComponentStateId, ProjectId, ProjectStateId, ReleaseId, ResourceId, RevisionId,
};

use crate::reachability::{
    HistoricalId, PartialReachability, ReachabilityDefect, ReachabilityRoot, WorkingStateRoot,
};

/// The explicitly requested extent of a repository validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceVerificationDepth {
    /// Inspect metadata only; do not ask a Resource provider to inspect bytes.
    MetadataOnly,
    /// Verify only Resource bytes already available through the provider.
    VerifyAvailableResources,
    /// Fully verify obtainable Resource bytes without fetching/materialising.
    DeepResources,
}

impl ResourceVerificationDepth {
    /// Returns the exact public token for this verification depth.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MetadataOnly => "metadata_only",
            Self::VerifyAvailableResources => "verify_available_resources",
            Self::DeepResources => "deep_resources",
        }
    }
}

impl FromStr for ResourceVerificationDepth {
    type Err = InvocationError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "metadata_only" => Ok(Self::MetadataOnly),
            "verify_available_resources" => Ok(Self::VerifyAvailableResources),
            "deep_resources" => Ok(Self::DeepResources),
            _ => Err(InvocationError::InvalidDepth),
        }
    }
}

/// The approved validation scope: a local Repository or one Project.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationScope {
    /// Validate all metadata enumerated in the local Repository.
    Repository,
    /// Validate the Project-specific metadata enumerated by the provider.
    Project(ProjectId),
}

/// A typed invocation of `ValidateRepository`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValidationRequest {
    /// The explicitly requested scope.
    pub scope: ValidationScope,
    /// The explicitly requested Resource verification depth.
    pub resource_verification_depth: ResourceVerificationDepth,
}

/// Typed metadata identifier used for object identities and exact edge tuples.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MetadataIdentifier {
    /// Component State identifier.
    ComponentState(ComponentStateId),
    /// Adapter State identifier.
    AdapterState(AdapterStateId),
    /// Project State identifier.
    ProjectState(ProjectStateId),
    /// Revision identifier.
    Revision(RevisionId),
    /// Release identifier.
    Release(ReleaseId),
}

impl fmt::Display for MetadataIdentifier {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ComponentState(id) => id.fmt(formatter),
            Self::AdapterState(id) => id.fmt(formatter),
            Self::ProjectState(id) => id.fmt(formatter),
            Self::Revision(id) => id.fmt(formatter),
            Self::Release(id) => id.fmt(formatter),
        }
    }
}

/// Normative kind of a required historical metadata edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MetadataEdgeKind {
    /// Component State to parent Component State.
    ComponentStateParent,
    /// Project State to member Component State.
    ProjectStateComponent,
    /// Project State to Adapter State.
    ProjectStateAdapterState,
    /// Revision to Project State.
    RevisionProjectState,
    /// Revision to parent Revision.
    RevisionParent,
    /// Release to target Revision.
    ReleaseRevision,
}

/// One required historical reference supplied from the canonical body.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RequiredMetadataReference {
    /// Normative edge kind.
    pub edge_kind: MetadataEdgeKind,
    /// Required target Identifier.
    pub target: MetadataIdentifier,
    /// Whether the existing admission contract requires equal Project identity.
    pub same_project_required: bool,
}

/// Object-specific body verification failure returned by its exact model API.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetadataBodyFailure {
    /// The exact schema is unknown or unavailable; no valid Identifier exists.
    SchemaUnavailable,
    /// The available exact schema or canonical body validation rejected it.
    InvalidBody(String),
}

/// Historical admission outcome from the existing object-specific admission
/// API, kept separate from the independently verified body Identifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetadataAdmissionStatus {
    /// Ordinary object-specific historical admission succeeded.
    Admitted,
    /// Required metadata is not admitted, without an established invalidity.
    NotAdmitted(String),
    /// An actual object-specific historical invariant failed.
    Invalid(String),
    /// Admission could not be conclusively assessed.
    Indeterminate(String),
}

/// A body-verified candidate enumerated under its repository key.
///
/// `body_identifier` MUST be calculated from the canonical body under its
/// exact available schema, independently of reference resolution, using the
/// existing model API. `required_references` MUST be projected from that same
/// canonical body. `admission` MUST be obtained through the ordinary
/// object-specific candidate/admission API; this operation does not replace
/// those admission checks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetadataCandidate {
    /// Repository key under which this candidate was enumerated.
    pub identifier: MetadataIdentifier,
    /// Project identity when established by the exact body/admitted context.
    pub project_id: Option<ProjectId>,
    /// Independently verified body-derived Identifier or body failure.
    pub body_identifier: Result<MetadataIdentifier, MetadataBodyFailure>,
    /// Existing object-specific admission outcome.
    pub admission: MetadataAdmissionStatus,
    /// Required metadata references extracted from the body.
    pub required_references: Vec<RequiredMetadataReference>,
    /// Resource References from the body; Resource bytes are separate.
    pub resources: Vec<ResourceId>,
}

/// An admitted target returned by the ordinary object admission boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvedMetadata {
    /// Identity returned by the resolver (checked against the requested key).
    pub identifier: MetadataIdentifier,
    /// Project identity where defined by the existing admitted object.
    pub project_id: Option<ProjectId>,
}

/// Failure while accessing a validation provider after report construction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderFailure {
    /// Stable capability name, such as `metadata_enumeration`.
    pub capability: String,
    /// Provider-neutral diagnostic text.
    pub message: String,
}

impl ProviderFailure {
    /// Creates a provider-neutral failure.
    #[must_use]
    pub fn new(capability: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            capability: capability.into(),
            message: message.into(),
        }
    }
}

/// Enumeration result and explicit scope coverage.
///
/// `Complete` is valid only when every object/provider class required by the
/// requested validation scope has been enumerated and its applicable checks
/// performed. A provider that cannot assess an in-scope class (for example,
/// Line/default-Line metadata or other operational records not represented by
/// [`MetadataCandidate`]) MUST return `Partial` or `Unavailable` and identify
/// that capability instead of claiming complete coverage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetadataEnumeration {
    /// Enumerated candidates in the requested scope.
    pub objects: Vec<MetadataCandidate>,
    /// Whether the complete requested object set was enumerated.
    pub coverage: CoverageStatus,
    /// Provider capability limitations, if any.
    pub unavailable_capabilities: Vec<String>,
}

/// Exact query key for local operational declared-boundary metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct DeclaredBoundaryQuery {
    /// Referring immutable object's Identifier.
    pub referring: MetadataIdentifier,
    /// Normative edge kind.
    pub edge_kind: MetadataEdgeKind,
    /// Omitted target Identifier.
    pub target: MetadataIdentifier,
}

/// Explicit three-way declared-boundary lookup result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeclaredBoundaryLookup {
    /// Exact tuple has an intentional-omission declaration.
    Match,
    /// Exact tuple has no declaration.
    NoMatch,
}

/// Result of Resource verification using bytes already available to a provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceVerification {
    /// Available bytes passed the reported verification method and strength.
    Available {
        /// Provider's verification method.
        method: String,
        /// Verification strength.
        strength: VerificationStrength,
    },
    /// No locally available bytes could be verified; this is not corruption.
    UnavailableOrNotLocallyMaterialised,
    /// Available bytes failed Resource identity/content verification.
    Corrupt {
        /// Provider's verification method.
        method: String,
        /// Verification strength.
        strength: VerificationStrength,
    },
}

/// Strength of the reported Resource verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerificationStrength {
    /// Full Resource bytes were hashed and compared with the Resource Identifier.
    FullContent,
    /// Provider-specific verification strength, retained verbatim.
    ProviderSpecific(String),
}

/// Result for one assessed Resource in the completed report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceResult {
    /// Resource Identifier.
    pub identifier: ResourceId,
    /// Final Resource state for the requested depth.
    pub state: ResourceState,
    /// Verification method, when bytes were actually checked.
    pub method: Option<String>,
    /// Verification strength, when bytes were actually checked.
    pub strength: Option<VerificationStrength>,
}

/// Distinct Resource result states required by the validation contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceState {
    /// Bytes were not requested for inspection.
    NotChecked,
    /// No locally available bytes could be verified.
    UnavailableOrNotLocallyMaterialised,
    /// Available bytes passed verification.
    Available,
    /// Available bytes failed verification.
    Corrupt,
}

/// Machine-readable metadata integrity aggregate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetadataIntegrity {
    /// Every required check in adequately covered scope passed.
    Valid,
    /// An actually checked metadata invariant failed.
    Invalid,
    /// Required data or provider coverage prevents a conclusion.
    Indeterminate,
}

/// Machine-readable local history completeness aggregate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoryCompleteness {
    /// All required metadata resolved with complete applicable provider
    /// coverage, including every required root-provider class.
    Complete,
    /// At least one absent required edge has an exact boundary declaration,
    /// with no known undeclared missing edge.
    DeclaredIncomplete,
    /// At least one required target is absent without an exact declaration.
    Unresolved,
    /// Available providers cannot meaningfully determine requested scope.
    NotAssessed,
}

/// Coverage status for object enumeration and each provider capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoverageStatus {
    /// The provider covered its declared capability for this invocation.
    Complete,
    /// The provider covered only a subset or its capability is intentionally
    /// partial under the current Core implementation.
    Partial,
    /// The capability was unavailable.
    Unavailable,
}

/// Required Core §62 root/provider classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RootProvider {
    /// Current retained Line targets and admitted Releases enumerated by WORK-0013.
    LinesAndReleases,
    /// Open Contributions.
    Contributions,
    /// Configured archival pins.
    ArchivalPins,
    /// Present Base Revision and source Component States in current Working State records.
    WorkingStateSafetyReferences,
    /// Pending publication transactions.
    PendingPublicationTransactions,
}

/// Coverage state for one required root provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootProviderCoverage {
    /// Provider class.
    pub provider: RootProvider,
    /// Coverage status.
    pub status: CoverageStatus,
    /// Capability not currently covered, if applicable.
    pub unavailable_capability: Option<String>,
}

/// Explicit scope and provider coverage in a completed report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationCoverage {
    /// Echo of the requested scope.
    pub requested_scope: ValidationScope,
    /// Metadata enumeration coverage.
    pub object_enumeration: CoverageStatus,
    /// Existing admitted-object resolver coverage.
    pub metadata_resolution: CoverageStatus,
    /// Declared-boundary provider coverage.
    pub declared_boundaries: CoverageStatus,
    /// Resource verification coverage; `None` when `metadata_only` did not
    /// request Resource-byte inspection.
    pub resource_verification: Option<CoverageStatus>,
    /// Required root/provider coverage.
    pub required_roots: Vec<RootProviderCoverage>,
    /// Explicitly partial repository reachability result, when obtained.
    pub partial_repository_reachability: Option<PartialReachability>,
    /// Capabilities not assessed or unavailable.
    pub unavailable_capabilities: Vec<String>,
}

/// Typed findings in a completed report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationFinding {
    /// An available schema or canonical body check failed.
    MetadataIntegrityFailure {
        /// Object that failed the check.
        object: MetadataIdentifier,
        /// Failure reported by the model boundary.
        reason: String,
    },
    /// An object could not be body-verified because its schema is unavailable.
    SchemaUnavailable {
        /// Object that could not be verified.
        object: MetadataIdentifier,
    },
    /// Enumerated candidate's body-derived ID disagrees with its repository key.
    IdentityMismatch {
        /// Expected key or referenced target.
        expected: MetadataIdentifier,
        /// Calculated or resolver-returned identity.
        actual: MetadataIdentifier,
    },
    /// Existing admission boundary rejected/missed a required metadata object.
    SchemaOrAdmissionFailure {
        /// Candidate requiring admission.
        object: MetadataIdentifier,
        /// Provider-neutral explanation.
        reason: String,
    },
    /// A required same-Project reference points across Project identity.
    ProjectMismatch {
        /// Referring object.
        referring: MetadataIdentifier,
        /// Required target.
        target: MetadataIdentifier,
    },
    /// A known metadata-reference cycle was found.
    Cycle {
        /// One object on the detected cycle.
        object: MetadataIdentifier,
    },
    /// Required target absent without exact boundary declaration.
    UnresolvedMetadata {
        /// Exact referring object.
        referring: MetadataIdentifier,
        /// Normative edge kind.
        edge_kind: MetadataEdgeKind,
        /// Missing required target.
        target: MetadataIdentifier,
    },
    /// Metadata reached during partial repository traversal was unresolved,
    /// but no referring metadata edge or Working State root context applies.
    UnresolvedReachabilityMetadata {
        /// Unresolved historical object identifier.
        target: MetadataIdentifier,
    },
    /// A retained Line or admitted Release directly references a missing
    /// Revision. This reports the known root association without inventing an
    /// edge kind.
    UnresolvedRootReference {
        /// Enumerated Line or admitted Release that supplies the root.
        root: ReachabilityRoot,
        /// Directly referenced Revision with unavailable metadata.
        target: MetadataIdentifier,
    },
    /// A current persisted Working State directly references missing
    /// historical metadata. No historical referrer or edge kind is assigned.
    UnresolvedWorkingStateRootReference {
        /// Project and operational field context supplying the safety root.
        root: WorkingStateRoot,
        /// Directly referenced historical metadata Identifier.
        target: MetadataIdentifier,
    },
    /// Required target absent with exact boundary declaration.
    DeclaredHistoryBoundary {
        /// Exact referring object.
        referring: MetadataIdentifier,
        /// Normative edge kind.
        edge_kind: MetadataEdgeKind,
        /// Omitted target.
        target: MetadataIdentifier,
    },
    /// A Resource was not locally available for requested verification.
    ResourceUnavailable {
        /// Resource identifier.
        resource: ResourceId,
    },
    /// Available Resource bytes failed content verification.
    ResourceCorrupt {
        /// Resource identifier.
        resource: ResourceId,
        /// Verification method.
        method: String,
    },
    /// Scope or provider coverage is less than complete.
    IncompleteCoverage {
        /// Capability with partial or unavailable coverage.
        capability: String,
        /// Coverage status.
        status: CoverageStatus,
    },
    /// Provider failed after report construction began.
    ProviderFailure {
        /// Provider capability.
        capability: String,
        /// Provider diagnostic.
        message: String,
    },
}

/// Completed validation result. Findings do not turn it into an invocation error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationReport {
    /// Echo of the requested scope.
    pub requested_scope: ValidationScope,
    /// Echo of the requested Resource verification depth.
    pub resource_verification_depth: ResourceVerificationDepth,
    /// Aggregate metadata integrity.
    pub metadata_integrity: MetadataIntegrity,
    /// Aggregate local history completeness.
    pub history_completeness: HistoryCompleteness,
    /// Per-Resource state.
    pub resources: Vec<ResourceResult>,
    /// Explicit coverage.
    pub coverage: ValidationCoverage,
    /// Typed diagnostic findings.
    pub findings: Vec<ValidationFinding>,
}

/// Failure before a completed report can be constructed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvocationError {
    /// Resource depth token is not one of the exact approved names.
    InvalidDepth,
    /// Repository context could not be established as open and readable.
    RepositoryUnavailable,
    /// Requested Project scope does not exist or cannot be established.
    ProjectUnavailable,
}

impl fmt::Display for InvocationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidDepth => "invalid Resource verification depth",
            Self::RepositoryUnavailable => "repository context is unavailable",
            Self::ProjectUnavailable => "requested Project scope is unavailable",
        })
    }
}

impl std::error::Error for InvocationError {}

/// Provider-neutral read-only validation boundary.
///
/// Implementations MUST use existing model candidate identity/schema APIs and
/// ordinary admission/resolver APIs. `verify_resource` may inspect only bytes
/// already available and MUST NOT fetch, materialise, or mutate availability
/// or verification records. A `deep_resources` success MUST report
/// [`VerificationStrength::FullContent`]. A corrupt result MUST be based on a
/// full Resource identity check. The repository-reachability method exposes
/// only the WORK-0013 partial result, including Lines, Releases, and Working
/// State safety references; it must not infer other Core §62 roots.
pub trait RepositoryValidationBoundary {
    /// Establishes an open/readable Repository and, for Project scope, an
    /// existing readable Project. Called before report construction.
    ///
    /// # Errors
    ///
    /// Returns an invocation error if the requested context is invalid or
    /// unavailable.
    fn establish_context(&self, scope: ValidationScope) -> Result<(), InvocationError>;

    /// Enumerates validation candidates within exactly the requested scope.
    ///
    /// # Errors
    ///
    /// Returns a provider failure; the operation reports it in a completed
    /// partial report.
    fn enumerate_metadata(
        &self,
        scope: ValidationScope,
    ) -> Result<MetadataEnumeration, ProviderFailure>;

    /// Resolves a required target only through the existing admitted-object
    /// boundary. `None` means missing, unavailable, unchecked, or unadmitted.
    ///
    /// # Errors
    ///
    /// Returns a provider failure, never an empty/no-match result.
    fn resolve_admitted_metadata(
        &self,
        identifier: MetadataIdentifier,
    ) -> Result<Option<ResolvedMetadata>, ProviderFailure>;

    /// Queries the exact `(referring, edge_kind, target)` tuple.
    ///
    /// # Errors
    ///
    /// Returns a provider failure, never `NoMatch`.
    fn lookup_declared_boundary(
        &self,
        query: DeclaredBoundaryQuery,
    ) -> Result<DeclaredBoundaryLookup, ProviderFailure>;

    /// Verifies only already-available Resource bytes at the requested depth.
    ///
    /// # Errors
    ///
    /// Returns a provider failure without claiming Resource unavailability or
    /// corruption.
    fn verify_resource(
        &self,
        resource: ResourceId,
        depth: ResourceVerificationDepth,
    ) -> Result<ResourceVerification, ProviderFailure>;

    /// Returns existing WORK-0013 partial repository reachability for scope.
    ///
    /// # Errors
    ///
    /// Returns an enumeration/provider failure, which becomes an incomplete
    /// report finding.
    fn partial_repository_reachability(
        &self,
        scope: ValidationScope,
    ) -> Result<PartialReachability, ProviderFailure>;
}

/// Runs strictly read-only repository validation.
///
/// Invalid/unavailable context returns [`InvocationError`]. Once context is
/// established, provider failures are retained as findings and partial
/// coverage in the completed report.
///
/// # Errors
///
/// Returns [`InvocationError`] when the requested Repository or Project
/// context cannot be established before report construction begins.
#[allow(clippy::too_many_lines)]
pub fn validate_repository(
    request: ValidationRequest,
    boundary: &dyn RepositoryValidationBoundary,
) -> Result<ValidationReport, InvocationError> {
    boundary.establish_context(request.scope)?;

    let mut report = ValidationReport {
        requested_scope: request.scope,
        resource_verification_depth: request.resource_verification_depth,
        metadata_integrity: MetadataIntegrity::Indeterminate,
        history_completeness: HistoryCompleteness::NotAssessed,
        resources: Vec::new(),
        coverage: ValidationCoverage {
            requested_scope: request.scope,
            object_enumeration: CoverageStatus::Unavailable,
            metadata_resolution: CoverageStatus::Complete,
            declared_boundaries: CoverageStatus::Complete,
            resource_verification: (request.resource_verification_depth
                != ResourceVerificationDepth::MetadataOnly)
                .then_some(CoverageStatus::Unavailable),
            required_roots: Vec::new(),
            partial_repository_reachability: None,
            unavailable_capabilities: Vec::new(),
        },
        findings: Vec::new(),
    };

    let enumeration = match boundary.enumerate_metadata(request.scope) {
        Ok(enumeration) => {
            report.coverage.object_enumeration = if enumeration.coverage == CoverageStatus::Complete
                && !enumeration.unavailable_capabilities.is_empty()
            {
                CoverageStatus::Partial
            } else {
                enumeration.coverage
            };
            report
                .coverage
                .unavailable_capabilities
                .extend(enumeration.unavailable_capabilities);
            if report.coverage.object_enumeration != CoverageStatus::Complete {
                report.findings.push(ValidationFinding::IncompleteCoverage {
                    capability: "metadata_enumeration".to_owned(),
                    status: report.coverage.object_enumeration,
                });
            }
            Some(enumeration.objects)
        }
        Err(error) => {
            report.coverage.object_enumeration = CoverageStatus::Unavailable;
            push_provider_failure(&mut report, error);
            None
        }
    };

    let mut scope_objects = Vec::new();
    if let Some(objects) = enumeration {
        for object in objects {
            if let ValidationScope::Project(project_id) = request.scope {
                if object.project_id != Some(project_id) {
                    report.coverage.object_enumeration = CoverageStatus::Partial;
                    report.coverage.unavailable_capabilities.push(
                        "provider returned an object outside or not attributable to requested Project"
                            .to_owned(),
                    );
                    report.findings.push(ValidationFinding::IncompleteCoverage {
                        capability: "project_scoped_metadata_enumeration".to_owned(),
                        status: CoverageStatus::Partial,
                    });
                    continue;
                }
            }
            scope_objects.push(object);
        }
    }

    let mut candidates = BTreeMap::new();
    for object in &scope_objects {
        if candidates.insert(object.identifier, object).is_some() {
            report
                .findings
                .push(ValidationFinding::MetadataIntegrityFailure {
                    object: object.identifier,
                    reason: "duplicate metadata Identifier in enumeration".to_owned(),
                });
        }
        match &object.body_identifier {
            Ok(calculated) if *calculated != object.identifier => {
                report.findings.push(ValidationFinding::IdentityMismatch {
                    expected: object.identifier,
                    actual: *calculated,
                });
            }
            Ok(_) => {}
            Err(MetadataBodyFailure::SchemaUnavailable) => {
                report.findings.push(ValidationFinding::SchemaUnavailable {
                    object: object.identifier,
                });
            }
            Err(MetadataBodyFailure::InvalidBody(reason)) => {
                report
                    .findings
                    .push(ValidationFinding::MetadataIntegrityFailure {
                        object: object.identifier,
                        reason: reason.clone(),
                    });
            }
        }
        match &object.admission {
            MetadataAdmissionStatus::Admitted => {}
            MetadataAdmissionStatus::NotAdmitted(reason)
            | MetadataAdmissionStatus::Indeterminate(reason) => {
                report
                    .findings
                    .push(ValidationFinding::SchemaOrAdmissionFailure {
                        object: object.identifier,
                        reason: reason.clone(),
                    });
            }
            MetadataAdmissionStatus::Invalid(reason) => {
                report
                    .findings
                    .push(ValidationFinding::MetadataIntegrityFailure {
                        object: object.identifier,
                        reason: reason.clone(),
                    });
            }
        }
    }

    let mut has_declared = false;
    let mut has_unresolved = false;
    let mut has_indeterminate_reference = false;
    let mut boundary_provider_failed = false;
    let mut resource_ids = BTreeSet::new();
    for object in &scope_objects {
        resource_ids.extend(object.resources.iter().copied());
        for reference in &object.required_references {
            let query = DeclaredBoundaryQuery {
                referring: object.identifier,
                edge_kind: reference.edge_kind,
                target: reference.target,
            };
            match boundary.resolve_admitted_metadata(reference.target) {
                Ok(Some(resolved)) => {
                    if resolved.identifier != reference.target {
                        report.findings.push(ValidationFinding::IdentityMismatch {
                            expected: reference.target,
                            actual: resolved.identifier,
                        });
                        continue;
                    }
                    if reference.same_project_required {
                        match (object.project_id, resolved.project_id) {
                            (Some(source), Some(target)) if source != target => {
                                report.findings.push(ValidationFinding::ProjectMismatch {
                                    referring: object.identifier,
                                    target: reference.target,
                                });
                            }
                            (Some(_), Some(_)) => {}
                            _ => {
                                has_indeterminate_reference = true;
                                report
                                    .findings
                                    .push(ValidationFinding::SchemaOrAdmissionFailure {
                                        object: object.identifier,
                                        reason: "same-Project admission could not be established"
                                            .to_owned(),
                                    });
                            }
                        }
                    }
                }
                Ok(None) if candidates.contains_key(&reference.target) => {
                    // A present candidate is validated normally; an old
                    // declaration cannot hide an object that has reappeared.
                    has_indeterminate_reference = true;
                    report
                        .findings
                        .push(ValidationFinding::SchemaOrAdmissionFailure {
                            object: reference.target,
                            reason: "target candidate exists but is not admitted".to_owned(),
                        });
                }
                Ok(None) => match boundary.lookup_declared_boundary(query) {
                    Ok(DeclaredBoundaryLookup::Match) => {
                        has_declared = true;
                        report
                            .findings
                            .push(ValidationFinding::DeclaredHistoryBoundary {
                                referring: object.identifier,
                                edge_kind: reference.edge_kind,
                                target: reference.target,
                            });
                    }
                    Ok(DeclaredBoundaryLookup::NoMatch) => {
                        has_unresolved = true;
                        report.findings.push(ValidationFinding::UnresolvedMetadata {
                            referring: object.identifier,
                            edge_kind: reference.edge_kind,
                            target: reference.target,
                        });
                    }
                    Err(error) => {
                        boundary_provider_failed = true;
                        has_indeterminate_reference = true;
                        report.coverage.declared_boundaries = CoverageStatus::Partial;
                        push_provider_failure(&mut report, error);
                    }
                },
                Err(error) => {
                    has_indeterminate_reference = true;
                    report.coverage.metadata_resolution = CoverageStatus::Partial;
                    push_provider_failure(&mut report, error);
                }
            }
        }
    }

    add_cycles(&scope_objects, &mut report.findings);

    match boundary.partial_repository_reachability(request.scope) {
        Ok(partial) => {
            if !partial.unresolved.is_empty()
                || !partial.unresolved_root_references.is_empty()
                || !partial.unresolved_working_state_root_references.is_empty()
            {
                has_unresolved = true;
            }
            let mut contextualized_root_targets = partial
                .unresolved_root_references
                .iter()
                .map(|root_reference| HistoricalId::Revision(root_reference.target))
                .collect::<BTreeSet<_>>();
            contextualized_root_targets.extend(
                partial
                    .unresolved_working_state_root_references
                    .iter()
                    .map(|root_reference| root_reference.target),
            );
            for root_reference in &partial.unresolved_root_references {
                report
                    .findings
                    .push(ValidationFinding::UnresolvedRootReference {
                        root: root_reference.root,
                        target: MetadataIdentifier::Revision(root_reference.target),
                    });
            }
            for root_reference in &partial.unresolved_working_state_root_references {
                report
                    .findings
                    .push(ValidationFinding::UnresolvedWorkingStateRootReference {
                        root: root_reference.root,
                        target: historical_identifier(root_reference.target),
                    });
            }
            for target in &partial.unresolved {
                if !contextualized_root_targets.contains(target) {
                    report
                        .findings
                        .push(ValidationFinding::UnresolvedReachabilityMetadata {
                            target: historical_identifier(*target),
                        });
                }
            }
            for defect in &partial.defects {
                match defect {
                    ReachabilityDefect::IdentifierMismatch {
                        requested,
                        resolved,
                    } => {
                        report.findings.push(ValidationFinding::IdentityMismatch {
                            expected: historical_identifier(*requested),
                            actual: historical_identifier(*resolved),
                        });
                    }
                    ReachabilityDefect::Cycle { object } => {
                        report.findings.push(ValidationFinding::Cycle {
                            object: historical_identifier(*object),
                        });
                    }
                }
            }
            report.coverage.partial_repository_reachability = Some(partial);
            report.coverage.required_roots.push(RootProviderCoverage {
                provider: RootProvider::LinesAndReleases,
                status: CoverageStatus::Complete,
                unavailable_capability: None,
            });
            report.coverage.required_roots.push(RootProviderCoverage {
                provider: RootProvider::WorkingStateSafetyReferences,
                status: CoverageStatus::Complete,
                unavailable_capability: None,
            });
        }
        Err(error) => {
            report.coverage.required_roots.push(RootProviderCoverage {
                provider: RootProvider::LinesAndReleases,
                status: CoverageStatus::Unavailable,
                unavailable_capability: Some(error.capability.clone()),
            });
            report.coverage.required_roots.push(RootProviderCoverage {
                provider: RootProvider::WorkingStateSafetyReferences,
                status: CoverageStatus::Unavailable,
                unavailable_capability: Some(error.capability.clone()),
            });
            report.findings.push(ValidationFinding::IncompleteCoverage {
                capability: error.capability.clone(),
                status: CoverageStatus::Unavailable,
            });
            push_provider_failure(&mut report, error);
        }
    }
    for provider in [
        RootProvider::Contributions,
        RootProvider::ArchivalPins,
        RootProvider::PendingPublicationTransactions,
    ] {
        let capability = match provider {
            RootProvider::Contributions => "Contribution roots are not implemented",
            RootProvider::ArchivalPins => "configured archival-pin roots are not implemented",
            RootProvider::PendingPublicationTransactions => {
                "pending-publication roots are not implemented"
            }
            RootProvider::WorkingStateSafetyReferences => {
                "Working State safety-reference roots are supplied by WORK-0013"
            }
            RootProvider::LinesAndReleases => "Line and Release coverage is supplied by WORK-0013",
        };
        report.coverage.required_roots.push(RootProviderCoverage {
            provider,
            status: CoverageStatus::Partial,
            unavailable_capability: Some(capability.to_owned()),
        });
    }
    report.findings.push(ValidationFinding::IncompleteCoverage {
        capability: "Core §62 global root coverage".to_owned(),
        status: CoverageStatus::Partial,
    });

    assess_resources(
        request.resource_verification_depth,
        &resource_ids,
        boundary,
        &mut report,
    );

    let checked_invalid = report.findings.iter().any(|finding| {
        matches!(
            finding,
            ValidationFinding::MetadataIntegrityFailure { .. }
                | ValidationFinding::IdentityMismatch { .. }
                | ValidationFinding::ProjectMismatch { .. }
                | ValidationFinding::Cycle { .. }
        )
    });
    let metadata_coverage_sufficient =
        report.coverage.object_enumeration == CoverageStatus::Complete;
    let history_coverage_sufficient = metadata_coverage_sufficient
        && report.coverage.metadata_resolution == CoverageStatus::Complete
        && report.coverage.declared_boundaries == CoverageStatus::Complete
        && report
            .coverage
            .required_roots
            .iter()
            .all(|provider| provider.status == CoverageStatus::Complete);
    let has_body_unknown = report.findings.iter().any(|finding| {
        matches!(
            finding,
            ValidationFinding::SchemaUnavailable { .. }
                | ValidationFinding::SchemaOrAdmissionFailure { .. }
        )
    });
    report.metadata_integrity = if checked_invalid {
        MetadataIntegrity::Invalid
    } else if !metadata_coverage_sufficient
        || has_body_unknown
        || has_unresolved
        || has_declared
        || has_indeterminate_reference
        || boundary_provider_failed
    {
        MetadataIntegrity::Indeterminate
    } else {
        MetadataIntegrity::Valid
    };

    let has_schema_unknown = report.findings.iter().any(|finding| {
        matches!(
            finding,
            ValidationFinding::SchemaUnavailable { .. }
                | ValidationFinding::SchemaOrAdmissionFailure { .. }
        )
    });
    report.history_completeness = if has_unresolved {
        HistoryCompleteness::Unresolved
    } else if has_declared {
        HistoryCompleteness::DeclaredIncomplete
    } else if !history_coverage_sufficient
        || has_indeterminate_reference
        || boundary_provider_failed
        || has_schema_unknown
        || report.coverage.declared_boundaries != CoverageStatus::Complete
    {
        HistoryCompleteness::NotAssessed
    } else {
        HistoryCompleteness::Complete
    };

    Ok(report)
}

fn assess_resources(
    depth: ResourceVerificationDepth,
    resources: &BTreeSet<ResourceId>,
    boundary: &dyn RepositoryValidationBoundary,
    report: &mut ValidationReport,
) {
    for resource in resources {
        let result = if depth == ResourceVerificationDepth::MetadataOnly {
            ResourceResult {
                identifier: *resource,
                state: ResourceState::NotChecked,
                method: None,
                strength: None,
            }
        } else {
            match boundary.verify_resource(*resource, depth) {
                Ok(ResourceVerification::Available { method, strength })
                    if depth != ResourceVerificationDepth::DeepResources
                        || strength == VerificationStrength::FullContent =>
                {
                    ResourceResult {
                        identifier: *resource,
                        state: ResourceState::Available,
                        method: Some(method),
                        strength: Some(strength),
                    }
                }
                Ok(ResourceVerification::UnavailableOrNotLocallyMaterialised) => {
                    report
                        .findings
                        .push(ValidationFinding::ResourceUnavailable {
                            resource: *resource,
                        });
                    ResourceResult {
                        identifier: *resource,
                        state: ResourceState::UnavailableOrNotLocallyMaterialised,
                        method: None,
                        strength: None,
                    }
                }
                Ok(ResourceVerification::Corrupt {
                    method,
                    strength: strength @ VerificationStrength::FullContent,
                }) => {
                    report.findings.push(ValidationFinding::ResourceCorrupt {
                        resource: *resource,
                        method: method.clone(),
                    });
                    ResourceResult {
                        identifier: *resource,
                        state: ResourceState::Corrupt,
                        method: Some(method),
                        strength: Some(strength),
                    }
                }
                Ok(
                    ResourceVerification::Corrupt { .. } | ResourceVerification::Available { .. },
                ) => {
                    report.coverage.resource_verification = Some(CoverageStatus::Partial);
                    report.findings.push(ValidationFinding::ProviderFailure {
                        capability: "resource_verification".to_owned(),
                        message: "provider did not meet the requested verification strength"
                            .to_owned(),
                    });
                    ResourceResult {
                        identifier: *resource,
                        state: ResourceState::NotChecked,
                        method: None,
                        strength: None,
                    }
                }
                Err(error) => {
                    report.coverage.resource_verification = Some(CoverageStatus::Partial);
                    report.findings.push(ValidationFinding::ProviderFailure {
                        capability: error.capability,
                        message: error.message,
                    });
                    ResourceResult {
                        identifier: *resource,
                        state: ResourceState::NotChecked,
                        method: None,
                        strength: None,
                    }
                }
            }
        };
        report.resources.push(result);
    }
    if depth != ResourceVerificationDepth::MetadataOnly
        && (report.coverage.resource_verification == Some(CoverageStatus::Unavailable)
            || resources.is_empty())
    {
        report.coverage.resource_verification = Some(CoverageStatus::Complete);
    }
}

fn add_cycles(objects: &[MetadataCandidate], findings: &mut Vec<ValidationFinding>) {
    let object_ids = objects
        .iter()
        .map(|object| object.identifier)
        .collect::<BTreeSet<_>>();
    let mut edges = BTreeMap::<MetadataIdentifier, Vec<MetadataIdentifier>>::new();
    for object in objects {
        if object.body_identifier != Ok(object.identifier) {
            continue;
        }
        let targets = object
            .required_references
            .iter()
            .map(|reference| reference.target)
            .filter(|target| object_ids.contains(target))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        edges.insert(object.identifier, targets);
    }

    let mut complete = BTreeSet::new();
    let mut cycles = BTreeSet::new();
    for root in object_ids {
        if complete.contains(&root) {
            continue;
        }
        let mut active = BTreeSet::new();
        let mut stack = vec![(root, false)];
        while let Some((node, exiting)) = stack.pop() {
            if exiting {
                active.remove(&node);
                complete.insert(node);
                continue;
            }
            if active.contains(&node) {
                cycles.insert(node);
                continue;
            }
            if complete.contains(&node) {
                continue;
            }
            active.insert(node);
            stack.push((node, true));
            if let Some(targets) = edges.get(&node) {
                stack.extend(targets.iter().rev().map(|target| (*target, false)));
            }
        }
    }
    findings.extend(
        cycles
            .into_iter()
            .map(|object| ValidationFinding::Cycle { object }),
    );
}

const fn historical_identifier(identifier: HistoricalId) -> MetadataIdentifier {
    match identifier {
        HistoricalId::Revision(id) => MetadataIdentifier::Revision(id),
        HistoricalId::ProjectState(id) => MetadataIdentifier::ProjectState(id),
        HistoricalId::ComponentState(id) => MetadataIdentifier::ComponentState(id),
        HistoricalId::AdapterState(id) => MetadataIdentifier::AdapterState(id),
    }
}

fn push_provider_failure(report: &mut ValidationReport, error: ProviderFailure) {
    report.findings.push(ValidationFinding::ProviderFailure {
        capability: error.capability.clone(),
        message: error.message,
    });
    report
        .coverage
        .unavailable_capabilities
        .push(error.capability);
}
