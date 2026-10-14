# Migrate @cms/docs-ui Navigation to @tanstack/react-router Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Completely eliminate Next.js dependencies from `@cms/docs-ui` and replace them with `@tanstack/react-router` and safe browser fallbacks.

**Architecture:** Create a unified router hook utility (`use-docs-router.ts`) providing `usePathname()` and `useRouter()` that dynamically use TanStack Router when rendered inside a `RouterProvider` and gracefully fall back to `window.location` outside router contexts. Migrate `<Link>` to support `href` and `to` backed by TanStack Router and standard `<a>` tags. Clean up Next.js App Router layout components (`BareboneLayout`) and remove `next` and `@next/third-parties`.

**Tech Stack:** React 18, TypeScript 5.7+, `@tanstack/react-router` 1.170.x, `tsdown`, `vitest`.

## Global Constraints

- Monorepo package manager is `bun`.
- Workspace `@tanstack/react-router` version is `^1.170.0`.
- Zero imports of `next/*` or `@next/*` remaining in `packages/docs-ui`.
- All navigation components must gracefully fall back when mounted outside a `RouterProvider` (no uncaught `useRouter must be used within a RouterProvider` errors in standalone tests or component previews).
- `bun --filter @cms/docs-ui typecheck` and `bun --filter @cms/docs-ui build` must pass cleanly with 0 errors.

---

### Task 1: Package Dependencies (`package.json`)

**Files:**
- Modify: `packages/docs-ui/package.json`

**Interfaces:**
- Produces: `@cms/docs-ui/package.json` with `@tanstack/react-router` in `peerDependencies` and `devDependencies`, and no `next` or `@next/third-parties` dependencies.

- [ ] **Step 1: Update `packages/docs-ui/package.json`**

In `packages/docs-ui/package.json`:
1. Remove `"@next/third-parties": "16.4.0"` from `"dependencies"`.
2. Remove `"next": "16.4.0"` from `"devDependencies"`.
3. Add `"@tanstack/react-router": "^1.170.0"` to `"devDependencies"`.
4. In `"peerDependencies"`:
   - Remove `"next": "*"`.
   - Add `"@tanstack/react-router": ">=1.0.0"`.
5. Add `"peerDependenciesMeta"`:
   ```json
   "peerDependenciesMeta": {
     "@tanstack/react-router": {
       "optional": true
     }
   }
   ```

- [ ] **Step 2: Install dependencies with bun**

Run: `bun install`
Expected: Installs dependencies and updates `bun.lock` cleanly.

- [ ] **Step 3: Commit dependency changes**

```bash
git add packages/docs-ui/package.json bun.lock
git commit -m "chore(docs-ui): replace next dependencies with @tanstack/react-router"
```

---

### Task 2: Router Abstraction & Safe Hooks (`use-docs-router.ts`)

**Files:**
- Create: `packages/docs-ui/src/hooks/use-docs-router.ts`
- Test: `packages/docs-ui/src/hooks/__tests__/use-docs-router.test.tsx`
- Modify: `packages/docs-ui/src/hooks/index.ts`
- Modify: `packages/docs-ui/src/index.ts`

**Interfaces:**
- Produces:
  - `usePathname(): string`
  - `useRouter(): { push(to: string, options?: { scroll?: boolean }): void; replace(to: string, options?: { scroll?: boolean }): void; back(): void }`
  - `useDocsNavigate(): (to: string, options?: { replace?: boolean; scroll?: boolean }) => void`

- [ ] **Step 1: Write tests for `use-docs-router`**

Create `packages/docs-ui/src/hooks/__tests__/use-docs-router.test.tsx`:
```tsx
import { renderHook, act } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { usePathname, useRouter } from '../use-docs-router';

describe('use-docs-router safe fallbacks', () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  it('usePathname returns window.location.pathname outside RouterProvider', () => {
    const { result } = renderHook(() => usePathname());
    expect(typeof result.current).toBe('string');
  });

  it('useRouter provides push, replace, back outside RouterProvider without crashing', () => {
    const { result } = renderHook(() => useRouter());
    expect(typeof result.current.push).toBe('function');
    expect(typeof result.current.replace).toBe('function');
    expect(typeof result.current.back).toBe('function');

    act(() => {
      result.current.push('/test-path');
      result.current.replace('/replace-path');
      result.current.back();
    });
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `bun --filter @cms/docs-ui test -- run src/hooks/__tests__/use-docs-router.test.tsx`
Expected: FAIL ("Cannot find module '../use-docs-router'")

- [ ] **Step 3: Implement `use-docs-router.ts`**

Create `packages/docs-ui/src/hooks/use-docs-router.ts`:
```tsx
import { useLocation, useNavigate } from '@tanstack/react-router';
import { useCallback } from 'react';

/**
 * Returns the current route pathname.
 * Uses TanStack Router's `useLocation` when mounted inside a `RouterProvider`,
 * and falls back safely to `window.location.pathname` otherwise.
 */
export function usePathname(): string {
  try {
    const location = useLocation();
    return location.pathname;
  } catch {
    if (typeof window !== 'undefined') {
      return window.location.pathname;
    }
    return '/';
  }
}

export type DocsRouter = {
  push: (to: string, options?: { scroll?: boolean }) => void;
  replace: (to: string, options?: { scroll?: boolean }) => void;
  back: () => void;
};

/**
 * Provides router navigation methods (`push`, `replace`, `back`).
 * Delegates to TanStack Router's `useNavigate` when available,
 * falling back safely to window.location outside router contexts.
 */
export function useRouter(): DocsRouter {
  let navigate: ReturnType<typeof useNavigate> | null = null;
  try {
    navigate = useNavigate();
  } catch {
    navigate = null;
  }

  const push = useCallback(
    (to: string, options?: { scroll?: boolean }) => {
      if (navigate) {
        navigate({ to, ...options });
      } else if (typeof window !== 'undefined') {
        if (to.startsWith('#')) {
          window.location.hash = to;
        } else {
          window.location.assign(to);
        }
      }
    },
    [navigate],
  );

  const replace = useCallback(
    (to: string, options?: { scroll?: boolean }) => {
      if (navigate) {
        navigate({ to, replace: true, ...options });
      } else if (typeof window !== 'undefined') {
        if (to.startsWith('#')) {
          window.location.hash = to;
        } else {
          window.location.replace(to);
        }
      }
    },
    [navigate],
  );

  const back = useCallback(() => {
    if (typeof window !== 'undefined') {
      window.history.back();
    }
  }, []);

  return { push, replace, back };
}
```

- [ ] **Step 4: Export from hooks and index**

In `packages/docs-ui/src/hooks/index.ts`:
Add:
```tsx
export * from './use-docs-router';
```

In `packages/docs-ui/src/index.ts`:
Ensure `export * from './hooks';` exports `usePathname` and `useRouter`.

- [ ] **Step 5: Run tests to verify they pass**

Run: `bun --filter @cms/docs-ui test -- run src/hooks/__tests__/use-docs-router.test.tsx`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add packages/docs-ui/src/hooks/use-docs-router.ts packages/docs-ui/src/hooks/__tests__/use-docs-router.test.tsx packages/docs-ui/src/hooks/index.ts packages/docs-ui/src/index.ts
git commit -m "feat(docs-ui): add use-docs-router safe navigation hooks"
```

---

### Task 3: Migrate Link & LinkButton Components

**Files:**
- Modify: `packages/docs-ui/src/components/Link/index.tsx`
- Modify: `packages/docs-ui/src/components/LinkButton/index.tsx`
- Test: `packages/docs-ui/src/components/Link/__tests__/index.test.tsx`
- Test: `packages/docs-ui/src/components/LinkButton/__tests__/index.test.tsx`

**Interfaces:**
- Produces:
  - `LinkProps`: supports `href`, `to`, `variant`, `withIcon`, HTML anchor props, and optional TanStack router props.
  - `<Link>`: renders `<a>` for external URLs or fallback, `<TanStackLink>` when inside router context.
  - `<LinkButton>`: button styled link built on top of `<Link>`.

- [ ] **Step 1: Write test for updated `<Link>`**

Update `packages/docs-ui/src/components/Link/__tests__/index.test.tsx`:
```tsx
import { render, screen } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import { Link } from '../index';

describe('Link component', () => {
  it('renders internal link correctly with fallback <a> tag', () => {
    render(<Link href="/docs/intro">Intro</Link>);
    const anchor = screen.getByRole('link', { name: 'Intro' });
    expect(anchor).toBeInTheDocument();
    expect(anchor).toHaveAttribute('href', '/docs/intro');
  });

  it('renders with to prop', () => {
    render(<Link to="/docs/quickstart">Quickstart</Link>);
    const anchor = screen.getByRole('link', { name: 'Quickstart' });
    expect(anchor).toHaveAttribute('href', '/docs/quickstart');
  });

  it('renders external links with target="_blank" and rel="noopener noreferrer"', () => {
    render(<Link href="https://example.com">External</Link>);
    const anchor = screen.getByRole('link', { name: 'External' });
    expect(anchor).toHaveAttribute('href', 'https://example.com');
    expect(anchor).toHaveAttribute('target', '_blank');
    expect(anchor).toHaveAttribute('rel', 'noopener noreferrer');
  });

  it('strips /page.mdx from internal links', () => {
    render(<Link href="/docs/guide/page.mdx#section">Guide</Link>);
    const anchor = screen.getByRole('link', { name: 'Guide' });
    expect(anchor).toHaveAttribute('href', '/docs/guide#section');
  });
});
```

- [ ] **Step 2: Update `packages/docs-ui/src/components/Link/index.tsx`**

Replace implementation:
```tsx
import { TriangleRightMini } from '@cms/icons';
import { Link as TanStackLink } from '@tanstack/react-router';
import clsx from 'clsx';
import type React from 'react';

export type LinkProps = React.AnchorHTMLAttributes<HTMLAnchorElement> & {
  href?: string;
  to?: string;
  children?: React.ReactNode;
  className?: string;
  withIcon?: boolean;
  variant?: 'default' | 'content';
  replace?: boolean;
  scroll?: boolean;
};

const HASH_REGEX = /#.*$/;
const EXTERNAL_REGEX = /^(https?:|\/\/|mailto:|tel:)/i;

export const Link = ({
  href,
  to,
  children,
  className,
  withIcon = false,
  variant = 'default',
  replace,
  ...rest
}: LinkProps) => {
  let targetUrl = to || href || '';

  const hash = targetUrl.match(HASH_REGEX)?.[0] || '';
  const urlWithoutHash = targetUrl.replace(HASH_REGEX, '');
  if (urlWithoutHash.endsWith('/page.mdx') || urlWithoutHash.endsWith('page.mdx')) {
    targetUrl = urlWithoutHash.replace(/\/page\.mdx$/, '').replace(/page\.mdx$/, '') + hash;
  }

  const isExternal = EXTERNAL_REGEX.test(targetUrl);
  const linkClasses = clsx(
    variant === 'default' && 'text-cms-fg-interactive hover:text-cms-fg-interactive-hover',
    variant === 'content' && [
      'underline decoration-cms-fg-muted hover:decoration-cms-fg-interactive',
      'decoration-[1.5px] font-medium transition-[text-decoration-color]',
    ],
    withIcon && 'flex gap-0.25 items-center group',
    className,
  );

  if (isExternal) {
    return (
      <a
        href={targetUrl}
        className={linkClasses}
        target={rest.target || '_blank'}
        rel={rest.rel || 'noopener noreferrer'}
        {...rest}
      >
        {children}
        {withIcon && <TriangleRightMini className="group-hover:translate-x-docs_0.125 transition-transform" />}
      </a>
    );
  }

  try {
    return (
      <TanStackLink
        to={targetUrl}
        replace={replace}
        className={linkClasses}
        {...rest}
      >
        {children}
        {withIcon && <TriangleRightMini className="group-hover:translate-x-docs_0.125 transition-transform" />}
      </TanStackLink>
    );
  } catch {
    return (
      <a href={targetUrl} className={linkClasses} {...rest}>
        {children}
        {withIcon && <TriangleRightMini className="group-hover:translate-x-docs_0.125 transition-transform" />}
      </a>
    );
  }
};
```

- [ ] **Step 3: Update `packages/docs-ui/src/components/LinkButton/index.tsx`**

Replace `next/link` types with `LinkProps`:
```tsx
import { Button, type ButtonProps } from '@/components/Button';
import { Link, type LinkProps } from '@/components/Link';
import type React from 'react';

export type LinkButtonProps = ButtonProps & {
  href?: string;
  to?: string;
  linkProps?: Partial<LinkProps>;
};

export const LinkButton = ({ href, to, linkProps, children, ...props }: LinkButtonProps) => {
  return (
    <Link href={href} to={to} {...linkProps}>
      <Button {...props}>{children}</Button>
    </Link>
  );
};
```

- [ ] **Step 4: Run Link and LinkButton tests**

Run: `bun --filter @cms/docs-ui test -- run src/components/Link/__tests__/index.test.tsx src/components/LinkButton/__tests__/index.test.tsx`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add packages/docs-ui/src/components/Link/ packages/docs-ui/src/components/LinkButton/
git commit -m "feat(docs-ui): migrate Link and LinkButton to TanStack Router with fallback"
```

---

### Task 4: Replace `next/navigation` and `next/link` across Components and Providers

**Files:**
- Modify: `packages/docs-ui/src/providers/Sidebar/index.tsx`
- Modify: `packages/docs-ui/src/providers/MainNav/index.tsx`
- Modify: `packages/docs-ui/src/providers/LearningPath/index.tsx`
- Modify: `packages/docs-ui/src/providers/Analytics/index.tsx`
- Modify: `packages/docs-ui/src/hooks/use-similar-pages/index.tsx`
- Modify: `packages/docs-ui/src/hooks/use-page-scroll-manager/index.tsx`
- Modify: `packages/docs-ui/src/hooks/use-heading-url/index.tsx`
- Modify: `packages/docs-ui/src/hooks/use-active-on-scroll/index.tsx`
- Modify: `packages/docs-ui/src/components/TypeList/Items/index.tsx`
- Modify: `packages/docs-ui/src/components/TechArticleJsonLd/index.tsx`
- Modify: `packages/docs-ui/src/layouts/barebone.tsx`
- Modify: All component files importing `next/link` (`Sidebar/Item/Link`, `Sidebar/Item/Category`, `Sidebar/Item/Sidebar`, `MainNav/*`, `Menu/*`, `Pagination/*`, `Prerequisites/*`, `WorkflowDiagram/*`, `HeadlineTags`, `ErrorPage`, `Feedback`, `EditButton`).

**Interfaces:**
- Produces: Zero imports of `next/navigation`, `next/link`, or `@next/*` across all source files in `packages/docs-ui/src`.

- [ ] **Step 1: Replace `next/navigation` in providers and hooks**

In each of the following files, replace:
```tsx
import { usePathname, useRouter } from 'next/navigation';
// or
import { usePathname } from 'next/navigation';
```
with:
```tsx
import { usePathname, useRouter } from '@/hooks/use-docs-router';
// or
import { usePathname } from '@/hooks/use-docs-router';
```
Files:
- `packages/docs-ui/src/providers/Sidebar/index.tsx`
- `packages/docs-ui/src/providers/MainNav/index.tsx`
- `packages/docs-ui/src/providers/LearningPath/index.tsx`
- `packages/docs-ui/src/providers/Analytics/index.tsx`
- `packages/docs-ui/src/hooks/use-similar-pages/index.tsx`
- `packages/docs-ui/src/hooks/use-page-scroll-manager/index.tsx`
- `packages/docs-ui/src/hooks/use-heading-url/index.tsx`
- `packages/docs-ui/src/hooks/use-active-on-scroll/index.tsx`
- `packages/docs-ui/src/components/TypeList/Items/index.tsx`
- `packages/docs-ui/src/components/TechArticleJsonLd/index.tsx`

- [ ] **Step 2: Replace `next/link` with `Link` from `@/components/Link`**

In all component files importing `import Link from 'next/link';`, replace with:
```tsx
import { Link } from '@/components/Link';
```
Files:
- `packages/docs-ui/src/components/Sidebar/Item/Link/index.tsx`
- `packages/docs-ui/src/components/Sidebar/Item/Category/index.tsx`
- `packages/docs-ui/src/components/Sidebar/Item/Sidebar/index.tsx`
- `packages/docs-ui/src/components/MainNav/index.tsx`
- `packages/docs-ui/src/components/MainNav/Version/index.tsx`
- `packages/docs-ui/src/components/MainNav/MobileMenu/Main/index.tsx`
- `packages/docs-ui/src/components/MainNav/MobileMenu/SubMenu/index.tsx`
- `packages/docs-ui/src/components/Menu/Item/index.tsx`
- `packages/docs-ui/src/components/Menu/SubMenu/index.tsx`
- `packages/docs-ui/src/components/Pagination/Card/index.tsx`
- `packages/docs-ui/src/components/Prerequisites/Item/index.tsx`
- `packages/docs-ui/src/components/WorkflowDiagram/Common/Node/index.tsx`
- `packages/docs-ui/src/components/HeadlineTags/index.tsx`
- `packages/docs-ui/src/components/ErrorPage/index.tsx`
- `packages/docs-ui/src/components/Feedback/index.tsx`
- `packages/docs-ui/src/components/EditButton/index.tsx`

- [ ] **Step 3: Refactor `packages/docs-ui/src/layouts/barebone.tsx`**

Replace `BareboneLayout` with standard React container div and remove `@next/third-parties/google`:
```tsx
import clsx from 'clsx';
import type React from 'react';

export type BareboneLayoutProps = {
  htmlClassName?: string;
  children: React.ReactNode;
  gaId?: string;
};

export const BareboneLayout = ({ htmlClassName, children, gaId }: BareboneLayoutProps) => (
  <div className={clsx('h-full w-full', htmlClassName)}>
    {children}
    {gaId && (
      <script
        async
        src={`https://www.googletagmanager.com/gtag/js?id=${gaId}`}
      />
    )}
  </div>
);
```

- [ ] **Step 4: Verify zero `next/` imports remain in `src/`**

Run: `git grep -n "from ['\"]next" packages/docs-ui/src`
Expected: 0 matches (except possible test files to be cleaned in Task 5).

- [ ] **Step 5: Commit**

```bash
git add packages/docs-ui/src/
git commit -m "refactor(docs-ui): replace next/navigation and next/link with internal router hooks and Link"
```

---

### Task 5: Test Suite Updates & Workspace Verification

**Files:**
- Modify: Vitest test files in `packages/docs-ui/src/**/__tests__/*.test.tsx` that previously mocked `next/navigation` or `next/link`.

**Interfaces:**
- Produces: Clean passing test suites without Next.js mocks.

- [ ] **Step 1: Update test mocks**

For test files mocking `next/navigation` or `next/link`:
Replace `vi.mock("next/navigation", ...)` and `vi.mock("next/link", ...)` with appropriate mocks for `@/hooks/use-docs-router` or `@/components/Link`, or rely on the safe fallback behavior.
Key files:
- `packages/docs-ui/src/providers/Sidebar/__tests__/index.test.tsx`
- `packages/docs-ui/src/providers/MainNav/__tests__/index.test.tsx`
- `packages/docs-ui/src/providers/Analytics/__tests__/index.test.tsx`
- `packages/docs-ui/src/layouts/__tests__/barebone.test.tsx`
- `packages/docs-ui/src/components/Sidebar/Item/Link/__tests__/index.test.tsx`
- `packages/docs-ui/src/components/MainNav/__tests__/index.test.tsx`
- `packages/docs-ui/src/components/Menu/__tests__/index.test.tsx`

- [ ] **Step 2: Run typecheck on `@cms/docs-ui`**

Run: `bun --filter @cms/docs-ui typecheck`
Expected: Exited with code 0

- [ ] **Step 3: Run build on `@cms/docs-ui`**

Run: `bun --filter @cms/docs-ui build`
Expected: `tsdown` emits `dist/index.js` and `dist/index.d.ts` without errors.

- [ ] **Step 4: Run unit tests on `@cms/docs-ui`**

Run: `bun --filter @cms/docs-ui test`
Expected: All tests pass.

- [ ] **Step 5: Verify typecheck on consuming apps (`apps/studio`)**

Run: `bun --filter @cms/studio typecheck`
Expected: Exited with code 0

- [ ] **Step 6: Commit test updates and clean build**

```bash
git add packages/docs-ui/
git commit -m "test(docs-ui): update test suite for tanstack-router navigation and verify build"
```
