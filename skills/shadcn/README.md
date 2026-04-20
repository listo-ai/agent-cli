# shadcn/ui

## Do
- Start with existing shadcn/ui primitives before writing custom wrappers.
- Keep composition shallow: prefer combining primitives over forking them.
- Use variants, slots, and utility helpers consistently with the surrounding codebase.
- Keep accessibility labels, keyboard behavior, and focus states intact.

## Don't
- Don't rewrite standard controls with raw Tailwind when a shadcn primitive already exists.
- Don't break Radix behaviors by stripping refs, props, or data attributes.
- Don't introduce mismatched spacing or typography tokens for one screen.

## Review Checklist
- Does the component still behave correctly with keyboard navigation?
- Are class names following the local style helpers?
- Is the new UI aligned with the app's current shadcn patterns?
