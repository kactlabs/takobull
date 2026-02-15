---
inclusion: manual
---

# Migration Task Context

When the user mentions "migration", "migraiton task", or "mg" in the first line of input:

## Reference Folder
- Use `pkg` as a reference folder to understand "PicoClaw" Go files
- These files are for reference purposes only

## Important Constraint
- **Do NOT edit any Go files under the `pkg` folder**
- Only read and reference these files to understand the existing implementation
- All implementation work should be done in the Rust codebase under `src`

## Rationale
The `pkg` folder contains the original Go implementation of PicoClaw. When working on migration tasks, understanding the Go implementation helps ensure feature parity and correct behavior in the Rust port, but we should not modify the reference implementation.
