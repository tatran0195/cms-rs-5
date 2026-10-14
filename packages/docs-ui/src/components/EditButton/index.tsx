import { ArrowUpRightOnBox } from '@cms/icons';
import clsx from 'clsx';
import { Link } from '@/components/Link';
import { EditDate } from '../EditDate';

type EditButtonProps = {
  filePath: string;
  editDate?: string;
};

export const EditButton = ({ filePath, editDate }: EditButtonProps) => (
  <div className="flex flex-wrap gap-docs_0.5 mt-docs_2 text-cms-fg-subtle">
    {editDate && <EditDate date={editDate} />}

    <Link
      href={`https://github.com/cms/medusa/edit/develop${filePath}`}
      className={clsx('flex w-fit gap-docs_0.25 items-center', 'text-cms-fg-subtle hover:text-cms-fg-base', 'text-compact-small-plus')}
      data-testid="edit-button"
    >
      <span>Edit this page</span>
      <ArrowUpRightOnBox />
    </Link>
  </div>
);
