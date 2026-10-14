import { Link } from '@/components/Link';
import { GITHUB_ISSUES_LINK } from '../../constants';
import { Button } from '../Button';
import { ErrorPageIcon } from './Icon';

export const ErrorPage = () => (
  <div className="w-full h-full bg-cms-bg-subtle flex items-center justify-center">
    <div className="flex gap-docs_1.5 flex-col items-center justify-center w-fit max-w-lg">
      <ErrorPageIcon />
      <div className="flex flex-col items-center gap-docs_0.5">
        <h1 className="text-cms-fg-base text-2xl">Oops! Something went wrong.</h1>
        <span className="text-cms-fg-subtle txt-large-plus text-pretty text-center">
          Don’t worry. Our team have automatically been notified of this issue and they are working on it. Please try again later.
        </span>
      </div>
      <div className="flex items-center justify-center gap-docs_0.75">
        <Link href={GITHUB_ISSUES_LINK} target="_blank" rel="noreferrer">
          <Button variant="secondary">Report issue</Button>
        </Link>
        <Link href="/" passHref>
          <Button variant="primary">Go to homepage</Button>
        </Link>
      </div>
    </div>
  </div>
);
