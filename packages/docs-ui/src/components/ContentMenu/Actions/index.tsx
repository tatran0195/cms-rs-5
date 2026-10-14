'use client';

import { BroomSparkle, MarkdownSolid } from '@cms/icons';
import { useChat } from '@kapaai/react-sdk';
import { Link } from '@/components/Link';
import { usePathname } from '@/hooks/use-docs-router';
import { useMemo } from 'react';
import { useAiAssistant } from '@/providers/AiAssistant';
import { useSiteConfig } from '@/providers/SiteConfig';
import { ContentMenuSection } from '../Section';

export const ContentMenuActions = () => {
  const {
    config: { baseUrl, basePath, features, contentMenuSections },
  } = useSiteConfig();
  const pathname = usePathname();
  const { setChatOpened } = useAiAssistant();
  const { isGeneratingAnswer, isPreparingAnswer, submitQuery } = useChat();
  const loading = useMemo(() => isGeneratingAnswer || isPreparingAnswer, [isGeneratingAnswer, isPreparingAnswer]);
  const pageUrl = `${baseUrl}${basePath || ''}${pathname}`.replace(/\/$/, '');

  const handleAiAssistantClick = () => {
    if (loading) {
      return;
    }
    submitQuery(`Explain the page ${pageUrl}`);
    setChatOpened(true);
  };

  return (
    <ContentMenuSection title="Shortcuts" hideChildrenDivider={!contentMenuSections?.products && !contentMenuSections?.whatsNew}>
      <div className="flex flex-col gap-docs_0.5 p-docs_1">
        <Link
          className="flex items-center gap-docs_0.5 text-cms-fg-subtle text-x-small-plus hover:text-cms-fg-base"
          href={`${pageUrl}/index.html.md`}
          data-testid="markdown-link"
        >
          <MarkdownSolid width={15} height={15} />
          View as Markdown
        </Link>
        {features?.aiAssistant && (
          <button
            className="appearance-none p-0 flex items-center gap-docs_0.5 text-cms-fg-subtle text-x-small-plus hover:text-cms-fg-base"
            onClick={handleAiAssistantClick}
            data-testid="ai-assistant-button"
          >
            <BroomSparkle width={15} height={15} />
            Explain this page
          </button>
        )}
      </div>
    </ContentMenuSection>
  );
};
