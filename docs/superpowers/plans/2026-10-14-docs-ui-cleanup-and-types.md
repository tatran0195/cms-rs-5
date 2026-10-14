# @cms/docs-ui Configuration Cleanup and Self-Contained Types Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Clean legacy MedusaJS configuration from `@cms/docs-ui`, make its types fully self-contained without depending on `@cms/types`, and restore monorepo typecheck integrity.

**Architecture:** Remove obsolete tsconfigs, override inherited `vite/client` compiler types, exclude test mocks from `tsdown` dts generation, define minimal domain types (`DocsConfig`, `FrontMatter`, `Workflow`, `Ref`, etc.) directly inside `packages/docs-ui/src/types/`, and restore missing re-exported types in `packages/types/src/`.

**Tech Stack:** TypeScript 7, Bun 1.4, tsdown / Rolldown, React 19, Vitest.

## Global Constraints
- Do not introduce new external dependencies.
- Remove `@cms/types` dependency from `packages/docs-ui/package.json`.
- Preserve existing component functionality and exports in `@cms/docs-ui`.
- All commands run with `bun` in `d:\Workspace\Software\Cloned-Repos\cms-rs-4`.

---

### Task 1: Clean Configuration and Remove Legacy Medusa Files

**Files:**
- Delete: `packages/docs-ui/tsconfig.cjs.json`
- Delete: `packages/docs-ui/tsconfig.esm.json`
- Delete: `packages/docs-ui/tsconfig.tests.json`
- Modify: `packages/docs-ui/tsconfig.json`
- Modify: `packages/docs-ui/package.json`

**Interfaces:**
- Produces: Clean TypeScript configuration for `@cms/docs-ui` resolving node types and excluding test mocks.

- [ ] **Step 1: Delete obsolete Medusa tsconfig files**

Run command in shell to remove the three files:
```powershell
Remove-Item -Path "packages/docs-ui/tsconfig.cjs.json", "packages/docs-ui/tsconfig.esm.json", "packages/docs-ui/tsconfig.tests.json" -Force
```

- [ ] **Step 2: Update `packages/docs-ui/tsconfig.json`**

Replace content of `packages/docs-ui/tsconfig.json`:
```json
{
  "extends": "@cms/tsconfig/react.json",
  "compilerOptions": {
    "skipLibCheck": true,
    "noUncheckedIndexedAccess": false,
    "types": ["node"],
    "paths": {
      "@cms/docs-ui": ["./src/index.ts"],
      "docs-ui": ["./src/index.ts"],
      "@/*": ["./src/*"]
    }
  },
  "include": ["src/**/*"],
  "exclude": [
    "node_modules",
    "dist",
    ".cache",
    "src/**/__tests__/**",
    "src/**/__mocks__/**",
    "src/**/*.test.ts",
    "src/**/*.test.tsx"
  ]
}
```

- [ ] **Step 3: Update `packages/docs-ui/package.json`**

Remove `@cms/types` from `dependencies` and `devDependencies`, and remove `cpy-cli` from `devDependencies`.
In `packages/docs-ui/package.json`:
Lines 30-59 (`dependencies`): remove `"@cms/types": "workspace:*",`
Lines 60-76 (`devDependencies`): remove `"@cms/types": "workspace:*",` and `"cpy-cli": "^7.0.0",`

- [ ] **Step 4: Verify config changes**

Run: `bun --filter @cms/docs-ui typecheck`
Expected: TS2688 (Cannot find type definition file for 'vite/client') is GONE. Errors remaining should only be unresolved types (`DocsConfig`, `Workflow`, etc.).

- [ ] **Step 5: Commit config cleanup**

```powershell
git add packages/docs-ui/tsconfig.json packages/docs-ui/package.json
git rm -f packages/docs-ui/tsconfig.cjs.json packages/docs-ui/tsconfig.esm.json packages/docs-ui/tsconfig.tests.json
git commit -m "chore(docs-ui): clean legacy configs and package dependencies"
```

---

### Task 2: Implement Self-Contained Minimal Types in `@cms/docs-ui`

**Files:**
- Create: `packages/docs-ui/src/types/config.ts`
- Create: `packages/docs-ui/src/types/frontmatter.ts`
- Create: `packages/docs-ui/src/types/workflow.ts`
- Create: `packages/docs-ui/src/types/reference.ts`
- Create: `packages/docs-ui/src/types/announcement.ts`
- Create: `packages/docs-ui/src/types/api-testing.ts`
- Modify: `packages/docs-ui/src/types/index.ts`
- Modify: `packages/docs-ui/src/utils/get-navbar-items.ts:1`

**Interfaces:**
- Produces: Complete type exports from `packages/docs-ui/src/types/index.ts` covering all UI component requirements.

- [ ] **Step 1: Create `packages/docs-ui/src/types/config.ts`**

```ts
import type { Sidebar } from './sidebar.js';

export type DocsConfig = {
  baseUrl?: string;
  basePath?: string;
  sidebars?: Sidebar[];
  project?: {
    title?: string;
    key?: string;
  };
  reportIssueLink?: string;
  logo?: string;
  features?: {
    aiAssistant?: boolean;
    [key: string]: any;
  };
  [key: string]: any;
};
```

- [ ] **Step 2: Create `packages/docs-ui/src/types/frontmatter.ts`**

```ts
export type FrontMatter = {
  title?: string;
  description?: string;
  slug?: string;
  [key: string]: any;
};
```

- [ ] **Step 3: Create `packages/docs-ui/src/types/workflow.ts`**

```ts
export type WorkflowStepUi = {
  id: string;
  name: string;
  description?: string;
  depth?: number;
  link?: string;
  type?: string;
  status?: string;
  children?: WorkflowStepUi[];
};

export type Workflow = {
  name: string;
  steps: WorkflowStepUi[];
};
```

- [ ] **Step 4: Create `packages/docs-ui/src/types/reference.ts`**

```ts
export type Ref = {
  name?: string;
  type?: string;
  description?: string;
  path?: string;
  [key: string]: any;
};

export type DocBlock = {
  title?: string;
  description?: string;
  content?: string;
  [key: string]: any;
};

export type DocPage = {
  title?: string;
  description?: string;
  blocks?: DocBlock[];
  [key: string]: any;
};

export type DocEvent = {
  name: string;
  description?: string;
  data?: any;
  [key: string]: any;
};

export type DocWorkflowEvent = {
  workflow: string;
  event: string;
  description?: string;
};
```

- [ ] **Step 5: Create `packages/docs-ui/src/types/announcement.ts`**

```ts
export type CloudAnnouncement = {
  id?: string;
  title?: string;
  description?: string;
  date?: string;
  link?: string;
  badge?: {
    text: string;
    variant?: string;
  };
};
```

- [ ] **Step 6: Create `packages/docs-ui/src/types/api-testing.ts`**

```ts
export type ApiTestingOptions = {
  method?: string;
  url?: string;
  path?: string;
  headers?: Record<string, string>;
  body?: any;
  query?: Record<string, any>;
  param?: Record<string, any>;
};
```

- [ ] **Step 7: Update `packages/docs-ui/src/types/index.ts`**

Replace `packages/docs-ui/src/types/index.ts`:
```ts
export * from './navigation.js';
export * from './navigation-dropdown.js';
export * from './sidebar.js';
export * from './toc.js';
export * from './menu.js';
export * from './config.js';
export * from './frontmatter.js';
export * from './workflow.js';
export * from './reference.js';
export * from './announcement.js';
export * from './api-testing.js';
```

- [ ] **Step 8: Fix `packages/docs-ui/src/utils/get-navbar-items.ts` import path**

In `packages/docs-ui/src/utils/get-navbar-items.ts` line 1:
Change:
```ts
import type { MenuItem, NavigationItem } from '../../types';
```
To:
```ts
import type { MenuItem, NavigationItem } from '../types';
```

- [ ] **Step 9: Run typecheck on `@cms/docs-ui`**

Run: `bun --filter @cms/docs-ui typecheck`
Expected: Exited with code 0 (no errors).

- [ ] **Step 10: Commit self-contained types**

```powershell
git add packages/docs-ui/src/types/ packages/docs-ui/src/utils/get-navbar-items.ts
git commit -m "feat(docs-ui): make types self-contained with minimal definitions"
```

---

### Task 3: Build and Test `@cms/docs-ui`

**Files:**
- None modified directly (validates build output `packages/docs-ui/dist/`)

**Interfaces:**
- Produces: Validated `dist/index.js` and `dist/index.d.ts` generated by `tsdown`.

- [ ] **Step 1: Run build**

Run: `bun --filter @cms/docs-ui build`
Expected: Build succeeds with `tsdown` emitting ESM and `.d.ts` declarations.

- [ ] **Step 2: Run tests**

Run: `bun --filter @cms/docs-ui test`
Expected: Vitest runs test suites in `@cms/docs-ui`.

---

### Task 4: Restore Workspace `@cms/types` Integrity and Monorepo Check

**Files:**
- Create: `packages/types/src/menu.ts`
- Create: `packages/types/src/navigation.ts`
- Create: `packages/types/src/navigation-dropdown.ts`
- Create: `packages/types/src/sidebar.ts`
- Create: `packages/types/src/toc.ts`
- Modify: `packages/types/src/general.ts`
- Modify: `packages/types/src/ui.ts`

**Interfaces:**
- Produces: Fully compilable `@cms/types` satisfying downstream packages (`@cms/docs-utils`, `@cms/remark-rehype-plugins`, `apps/studio`).

- [ ] **Step 1: Restore the 5 shared type files in `packages/types/src/`**

Copy the files from `packages/docs-ui/src/types/` into `packages/types/src/`:
- `packages/types/src/menu.ts`
- `packages/types/src/navigation.ts`
- `packages/types/src/navigation-dropdown.ts`
- `packages/types/src/sidebar.ts`
- `packages/types/src/toc.ts`

Ensure `packages/types/src/sidebar.ts` and `menu.ts` include `import type React from 'react';` so `React.ReactNode` resolves.

- [ ] **Step 2: Fix `React` namespace in `packages/types/src/general.ts` and `ui.ts`**

In `packages/types/src/general.ts`:
Add `import type React from 'react';` at line 1.

In `packages/types/src/ui.ts`:
Add `import type React from 'react';` at line 1.

- [ ] **Step 3: Run typecheck on `@cms/types`**

Run: `bun --filter @cms/types typecheck`
Expected: Exited with code 0 (no errors).

- [ ] **Step 4: Run workspace-wide typecheck**

Run: `bun typecheck`
Expected: All packages pass typecheck.

- [ ] **Step 5: Commit workspace type fixes**

```powershell
git add packages/types/src/
git commit -m "fix(types): restore missing types and React imports"
```
