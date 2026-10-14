"use client";

import type { CloudStatus, CloudStatusIndicator } from "@/types";
import clsx from "clsx";
import { useEffect, useState } from "react";
import { CLOUD_STATUS_PAGE_URL } from "../../../constants";
import { getCloudStatus } from "../../../utils/get-cloud-status";
import { Tooltip } from "../../Tooltip";

// the full class names are used so that tailwind can pick them up
const indicatorColors: Record<CloudStatusIndicator, string> = {
  none: "bg-cms-tag-green-icon",
  maintenance: "bg-cms-tag-blue-icon",
  minor: "bg-cms-tag-orange-icon",
  major: "bg-cms-tag-orange-icon",
  critical: "bg-cms-tag-red-icon",
};

export const ContentMenuCloudStatus = () => {
  const [status, setStatus] = useState<CloudStatus | null>(null);

  useEffect(() => {
    getCloudStatus()
      .then(setStatus)
      .catch((e) => {
        // fail silently, we don't want to break the page if the status can't be loaded
        console.error("Failed to load the CMS Cloud status", e);
      });
  }, []);

  if (!status) {
    return null;
  }

  return (
    <div className="px-docs_0.5">
      <a
        href={CLOUD_STATUS_PAGE_URL}
        target="_blank"
        rel="noopener noreferrer"
        className={clsx(
          "flex w-full items-center rounded-docs_sm no-underline",
          "px-docs_0.5 py-docs_0.25",
          "text-cms-fg-subtle text-x-small-plus",
          "hover:bg-cms-bg-base-hover hover:text-cms-fg-base",
        )}
        // the row only shows the "CMS Cloud" label, so the status itself is
        // conveyed with the block's color and in the tooltip. this makes it
        // available to screen readers as well.
        aria-label={`CMS Cloud status: ${status.description}. Opens the status page in a new tab.`}
        data-testid="content-menu-cloud-status"
      >
        <Tooltip
          text={status.description}
          tooltipClassName="font-base"
          className="w-full"
          innerClassName="flex items-center gap-docs_0.5"
          place="left"
        >
          <span
            className={clsx(
              "h-docs_0.5 w-docs_0.5 flex-shrink-0 rounded-docs_xxs",
              indicatorColors[status.indicator] || "bg-cms-tag-neutral-icon",
            )}
            data-testid="content-menu-cloud-status-indicator"
          />
          CMS Cloud
        </Tooltip>
      </a>
    </div>
  );
};
