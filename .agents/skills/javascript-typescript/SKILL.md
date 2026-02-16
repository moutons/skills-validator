---
name: javascript-typescript
description: JavaScript/TypeScript project guidance including pnpm, feature-based directories, barrel files, strict TypeScript. Use when working on JS/TS projects.
license: Apache-2.0
metadata:
  author: moutons <sdmouton@gmail.com>
  version: "0.3.0"
---

# JavaScript/TypeScript

See [agentskills.io/specification](https://agentskills.io/specification) for skill structure. See [opencode.ai/docs/skills](https://opencode.ai/docs/skills/) for tool integration.

See `~/.agents/shared/safety.md` for safety rules. See `~/.agents/shared/architecture.md` for architecture principles.

## NEVER vs ALWAYS

**NEVER** do the following:

- NEVER use npm or yarn (always use pnpm)
- NEVER use .js files (always use TypeScript)
- NEVER use `any` type

**ALWAYS** do the following:

- ALWAYS use strict TypeScript
- ALWAYS use barrel files (index.ts)
- ALWAYS group by feature, not by type

## Language

Always use **TypeScript** (not .js files).

## Package Manager

Use **pnpm** (never npm/yarn):

```bash
pnpm install
pnpm add <package>
pnpm dlx <package>
```

## Separate src/ and dist/

```text
src/     # Source (version controlled)
dist/    # Compiled (gitignored)
```

## Feature-Based Structure

```text
src/
├── features/
│   ├── users/
│   │   ├── components/
│   │   ├── hooks/
│   │   ├── services/
│   │   └── index.ts      # Barrel export
│   └── orders/
└── shared/
```

## Barrel Files

Use `index.ts` to export public APIs cleanly.

## Separate Logic from Infrastructure

```text
src/features/users/
├── logic/      # Pure functions
├── services/   # API calls
└── components/ # UI
```

## Strict TypeScript

- `strict: true` in tsconfig
- No `any` or `as`
- Explicit return types

## Example: Feature with Barrel

```typescript
// src/features/users/types.ts
export interface User {
  id: number;
  email: string;
  name: string;
}

// src/features/users/api.ts
export async function fetchUser(id: number): Promise<User> {
  const res = await fetch(`/api/users/${id}`);
  if (!res.ok) throw new Error('Failed to fetch user');
  return res.json();
}

// src/features/users/index.ts
export * from './types';
export * from './api';

// src/features/users/components/UserCard.tsx
import { User, fetchUser } from '../';

export function UserCard({ userId }: { userId: number }) {
  const user = useAsync(() => fetchUser(userId));
  return <div>{user?.name}</div>;
}
```
