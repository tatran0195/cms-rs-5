import clsx from 'clsx';
import React, { useMemo } from 'react';
import { MarkdownContent } from '../../MarkdownContent';
import type { NoteProps } from '..';

type StringInfo = {
  allStringChildren: boolean;
  stringChildren: string[];
};

type NoteLayoutProps = NoteProps;

const PUNCTIONATIONS = ['.', ':', ';', ',', '!', '?'];

export const NoteLayout = ({ type, title, children, forceMultiline = false }: NoteLayoutProps) => {
  const getStringInfoFromChildren = (nodes: React.ReactNode): StringInfo => {
    let allStringChildren = true;
    const stringChildren: string[] = [];

    React.Children.forEach(nodes, (child) => {
      if (!allStringChildren) {
      } else if (['string', 'number'].includes(typeof child)) {
        stringChildren.push(`${child}`);
      } else if (Array.isArray(child)) {
        const childInfo = getStringInfoFromChildren(child);
        allStringChildren = childInfo.allStringChildren;
        stringChildren.push(...childInfo.stringChildren);
      } else if (React.isValidElement(child) && typeof child.props === 'object' && child.props && 'children' in child.props && child.props.children) {
        const typeStr = child.type.toString();
        const typeName = (typeof child.type === 'function' ? child.type.name : '') || typeStr;
        if (typeName.includes('InlineCode') && typeof child.props.children === 'string') {
          stringChildren.push(`\`${child.props.children}\``);
          return;
        }
        if (typeStr.includes('li')) {
          allStringChildren = false;
          return;
        }
        if ('href' in child.props && typeof child.props.children === 'string') {
          stringChildren.push(`[${child.props.children}](${child.props.href})`);
          return;
        }

        const childInfo = getStringInfoFromChildren(child.props.children as React.ReactNode);
        allStringChildren = childInfo.allStringChildren;
        stringChildren.push(...childInfo.stringChildren);
      }
    });

    return {
      allStringChildren,
      stringChildren,
    };
  };
  const { allStringChildren, stringChildren } = useMemo(() => {
    if (forceMultiline) {
      return {
        allStringChildren: false,
        stringChildren: '',
      };
    }

    const { allStringChildren, stringChildren } = getStringInfoFromChildren(children);

    return {
      allStringChildren,
      stringChildren: stringChildren.join(''),
    };
  }, [children, forceMultiline, getStringInfoFromChildren]);

  const showColon = useMemo(() => {
    const lastChar = title?.charAt(title.length - 1) || '';

    return !PUNCTIONATIONS.includes(lastChar);
  }, [title]);

  return (
    <div
      className={clsx(
        'py-[10px] px-docs_0.75 my-docs_1',
        'flex gap-docs_0.75 rounded-docs_DEFAULT items-stretch',
        'bg-cms-bg-component border border-cms-border-base',
      )}
      data-testid="note-layout"
    >
      <span
        className={clsx(
          'rounded-full w-docs_0.25',
          // TODO remove once we use the new prerequisites component across docs
          (type === 'default' || type === 'check') && 'bg-cms-tag-neutral-icon',
          (type === 'error' || type === 'warning') && 'bg-cms-tag-red-icon',
          type === 'success' && 'bg-cms-tag-green-icon',
          // TODO remove once all soon components are removed
          type === 'soon' && 'bg-cms-tag-blue-icon',
        )}
        data-testid="note-layout-indicator"
      />
      <div className="flex-1">
        <div className="text-small text-cms-fg-subtle [&_ol]:!mb-0 [&_ul]:!mb-0">
          <span className={clsx('text-small-plus text-cms-fg-base')} data-testid="note-layout-title">
            {title}
            {showColon ? ':' : ''}&nbsp;
          </span>
          {allStringChildren && (
            <MarkdownContent allowedElements={['a', 'code']} unwrapDisallowed={true}>
              {stringChildren}
            </MarkdownContent>
          )}
          {!allStringChildren && children}
        </div>
      </div>
    </div>
  );
};
