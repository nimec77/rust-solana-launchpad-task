# Workflow Agreement

## References

- Plan: `docs/tasklist.md`
- Architecture: `docs/vision.md`
- Task details: `docs/idea.md`

## Rules

1. **Follow the tasklist strictly** — work iteration by iteration, in order. No skipping ahead.

2. **Each iteration has 5 phases:**

   ```
   Propose → Approve → Implement → Verify → Commit
   ```

   - **Propose** — assistant presents the solution with exact code snippets and file paths. No code is written yet.
   - **Approve** — user reviews and confirms (or requests changes). Implementation begins only after explicit approval.
   - **Implement** — assistant writes the code exactly as agreed.
   - **Verify** — run the iteration's test command. Share output with user. Wait for user confirmation.
   - **Commit** — after user confirms, commit changes and update `docs/tasklist.md`:
     - Mark completed checkboxes `[x]`
     - Update progress table status (`⬜` → `✅`)

3. **Wait for confirmation at every gate** — never proceed to the next phase or iteration without explicit user approval.

4. **One commit per iteration** — commit message format: `feat: iteration N — <short description>`.

5. **If tests fail** — diagnose, propose a fix, get approval, retry. Do not move on until tests pass and user confirms.

6. **No scope creep** — only do what the current iteration requires. No refactoring, no extras.
