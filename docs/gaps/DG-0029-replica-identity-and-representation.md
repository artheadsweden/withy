# DG-0029 — Resource Replica identity and representation

Status: OPEN
Classification: BLOCKS-FEATURE
Discovered by: OMVCS Lead
Discovered during: M3 bounded preflight
Date: 2026-10-10

## Relevant specifications

- `Specs/OMVCS Core Specification.md` §§31–36, 48–51, and 55.
- `Specs/OMVCS Storage Adapter Specification.md` §§16, 22–36, 59–63,
  71–75, 157–164, 176–185, and 252.
- `Specs/OMVCS Glossary.md`, Chunk, Chunk Manifest, Resource Replica,
  Storage Location, Storage Map, Replica Addition, Replica Removal, and
  Availability State.
- `Specs/OMVCS Core Invariants Specification.md`, INV-RES-001–007,
  INV-STOR-003–005, and INV-INT-001–002.
- Related open decisions: DEC-STORAGE-004, DEC-STORAGE-005,
  DEC-STORAGE-007, and DEC-STORAGE-011.

## Problem

The Specs define a Resource Replica as a verified physical copy of a
complete Resource Object at an Endpoint. They also allow different physical
representations, including Chunk storage, and require sufficient
reconstruction information. However, they do not define the provider-neutral
identity and multiplicity contract for Replica records or how a Resource
Replica record binds to a particular reconstruction representation.

The example Replica Identifier and lifecycle states are illustrative, not
a normative identity/cardinality contract. The Specs do not establish
whether distinct physical copies or distinct reconstruction representations
at an Endpoint yield distinct Replica records, or how the Replica identity
binds to the information needed to retrieve/reconstruct that copy. They do
specify that an incomplete Chunk representation is not fully available;
this gap is about the operational record identity and binding, not Chunk
identity, ordering, or integrity rules.

## Why the current specifications are insufficient

The choice affects observable Storage Map contents, Replica identity and
registration/removal/counting. Choosing a Resource-to-representation
record shape or a Replica identifier derivation would add semantics not
fixed by the conceptual examples or by the separate provider locator
decision. Core and Storage already define Resource and Chunk identity,
ordered reconstruction, verification, and incomplete/corrupt-copy handling;
those rules are not reopened here.

## Affected work

- WORK-0016 Replica model and Storage Map.
- WORK-0017 verification evidence associated with registered Replicas.
- WORK-0018 through WORK-0021 wherever providers, replication, or migration
  register, identify, or remove Replicas.

## Can unaffected work continue?

Yes. WORK-0015 may define the provider-neutral Resource/Chunk byte-I/O
contract and capability reporting without persistent Replica records,
verification-strength taxonomy, or a selected Chunking algorithm.

## Candidate directions

Non-normative alternatives for human consideration include a Resource-level
Replica with an operational ordered Chunk Manifest, or explicit
Chunk-level Replica records grouped by a Resource-level reconstruction
record. No direction is selected here.

## Required decision

Define Replica identity and cardinality, and how a Resource-level Replica
record binds to the provider-independent operational information needed to
identify and reconstruct the physical copy. Preserve the already normative
Resource/Chunk identity, reconstruction ordering, and integrity rules.

## Resolution

UNRESOLVED
