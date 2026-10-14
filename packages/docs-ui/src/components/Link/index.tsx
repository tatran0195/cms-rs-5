import { TriangleRightMini } from '@cms/icons';
import { Link as TanStackLink, useRouter } from '@tanstack/react-router';
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

  // Check if router is available before rendering TanStackLink to avoid throwing
  let hasRouter = false;
  try {
    const router = useRouter({ warn: false });
    hasRouter = Boolean(router);
  } catch {
    hasRouter = false;
  }

  if (hasRouter) {
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
  }

  return (
    <a href={targetUrl} className={linkClasses} {...rest}>
      {children}
      {withIcon && <TriangleRightMini className="group-hover:translate-x-docs_0.125 transition-transform" />}
    </a>
  );
};
