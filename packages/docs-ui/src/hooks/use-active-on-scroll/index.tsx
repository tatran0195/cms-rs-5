'use client';

import { usePathname } from '../use-docs-router';
import { useCallback, useEffect, useMemo, useState } from 'react';
import { isElmWindow, useIsBrowser, useScrollController } from '../..';
import { useMutationObserver } from '../use-mutation-observer';

export type ActiveOnScrollItem = {
  heading: HTMLHeadingElement;
  children?: ActiveOnScrollItem[];
};

export type UseActiveOnScrollProps = {
  rootElm?: Document | HTMLElement;
  enable?: boolean;
  useDefaultIfNoActive?: boolean;
  maxLevel?: number;
};

export const useActiveOnScroll = ({ rootElm, enable = true, useDefaultIfNoActive = true, maxLevel = 3 }: UseActiveOnScrollProps) => {
  const [items, setItems] = useState<ActiveOnScrollItem[]>([]);
  const [activeItemId, setActiveItemId] = useState('');
  const { scrollableElement } = useScrollController();
  const { isBrowser } = useIsBrowser();
  const _pathname = usePathname();
  const root = useMemo(() => {
    if (!enable) {
      return;
    }
    if (rootElm) {
      return rootElm;
    }

    if (!isBrowser) {
      return;
    }

    return document;
  }, [rootElm, isBrowser, enable]);
  const querySelector = useMemo(() => {
    let selector = '';
    for (let i = 2; i <= maxLevel; i++) {
      if (i > 2) {
        selector += ',';
      }
      selector += `h${i}`;
    }

    return selector;
  }, [maxLevel]);
  const getHeadingsInElm = useCallback(() => {
    if (!isBrowser || !enable) {
      return [];
    }

    const filteredHeadings: Element[] = [];
    root?.querySelectorAll(querySelector).forEach((heading) => {
      if (heading.id) {
        filteredHeadings.push(heading);
      }
    });

    return filteredHeadings;
  }, [isBrowser, root, enable, querySelector]);
  const setHeadingItems = useCallback(() => {
    if (!enable) {
      return;
    }
    const headings = getHeadingsInElm();
    const itemsToSet: ActiveOnScrollItem[] = [];

    // Nest headings by their level so deeper headings become children of the
    // nearest preceding shallower one (e.g. an h4 nests under its h3, which
    // nests under its h2). The TOC renders `children` recursively.
    const stack: { level: number; item: ActiveOnScrollItem }[] = [];
    headings?.forEach((heading) => {
      const level = Number.parseInt(heading.tagName.replace('H', ''), 10);
      const headingItem: ActiveOnScrollItem = {
        heading: heading as HTMLHeadingElement,
        children: [],
      };

      while (stack.length && stack[stack.length - 1].level >= level) {
        stack.pop();
      }
      if (stack.length) {
        stack[stack.length - 1].item.children?.push(headingItem);
      } else {
        itemsToSet.push(headingItem);
      }
      stack.push({ level, item: headingItem });
    });

    setItems(itemsToSet);
  }, [getHeadingsInElm, enable]);

  useMutationObserver({
    elm: root,
    callback: setHeadingItems,
  });

  const setActiveToClosest = useCallback(() => {
    if (!enable) {
      return;
    }
    const rootBoundingRectElm = root && 'getBoundingClientRect' in root ? root.getBoundingClientRect() : root?.body.getBoundingClientRect();

    if (rootBoundingRectElm === undefined || (rootBoundingRectElm.top < 0 && rootBoundingRectElm.bottom < 0)) {
      setActiveItemId('');
      return;
    }
    const headings = getHeadingsInElm();
    let selectedHeadingByHash: HTMLHeadingElement | undefined;
    const hash = location.hash.replace('#', '');
    let closestPositiveHeading: HTMLHeadingElement | undefined;
    let closestNegativeHeading: HTMLHeadingElement | undefined;
    let closestPositiveDistance = Number.POSITIVE_INFINITY;
    let closestNegativeDistance = Number.NEGATIVE_INFINITY;
    const halfway = isElmWindow(scrollableElement) ? scrollableElement.innerHeight / 2 : scrollableElement ? scrollableElement.scrollHeight / 2 : 0;

    const isWin = isElmWindow(scrollableElement);
    const currentScrollTop = isWin
      ? scrollableElement.scrollY
      : scrollableElement
        ? scrollableElement.scrollTop
        : 0;
    const currentClientHeight = isWin
      ? scrollableElement.innerHeight
      : scrollableElement
        ? scrollableElement.clientHeight
        : 0;
    const currentScrollHeight = isWin
      ? document.documentElement.scrollHeight
      : scrollableElement
        ? scrollableElement.scrollHeight
        : 0;

    if (currentScrollTop === 0) {
      // set the first heading as active if the scrollable element is at the top
      setActiveItemId(items.length && useDefaultIfNoActive ? items[0].heading.id : '');

      return;
    }
    if (currentScrollTop + currentClientHeight >= currentScrollHeight) {
      // set the last heading as active if the scrollable element is at the bottom
      let lastHeading = items[items.length - 1];
      while (lastHeading?.children?.length) {
        lastHeading = lastHeading.children[lastHeading.children.length - 1];
      }
      setActiveItemId(lastHeading && useDefaultIfNoActive ? lastHeading.heading.id : '');

      return;
    }

    headings?.forEach((heading) => {
      if (heading.id === hash) {
        selectedHeadingByHash = heading as HTMLHeadingElement;
      }
      const headingDistance = heading.getBoundingClientRect().top;

      if (headingDistance > 0 && headingDistance < closestPositiveDistance) {
        closestPositiveDistance = headingDistance;
        closestPositiveHeading = heading as HTMLHeadingElement;
      } else if (headingDistance < 0 && headingDistance > closestNegativeDistance) {
        closestNegativeDistance = headingDistance;
        closestNegativeHeading = heading as HTMLHeadingElement;
      }
    });

    const negativeDistanceToHalfway = closestNegativeDistance ? Math.abs(halfway + closestNegativeDistance) : 0;
    const positiveDistanceToHalfway = closestPositiveDistance ? Math.abs(halfway - closestPositiveDistance) : 0;

    const chosenClosest =
      !negativeDistanceToHalfway && !positiveDistanceToHalfway
        ? undefined
        : negativeDistanceToHalfway > positiveDistanceToHalfway
          ? closestNegativeHeading
          : closestPositiveHeading;

    setActiveItemId(
      chosenClosest
        ? (chosenClosest as HTMLHeadingElement).id
        : selectedHeadingByHash
          ? (selectedHeadingByHash as HTMLHeadingElement).id
          : items.length
            ? useDefaultIfNoActive
              ? items[0].heading.id
              : ''
            : '',
    );
  }, [
    getHeadingsInElm,
    items,
    enable,
    root,
    isElmWindow(scrollableElement) ? scrollableElement.innerHeight : undefined,
    scrollableElement && !isElmWindow(scrollableElement) ? scrollableElement.scrollHeight : undefined,
    scrollableElement && !isElmWindow(scrollableElement) ? scrollableElement.clientHeight : undefined,
    scrollableElement && !isElmWindow(scrollableElement) ? scrollableElement.scrollTop : undefined,
    useDefaultIfNoActive,
    scrollableElement,
  ]);

  useEffect(() => {
    if (!scrollableElement || !enable) {
      return;
    }

    scrollableElement.addEventListener('scroll', setActiveToClosest);

    return () => {
      scrollableElement.removeEventListener('scroll', setActiveToClosest);
    };
  }, [scrollableElement, setActiveToClosest, enable]);

  useEffect(() => {
    if (items.length && enable) {
      setActiveToClosest();
    }
  }, [items, setActiveToClosest, enable]);

  return {
    items,
    activeItemId,
  };
};
