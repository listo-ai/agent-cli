# React

## Do
- Keep components focused on rendering and move async state or orchestration into hooks.
- Prefer clear prop contracts and derived UI state over duplicated local state.
- Handle loading, empty, and error states explicitly.
- Preserve the existing design system and component primitives.

## Don't
- Don't bury business logic inside JSX branches.
- Don't add memoization by default; measure first and follow repo conventions.
- Don't create one-off visual patterns when the app already has a reusable primitive.

## Review Checklist
- Is the component easy to scan top-to-bottom?
- Can the behavior be tested without rendering the whole page?
- Does the state model avoid stale props or race-prone effects?
