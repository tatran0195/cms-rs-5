import { act, renderHook } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
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
