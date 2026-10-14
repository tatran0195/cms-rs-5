import clsx from 'clsx';
import { Link } from '@/components/Link';
import React from 'react';

type HeadlineTagsProps = {
  tags: (
    | string
    | {
        text: string;
        link: string;
      }
  )[];
  className?: string;
};

export const HeadlineTags = ({ tags, className }: HeadlineTagsProps) => (
  <div
    className={clsx(
      'flex gap-docs_0.25 flex-wrap justify-center items-center',
      'text-code-paragraph-xsmall-plus font-monospace uppercase',
      className,
    )}
  >
    {tags.map((tag, index) => (
      <React.Fragment key={index}>
        {typeof tag === 'string' ? (
          <span className=" text-cms-fg-subtle">[{tag}]</span>
        ) : (
          <Link href={tag.link} className="text-cms-fg-interactive hover:text-cms-fg-interactive-hover">
            [{tag.text}]
          </Link>
        )}
        {index !== tags.length - 1 && <span className="text-cms-fg-subtle">·</span>}
      </React.Fragment>
    ))}
  </div>
);
