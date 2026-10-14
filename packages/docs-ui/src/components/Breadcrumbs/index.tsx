"use client";

import { TriangleRightMini } from "@cms/icons";
import type { Sidebar } from "../../types";
import clsx from "clsx";
import Link from "next/link";
import React, { useMemo } from "react";
import type { BreadcrumbList } from "schema-dts";
import { useSidebar } from "@/providers/Sidebar";
import { useSiteConfig } from "@/providers/SiteConfig";
import { getJsonLd } from "@/utils/get-json-ld";
import { Button } from "../Button";

type BreadcrumbItems = {
  title: string;
  link: string;
}[];

export const Breadcrumbs = () => {
  const { sidebarHistory, getSidebarFirstLinkChild, getSidebar } = useSidebar();
  const {
    config: { breadcrumbOptions, baseUrl, basePath },
  } = useSiteConfig();

  const getLinkPath = (item: Sidebar.SidebarItemLink): string => item.path;

  const breadcrumbItems = useMemo(() => {
    const items: BreadcrumbItems = [];
    if (breadcrumbOptions?.startItems) {
      items.push(...breadcrumbOptions.startItems);
    }

    sidebarHistory.forEach((sidebar_id) => {
      const sidebar = getSidebar(sidebar_id);

      if (!sidebar) {
        return;
      }

      const sidebarFirstChild = getSidebarFirstLinkChild(sidebar);

      if (!sidebarFirstChild) {
        return;
      }

      items.push({
        title: sidebar.title,
        link: getLinkPath(sidebarFirstChild),
      });
    });

    // make sure items are unique (no duplicate links)
    const uniqueItems = items.filter(
      (item, index, self) =>
        index === self.findIndex((t) => t.link === item.link),
    );
    return uniqueItems;
  }, [
    sidebarHistory,
    breadcrumbOptions,
    getSidebarFirstLinkChild,
    getSidebar,
    getLinkPath,
  ]);

  const jsonLd = useMemo(() => {
    const baseLink = `${baseUrl}${basePath}`.replace(/\/+$/, "");
    return getJsonLd<BreadcrumbList>({
      "@context": "https://schema.org",
      "@type": "BreadcrumbList",
      itemListElement: breadcrumbItems.map((item, index) => ({
        "@type": "ListItem",
        position: index + 1,
        name: item.title,
        item: item.link.startsWith("#")
          ? baseLink
          : item.link.startsWith("/")
            ? `${baseLink}${item.link}`
            : item.link,
      })),
    });
  }, [breadcrumbItems, baseUrl, basePath]);

  return (
    <div
      className={clsx(
        "flex items-center gap-docs_0.25",
        "text-cms-fg-muted text-compact-small",
        "mb-docs_1 flex-wrap",
      )}
    >
      {breadcrumbItems.map(({ title, link }, index) => (
        <React.Fragment key={link}>
          {index > 0 && <TriangleRightMini />}
          <Button
            variant="transparent-clear"
            className={clsx(
              "px-docs_0.5 py-docs_0.25",
              link === "#" && "hover:cursor-default",
              "!p-0 hover:!bg-transparent hover:!text-cms-fg-subtle",
            )}
          >
            <Link
              href={link}
              className={clsx(link === "#" && "hover:cursor-default")}
            >
              {title}
            </Link>
          </Button>
        </React.Fragment>
      ))}
      <script
        type="application/ld+json"
        dangerouslySetInnerHTML={{
          __html: jsonLd,
        }}
      />
    </div>
  );
};
