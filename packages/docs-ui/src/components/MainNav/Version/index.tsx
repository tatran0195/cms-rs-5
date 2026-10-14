'use state';

import clsx from 'clsx';
import { Link } from '@/components/Link';
import { useSiteConfig } from '../../../providers/SiteConfig';
import { Tooltip } from '../../Tooltip';

export const MainNavVersion = () => {
  const {
    config: { version },
  } = useSiteConfig();

  return (
    <Link
      href={version.releaseUrl}
      target="_blank"
      className={clsx(version.hide && 'hidden', 'px-docs_0.5 py-docs_0.25 hover:bg-cms-button-transparent-hover rounded-docs_sm')}
    >
      <Tooltip html="View the release notes<br/>on GitHub">
        <span className="relative text-compact-small-plus block">
          <span className="flex justify-center items-center">v{version.number}</span>
        </span>
      </Tooltip>
    </Link>
  );
};
