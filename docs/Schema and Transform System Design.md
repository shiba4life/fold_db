Datafold Schema and Transform System Design

⸻

Schema Field Definition and Transforms

When a schema is loaded into the system, its fields include a list of transforms. These transforms are associated directly with individual fields:

schema: {
  ...
  fields: [
    field1: {
      ...
      transforms: [
        transform1,
        transform2,
        ...
      ]
    }
  ]
}

Upon writing (updating) to a field, the system automatically schedules the associated transforms for execution.

⸻

Transform Structure

A transform is defined with the following structure:

Transform: {
  inputs: [schema1.field1, schema2.field2, ...],
  logic: "...",
  output: schemaX.fieldX,
  parsed_expression
}

The transform takes inputs from one or more schema fields, applies logical operations (defined in the logic attribute), and produces an output to a specific field in a schema.

⸻

Schema States and Properties

Schemas in Datafold can exist in one of the following states:
	1.	available — schema is present but not yet approved.
	2.	approved — schema is user-approved and can be queried, mutated, field-mapped, and have transforms executed.
	3.	blocked — schema is user-blocked. It cannot be queried or mutated, but field-mapping and transforms still operate.

Additionally, schemas can have one of the following load statuses:
	•	loaded
	•	unloaded

Note: Unloaded schemas are not queryable or mutable, but their field mappers and mutations still function.

All schemas are immutable — once written, they cannot be updated or deleted.

⸻

Field Types

Each field in a schema can be of one of the following types:
	•	Single
	•	Collection
	•	Range
	•	Hash+Range

⸻

SchemaCore API

The core API of Datafold supports the following operations:
	1.	Fetch available schemas — Load schemas from a file source.
	2.	Approve schema — Transition schema to an approved state.
	3.	Block schema — Transition schema to a blocked state.
	4.	Load schema state — Load schema state from sled.
	5.	Load available schemas — Retrieve the available schemas list from sled.

⸻

Schema Loading Sequence

The schema loading process follows this sequence:
	1.	Load available schemas from sled.
	2.	Fetch available schemas from file (e.g., example schemas).
	3.	Load schema state from sled.

⸻

User Actions

Users can perform the following actions:
	1.	Approve a schema — changes schema state to approved.
	2.	Block a schema — changes schema state to blocked.

⸻

Field Mutation and Atom Tracking

Field behaviors related to ref_atom_uuid are as follows:
	1.	When a schema is first approved, all ref_atom_uuids are null.
	2.	When a mutation occurs, a ref_atom_uuid is created, associated with an atom, and persisted across all relevant states (schema, ref_atoms, atoms).
	3.	When a transform runs and the result field lacks a ref_atom_uuid, the system generates one, creates an atom, and persists it, similar to mutation behavior.

⸻

This document defines the foundational structure for Datafold’s schema, transform, and field mutation systems, emphasizing immutability, state control, and atomic tracking.