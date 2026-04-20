# TypeScript

## Do
- Keep `tsconfig` strict and let types flow from schemas and API clients.
- Prefer narrow domain types over `any`, `unknown`, or stringly-typed objects.
- Split transport, domain mapping, and UI concerns into separate modules.
- Use `pnpm` for installs, scripts, and workspace commands.

## Don't
- Don't silence type errors with blanket casts.
- Don't duplicate API shapes that already exist in generated contracts.
- Don't mix side effects into utility modules.

## Review Checklist
- Are public exports intentional?
- Do new helpers preserve type inference?
- Are runtime guards added when inputs come from the network or user content?

