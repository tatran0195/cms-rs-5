'use client';

import { usePathname } from '../use-docs-router';
import { useMemo } from 'react';
import { useIsBrowser } from '../../providers/BrowserProvider';
import { useSiteConfig } from '../../providers/SiteConfig';

type useHeadingUrlProps = {
  id: string;
};

export const useHeadingUrl = ({ id }: useHeadingUrlProps) => {
  const { isBrowser } = useIsBrowser();
  const {
    config: { basePath },
  } = useSiteConfig();
  const pathname = usePathname();
  const headingUrl = useMemo(() => {
    const hash = `#${id}`;
    if (!isBrowser) {
      return hash;
    }

    const url = `${window.location.origin}${basePath}${pathname}`.replace(/\/$/, '');

    return `${url}${hash}`;
  }, [id, isBrowser, pathname, basePath]);

  return headingUrl;
};
