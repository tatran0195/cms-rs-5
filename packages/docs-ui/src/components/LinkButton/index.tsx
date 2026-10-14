import clsx from 'clsx';
import type React from 'react';
import { Link, type LinkProps } from '../Link';

export type LinkButtonProps = Omit<LinkProps, 'variant'> & {
  variant?: 'base' | 'interactive' | 'subtle' | 'muted';
  className?: string;
};

export const LinkButton = ({ variant = 'base', className, ...linkProps }: LinkButtonProps) => (
  <Link
    {...linkProps}
    className={clsx(
      className,
      'inline-flex justify-center items-center',
      'gap-docs_0.25 rounded-docs_xs',
      'text-compact-small-plus disabled:text-cms-fg-disabled',
      'focus:shadow-borders-focus no-underline',
      variant === 'base' && ['text-cms-fg-base hover:text-cms-fg-subtle', 'focus:text-cms-fg-base'],
      variant === 'interactive' && ['text-cms-fg-interactive hover:text-cms-interactive-hover', 'focus:text-cms-fg-interactive'],
      variant === 'subtle' && ['text-cms-fg-subtle hover:text-cms-fg-base', 'focus:text-cms-fg-subtle'],
      variant === 'muted' && ['text-cms-fg-muted hover:text-cms-fg-subtle', 'focus:text-cms-fg-muted'],
    )}
  />
);
