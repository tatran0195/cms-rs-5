import { useRouter as useTanStackRouter } from '@tanstack/react-router';
import { useCallback } from 'react';

/**
 * Returns the current route pathname.
 * Uses TanStack Router's state when mounted inside a `RouterProvider`,
 * and falls back safely to `window.location.pathname` otherwise.
 */
export function usePathname(): string {
  try {
    const router = useTanStackRouter({ warn: false });
    if (router?.state?.location?.pathname) {
      return router.state.location.pathname;
    }
  } catch {
    // outside router
  }

  if (typeof window !== 'undefined') {
    return window.location.pathname;
  }
  return '/';
}

export type DocsRouter = {
  push: (to: string, options?: { scroll?: boolean }) => void;
  replace: (to: string, options?: { scroll?: boolean }) => void;
  back: () => void;
};

/**
 * Provides router navigation methods (`push`, `replace`, `back`).
 * Delegates to TanStack Router when available,
 * falling back safely to window.location outside router contexts.
 */
export function useRouter(): DocsRouter {
  let router: any = null;
  try {
    router = useTanStackRouter({ warn: false });
  } catch {
    router = null;
  }

  const push = useCallback(
    (to: string, options?: { scroll?: boolean }) => {
      if (router && typeof router.navigate === 'function') {
        router.navigate({ to, ...options });
      } else if (typeof window !== 'undefined') {
        if (to.startsWith('#')) {
          window.location.hash = to;
        } else {
          window.location.assign(to);
        }
      }
    },
    [router],
  );

  const replace = useCallback(
    (to: string, options?: { scroll?: boolean }) => {
      if (router && typeof router.navigate === 'function') {
        router.navigate({ to, replace: true, ...options });
      } else if (typeof window !== 'undefined') {
        if (to.startsWith('#')) {
          window.location.hash = to;
        } else {
          window.location.replace(to);
        }
      }
    },
    [router],
  );

  const back = useCallback(() => {
    if (router && typeof router.history?.back === 'function') {
      router.history.back();
    } else if (typeof window !== 'undefined') {
      window.history.back();
    }
  }, [router]);

  return { push, replace, back };
}
