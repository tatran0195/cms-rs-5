import type { IconProps } from '@cms/icons/dist/types';
import clsx from 'clsx';
import type React from 'react';

export type BorderedIconProps = {
  icon?: string;
  IconComponent?: React.FC<IconProps> | null;
  wrapperClassName?: string;
  iconWrapperClassName?: string;
  iconClassName?: string;
  iconColorClassName?: string;
  iconWidth?: number;
  iconHeight?: number;
} & React.HTMLAttributes<HTMLSpanElement>;

export const BorderedIcon = ({
  icon = '',
  IconComponent = null,
  iconWrapperClassName,
  iconClassName,
  iconColorClassName = '',
  wrapperClassName,
  iconWidth = 28,
  iconHeight = 28,
  ...props
}: BorderedIconProps) => (
  <span
    className={clsx(
      'rounded-docs_sm p-docs_0.125 bg-cms-bg-base inline-flex items-center justify-center',
      'shadow-border-base dark:shadow-border-base-dark',
      iconWrapperClassName,
    )}
    {...props}
  >
    <span className={clsx('rounded-docs_xs', wrapperClassName)}>
      {!IconComponent && (
        <img src={icon || ''} className={clsx(iconClassName, 'bordered-icon rounded-docs_xs')} width={iconWidth} height={iconHeight} alt="" />
      )}
      {IconComponent && <IconComponent className={clsx('text-cms-fg-subtle rounded-docs_xs', iconClassName, 'bordered-icon', iconColorClassName)} />}
    </span>
  </span>
);
