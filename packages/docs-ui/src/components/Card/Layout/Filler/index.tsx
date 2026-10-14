import clsx from 'clsx';
import { Link } from '@/components/Link';
import type { CardProps } from '../../../..';

export const CardFillerLayout = ({ text, href, className }: CardProps) => (
  <div
    className={clsx(
      'flex justify-center items-center w-full',
      'gap-docs_0.75 px-docs_0.75 py-docs_0.5 rounded-docs_DEFAULT',
      'border border-dashed border-cms-border-strong',
      'bg-cms-bg-component text-cms-fg-subtle',
      className,
    )}
  >
    <div>
      <span className="text-compact-small">{text}</span>
      {href && (
        <>
          {' '}
          <Link href={href} className="text-compact-small-plus">
            Show All↗
          </Link>
        </>
      )}
    </div>
  </div>
);
