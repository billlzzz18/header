# Header Context

If a user mentions a "plan" or asks about the plan, and they have used the header extension in the current session, they are likely referring to `header/tracks.md` or one of the track plans (`header/tracks/<track_id>/plan.md`).

---

## Instruction Scope

This document defines **bootstrapping constraints and structural protocols only**.  
It establishes the **world, boundaries, and rails** in which work operates.

- Evaluated during setup and initialization.
- MUST NOT be treated as a runtime decision engine.
- MUST NOT be re-injected wholesale into every execution step.
- Commands and tasks operate **within** this world, not enforced by it.

This document is NOT a command definition and does NOT prescribe task-level behavior.

---

## Visual Repository Structure

The following diagram represents the **canonical world structure**.  
All references, resolutions, and proposals MUST align with this structure.

```text
project-root/
├─ header/
│  ├─ index.md
│  ├─ product.md
│  ├─ tech-stack.md
│  ├─ workflow.md
│  ├─ product-guidelines.md
│  ├─ tracks.md
│  └─ tracks/
│     └─ <track_id>/
│        ├─ index.md
│        ├─ spec.md
│        ├─ plan.md
│        └─ metadata.json
│
├─ skills/
│  ├─ skills.json
│  └─ <skill_name>/
│     ├─ SKILL.md
│     └─ implementation/
│
└─ README.md
```
This structure defines what exists, not what must be used.  
Anything outside this structure is considered out of world scope.

---

## Universal File Resolution Protocol

PROTOCOL: How the system understands where things are.

This protocol defines canonical structure, not runtime search behavior.  
It establishes where artifacts are expected to live when referenced.

1. Identify Index

Determine the relevant index file:

Project Context: header/index.md

Track Context:

1. Resolve and read the Tracks Registry via Project Context (header/tracks.md)

2. Find the entry for the specific `<track_id>`

3. Follow the linked folder to locate the track directory

4. Read `<track_folder>/index.md`

5. Fallback (structural only):

Resolve the Tracks Directory via Project Context  
Use `header/tracks/<track_id>/index.md`

2. Check Index

Read the resolved `index.md` file and look for a link with a matching or semantically similar label.

3. Resolve Path

If a link is found, resolve its path relative to the directory containing the `index.md` file.

4. Fallback

If the index file is missing or the link is absent, use the standard default paths below.

5. Verify

Verification confirms structural expectation, not task success.  
Failure to verify does NOT terminate work by itself.

---

## Standard Default Paths (Project)

Product Definition: `header/product.md`  
Tech Stack: `header/tech-stack.md`  
Workflow: `header/workflow.md`  
Product Guidelines: `header/product-guidelines.md`  
Tracks Registry: `header/tracks.md`  
Tracks Directory: `header/tracks/`

---

## Standard Default Paths (Track)

Specification: `header/tracks/<track_id>/spec.md`  
Implementation Plan: `header/tracks/<track_id>/plan.md`  
Metadata: `header/tracks/<track_id>/metadata.json`

---

## Standard File Format Contracts

File formats define required structure, not content or wording.  
Exact content rules are enforced by commands.

- **Track `index.md`**: MUST define purpose and references to specification and plan.
- **Track `spec.md`**: MUST define scope, constraints, and acceptance criteria.
- **Track `plan.md`**: MUST describe current state, target state, and ordered steps.
- **Skill `SKILL.md`**: MUST declare capability, inputs, outputs, and boundaries.

Files that do not satisfy their format contract are considered structurally invalid.

---

## Commitment Boundary

All generated outputs are proposals, not actions.

- Generation does NOT imply correctness.
- Generation does NOT imply acceptance.
- Generation does NOT imply execution.

A proposal becomes effective only when explicitly accepted by the user.  
The assistant has NO authority to self-commit or silently advance state.

---

## Skill Discovery
If work requires specialized capability:

- Discover available skills via `skills/skills.json`.
- Select skills by declared metadata only.
- Skill internals are opaque unless explicitly exposed.
- Skills represent capability, not authority.

If no suitable skill exists, the system transitions to proposal evaluation.

---

## Skill Invocation Constraint

Skill usage MUST remain within declared responsibility.

When a task exceeds a skill’s boundary:

- The assistant MUST NOT improvise capability.
- The assistant MUST surface the mismatch as a decision point.
- The assistant MAY propose alternatives that remain within the world.

---

## Proposal Mode

When direct continuation is not possible or when improvement opportunities are detected,  
the assistant MUST enter Proposal Mode.

In Proposal Mode, the assistant MUST:

1. Explain the observed condition in system terms.
2. Present viable options within the defined world.
3. Describe trade-offs of each option.
4. Ask a decision-oriented next-step question.

Proposal Mode enables progress without guessing or fabricating.

---

## Best Practices for Context Efficiency

To preserve focus and reduce noise:

- Do NOT load all files by default.
- Resolve and read only what is required for the current intent.
- Prefer index files before deep documents.
- Avoid re-reading unchanged artifacts.
- Treat context as a working surface, not memory.

---

## Command Readiness

This instruction set prepares the environment for command-driven execution.

- Commands are explicit events.
- Commands activate scoped behavior.
- Commands do not override the world defined here.

This document defines the rails.  
Commands decide how to move within them.

---

## Response Language

Responses MUST use the same primary language as the user's input,  
unless the user explicitly requests otherwise.