# Design: @cms/docs-ui Configuration Cleanup and Self-Contained Types

## Overview
The `@cms/docs-ui` package was copied from the MedusaJS repository and contains legacy configuration files, incorrect build settings, and incomplete type separation. This design refactors `@cms/docs-ui` to be fully self-contained with minimal internal types, eliminates all obsolete Medusa build configs, fixes TypeScript compiler and bundle settings, and restores workspace `@cms/types` integrity.

## Problem Statement
1. **Legacy Configuration Artifacts**:
   - `tsconfig.cjs.json`, `tsconfig.esm.json`, and `tsconfig.tests.json` are Medusa-specific legacy files not used by `tsdown`.
   - `tsconfig.json` extends `@cms/tsconfig/react.json` which includes `"types": ["vite/client", "node"]`. Because `docs-ui` does not depend on `vite`, `tsc` fails with `TS2688: Cannot find type definition file for 'vite/client'`.
   - Test files and mock files (`__tests__`, `__mocks__`) are not excluded from `tsconfig.json`, causing `tsdown` dts generation errors (`TS2883`) during build.
2. **Type Coupling and Gaps**:
   - In commit `08e8866`, `menu.ts`, `navigation.ts`, `navigation-dropdown.ts`, `sidebar.ts`, and `toc.ts` were moved into `packages/docs-ui/src/types/`.
   - However, other components in `packages/docs-ui` still reference `DocsConfig`, `FrontMatter`, `Workflow`, `DocBlock`, `Ref`, etc., which were not present in `packages/docs-ui/src/types/`.
   - Furthermore, `packages/types/src/index.ts` still re-exports those 5 modules, breaking `@cms/types` and downstream consumers.
3. **Redundant Dependencies**:
   - `@cms/types` is redundantly listed in both `dependencies` and `devDependencies` of `@cms/docs-ui`.
   - Unused build dependencies like `cpy-cli` remain in `devDependencies`.

## Architecture & Implementation Plan

### 1. Legacy Configuration Cleanup (`packages/docs-ui`)
- Delete obsolete files:
  - `packages/docs-ui/tsconfig.cjs.json`
  - `packages/docs-ui/tsconfig.esm.json`
  - `packages/docs-ui/tsconfig.tests.json`
- Update `packages/docs-ui/tsconfig.json`:
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
- Update `packages/docs-ui/package.json`:
  - Remove `@cms/types` from `dependencies` and `devDependencies`.
  - Remove `cpy-cli` from `devDependencies`.

### 2. Self-Contained Minimal Types (`packages/docs-ui/src/types`)
Define minimal required types directly in `packages/docs-ui/src/types/`:
- **`config.ts`**:
  ```ts
  export type DocsConfig = {
    baseUrl?: string;
    basePath?: string;
    sidebars?: any[];
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
- **`frontmatter.ts`**:
  ```ts
  export type FrontMatter = {
    title?: string;
    description?: string;
    slug?: string;
    [key: string]: any;
  };
  ```
- **`workflow.ts`**:
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
- **`reference.ts`**:
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
- **`announcement.ts`**:
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
- **`api-testing.ts`**:
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
- **`index.ts`**: Re-export all type modules with ESM extensions:
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

### 3. Internal Path Corrections (`packages/docs-ui/src`)
- Fix `packages/docs-ui/src/utils/get-navbar-items.ts` line 1:
  Change `import type { MenuItem, NavigationItem } from '../../types';` to `import type { MenuItem, NavigationItem } from '../types';`.
- Verify all relative `../../types` imports in `components`, `providers`, and `hooks` resolve cleanly to `src/types/index.ts`.

### 4. Restore Workspace `@cms/types`
- Place copies of `menu.ts`, `navigation.ts`, `navigation-dropdown.ts`, `sidebar.ts`, and `toc.ts` into `packages/types/src/`.
- In `packages/types/src/general.ts` and `packages/types/src/ui.ts`, ensure `import type React from 'react';` is present so `React.ReactNode` resolves without global namespace errors.

## Verification
1. `bun --filter @cms/docs-ui typecheck`: passes with 0 errors.
2. `bun --filter @cms/docs-ui build`: `tsdown` emits ESM bundles and `.d.ts` declarations without errors.
3. `bun --filter @cms/docs-ui test`: vitest suite executes.
4. `bun --filter @cms/types typecheck`: passes with 0 errors.
5. `bun typecheck`: workspace typecheck succeeds.
