# Design: Migrate @cms/docs-ui Navigation to @tanstack/react-router

**Date:** 2026-10-14  
**Status:** Approved  
**Author:** AI Pair Programmer & Workspace Team  

---

## 1. Background & Motivation

The `@cms/docs-ui` package was originally imported from the MedusaJS documentation platform. While core UI components (badges, buttons, admonitions, sidebar items) are framework-agnostic React components, navigation logic currently depends on Next.js App Router packages:
- `next/navigation` (`usePathname`, `useRouter`)
- `next/link` (`Link`, `LinkProps`)
- `@next/third-parties/google` (`GoogleAnalytics`)
- `next: 16.4.0` in `package.json` (`devDependencies` and `peerDependencies`)

The `cms-rs-4` repository is a Vite + React 18 monorepo. Its primary frontend application (`apps/studio`) uses **`@tanstack/react-router`** (`1.170.42`). There is no Next.js application in this codebase.

Retaining Next.js dependencies introduces:
1. **Unnecessary bloat**: `next: 16.4.0` is pulled into `node_modules` solely for a link component and pathname hook.
2. **Bundler friction**: Non-Next.js bundlers (like `tsdown` and Vite) require mocks or stubs for Next.js internals.
3. **Impedance mismatch**: Components inside `@cms/docs-ui` cannot participate natively in TanStack Router's type-safe routing and navigation events.

---

## 2. Goals & Non-Goals

### Goals
- **Eliminate Next.js completely**: Remove `next` and `@next/third-parties` from `packages/docs-ui/package.json`.
- **Integrate with `@tanstack/react-router`**: Add `@tanstack/react-router` as peer/dev dependency matching workspace versions (`^1.170.0`).
- **Provide Safe Navigation Hooks**: Create a centralized router hook utility (`packages/docs-ui/src/hooks/use-docs-router.ts`) providing `usePathname()` and `useRouter()` that leverage TanStack Router when mounted inside a `RouterProvider`, and gracefully fall back to `window.location` when rendered in isolated previews, standalone unit tests, or non-routed containers.
- **Unified `<Link>` Component**: Update `Link` to support both `href` and `to` properties, seamlessly delegating to TanStack Router's `<Link>` for internal routes while cleanly handling external links (`http`, `https`, `mailto`) and fallback anchor tags (`<a>`).
- **Clean Layouts**: Refactor `BareboneLayout` to eliminate Next.js App Router specific root tags (`<html>`, `<head>`) and replace `@next/third-parties/google` with a lightweight, optional script loader.
- **Maintain Workspace Integrity**: Ensure `typecheck`, `build`, and unit tests pass across `@cms/docs-ui` and consuming applications (`apps/studio`).

### Non-Goals
- Modifying backend Rust crates or API routes.
- Altering visual designs, Tailwind styling, or component class names.
- Converting TanStack Router in `apps/studio` to any other router.

---

## 3. Architecture & Detailed Design

### 3.1 Package Dependencies

In `packages/docs-ui/package.json`:
- **Remove**:
  - `dependencies`: `"@next/third-parties": "16.4.0"`
  - `devDependencies`: `"next": "16.4.0"`
  - `peerDependencies`: `"next": "*"`
- **Add**:
  - `peerDependencies`: `"@tanstack/react-router": ">=1.0.0"` (with optional peer dependency flag if applicable)
  - `devDependencies`: `"@tanstack/react-router": "^1.170.0"`

---

### 3.2 Safe Router Hooks (`packages/docs-ui/src/hooks/use-docs-router.ts`)

Create a unified navigation module:

```tsx
import { useLocation, useNavigate, useRouter as useTanStackRouter } from '@tanstack/react-router';
import { useCallback } from 'react';

export function usePathname(): string {
  try {
    // TanStack Router provides location via useLocation()
    const location = useLocation();
    return location.pathname;
  } catch {
    // Graceful fallback when rendered outside RouterProvider
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

export function useRouter(): DocsRouter {
  let navigate: ReturnType<typeof useNavigate> | null = null;
  try {
    navigate = useNavigate();
  } catch {
    navigate = null;
  }

  const push = useCallback((to: string, options?: { scroll?: boolean }) => {
    if (navigate) {
      navigate({ to, ...options });
    } else if (typeof window !== 'undefined') {
      if (to.startsWith('#')) {
        window.location.hash = to;
      } else {
        window.location.assign(to);
      }
    }
  }, [navigate]);

  const replace = useCallback((to: string, options?: { scroll?: boolean }) => {
    if (navigate) {
      navigate({ to, replace: true, ...options });
    } else if (typeof window !== 'undefined') {
      if (to.startsWith('#')) {
        window.location.hash = to;
      } else {
        window.location.replace(to);
      }
    }
  }, [navigate]);

  const back = useCallback(() => {
    if (typeof window !== 'undefined') {
      window.history.back();
    }
  }, []);

  return { push, replace, back };
}
```

---

### 3.3 Link Component Adaptation (`packages/docs-ui/src/components/Link/index.tsx`)

Update `LinkProps` and component implementation:

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

  // External URLs or mailto render standard <a>
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

  // Internal route: Try TanStack Router Link, falling back to <a> if context missing
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

---

### 3.4 Update Component References

1. **Replace `next/navigation` imports** with `from "@/hooks/use-docs-router"` (or relative paths):
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

2. **Replace `next/link` imports** with `from "@/components/Link"`:
   - `packages/docs-ui/src/components/LinkButton/index.tsx`
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

---

### 3.5 Refactor `BareboneLayout`

In `packages/docs-ui/src/layouts/barebone.tsx`:
- Remove Next.js `<html lang="en">` and `<head />` tags.
- Remove `@next/third-parties/google` import and usage.
- Return a standard container element:
  ```tsx
  import clsx from 'clsx';
  import type React from 'react';

  export type BareboneLayoutProps = {
    htmlClassName?: string;
    children: React.ReactNode;
    gaId?: string;
  };

  export const BareboneLayout = ({ htmlClassName, children, gaId }: BareboneLayoutProps) => {
    return (
      <div className={clsx('h-full w-full', htmlClassName)}>
        {children}
      </div>
    );
  };
  ```

---

## 4. Verification & Testing Strategy

1. **Unit Test Updates**:
   - Update Vitest setup and tests that previously mocked `"next/navigation"` and `"next/link"` to mock or utilize the new `@/hooks/use-docs-router` and `@/components/Link`.
2. **Typecheck**:
   - `bun --filter @cms/docs-ui typecheck`: verify 0 compiler errors.
3. **Build**:
   - `bun --filter @cms/docs-ui build`: verify clean bundling via `tsdown` without Next.js warnings or unbundled externals.
4. **App Integration**:
   - `bun --filter @cms/studio typecheck`: verify `apps/studio` remains completely compatible and compiles cleanly.
5. **Test Execution**:
   - `bun --filter @cms/docs-ui test`: run Vitest suite.
