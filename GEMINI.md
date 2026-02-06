# Header Context

If a user mentions a "plan" or asks about the plan, and they have used the header extension in the current session, they are likely referring to `header/tracks.md` or one of the track plans (`header/tracks/<track_id>/plan.md`).

---

## Universal File Resolution Protocol

**PROTOCOL: How to locate files.**

To find a file (e.g., "Product Definition") within a specific context (Project Root or a specific Track), follow these steps in order.

### 1. Identify Index

Determine the relevant index file:

- **Project Context:** `header/index.md`
- **Track Context:**
  1. Resolve and read the **Tracks Registry** via Project Context (`header/tracks.md`)
  2. Find the entry for the specific `<track_id>`
  3. Follow the linked folder to locate the track directory
  4. Read `<track_folder>/index.md`
  5. **Fallback:**  
     If the track is not yet registered or the link is broken:
     - Resolve the **Tracks Directory** via Project Context
     - Use `header/tracks/<track_id>/index.md`

### 2. Check Index

Read the resolved `index.md` file and look for a link with a matching or semantically similar label.

### 3. Resolve Path

If a link is found, resolve its path **relative to the directory containing the `index.md` file**.

*Example:*  
`header/index.md` → `./workflow.md`  
→ `header/workflow.md`

### 4. Fallback

If the index file is missing or the link is absent, use the standard default paths below.

### 5. Verify

You MUST verify that the resolved file actually exists on disk before using it.

---

## Standard Default Paths (Project)

- **Product Definition:** `header/product.md`
- **Tech Stack:** `header/tech-stack.md`
- **Workflow:** `header/workflow.md`
- **Product Guidelines:** `header/product-guidelines.md`
- **Tracks Registry:** `header/tracks.md`
- **Tracks Directory:** `header/tracks/`

---

## Standard Default Paths (Track)

- **Specification:** `header/tracks/<track_id>/spec.md`
- **Implementation Plan:** `header/tracks/<track_id>/plan.md`
- **Metadata:** `header/tracks/<track_id>/metadata.json`

---

## Skill Discovery

If a task requires specialized capability, discover available skills by reading `skills/skills.json` and select a skill by metadata only. Do NOT inspect skill internals unless explicitly instructed.

---

## Response Language

Responses MUST use the same primary language as the user's input, unless the user explicitly requests otherwise.
