'use client';

import { ArrowUpRightOnBox, TriangleRightMini, XMark } from '@cms/icons';
import clsx from 'clsx';
import { Link } from '@/components/Link';
import { BorderedIcon } from '@/components/BorderedIcon';
import { Button } from '@/components/Button';
import type { CardProps } from '@/components/Card';
import { ThemeImage } from '@/components/ThemeImage';
import { useIsExternalLink } from '@/hooks/use-is-external-link';

export const CardLayoutMini = ({
  icon,
  image,
  themeImage,
  title,
  text,
  href,
  hrefProps = {},
  closeable = false,
  onClose,
  className,
  imageDimensions = { width: 45, height: 36 },
  iconClassName,
  cardRef,
  onClick,
}: CardProps) => {
  const isExternal = useIsExternalLink({ href });

  return (
    <div
      className={clsx(
        'relative rounded-docs_DEFAULT border-cms-fg-on-inverted border',
        'shadow-elevation-card-rest dark:shadow-elevation-card-rest-dark',
        'hover:shadow-elevation-card-hover dark:hover:shadow-elevation-card-hover-dark',
        'bg-cms-tag-neutral-bg dark:bg-cms-bg-component',
        'hover:bg-cms-tag-neutral-bg-hover dark:hover:bg-cms-bg-component-hover',
        'w-fit transition-[shadow,background]',
        className,
      )}
      ref={cardRef}
    >
      <div className={clsx('rounded-docs_DEFAULT flex gap-docs_0.75 py-docs_0.25', 'pl-docs_0.25 pr-docs_0.75 items-center')}>
        {icon && <BorderedIcon wrapperClassName={clsx('p-[4.5px] bg-cms-bg-component-hover')} IconComponent={icon} />}
        {image && (
          <img
            src={image}
            className={clsx('shadow-elevation-card-rest dark:shadow-elevation-card-rest-dark', 'rounded-docs_xs', iconClassName)}
            width={imageDimensions.width}
            height={imageDimensions.height}
            alt={title || text || ''}
            style={{
              width: `${imageDimensions.width}px`,
              height: `${imageDimensions.height}px`,
            }}
          />
        )}
        {themeImage && (
          <ThemeImage
            {...themeImage}
            className={clsx('shadow-elevation-card-rest dark:shadow-elevation-card-rest-dark', 'rounded-docs_xs', iconClassName)}
            width={imageDimensions.width}
            height={imageDimensions.height}
            alt={title || text || ''}
            style={{
              width: `${imageDimensions.width}px`,
              height: `${imageDimensions.height}px`,
            }}
          />
        )}
        <div className="flex flex-col">
          {title && (
            <span className="text-x-small-plus text-cms-fg-base" data-testid="title">
              {title}
            </span>
          )}
          {text && (
            <span className="text-x-small-plus text-cms-fg-subtle" data-testid="text">
              {text}
            </span>
          )}
        </div>
        {!closeable && (
          <span className="text-cms-fg-subtle">
            {isExternal ? <ArrowUpRightOnBox data-testid="external-icon" /> : <TriangleRightMini data-testid="internal-icon" />}
          </span>
        )}
        {href && (
          <Link
            href={href}
            className="absolute left-0 top-0 w-full h-full z-[1]"
            prefetch={false}
            {...hrefProps}
            rel={isExternal ? 'noopener noreferrer' : undefined}
            target={isExternal ? '_blank' : undefined}
            aria-label={title}
            onClick={onClick}
          />
        )}
        {closeable && (
          <Button
            variant="transparent-clear"
            onClick={onClose}
            className="!p-[2.5px] z-[2] hover:!bg-cms-button-transparent-hover focus:!shadow-none focus:!bg-transparent"
            data-testid="close-button"
          >
            <XMark data-testid="close-icon" />
          </Button>
        )}
      </div>
    </div>
  );
};
