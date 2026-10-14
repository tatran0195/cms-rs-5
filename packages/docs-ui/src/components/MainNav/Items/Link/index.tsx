"use client";

import type { NavigationItemLink } from "../../types";
import clsx from "clsx";
import type React from "react";
import { LinkButton } from "../../../LinkButton";

type MainNavItemLinkProps = {
  item: NavigationItemLink;
  isActive: boolean;
  icon?: React.ReactNode;
  className?: string;
};

export const MainNavItemLink = ({
  item,
  isActive,
  icon,
  className,
}: MainNavItemLinkProps) => (
  <LinkButton
    href={item.link}
    className={clsx(
      "hover:bg-cms-button-transparent-hover",
      "rounded-docs_sm px-docs_0.5 py-docs_0.25",
      className,
    )}
    variant={isActive ? "base" : "subtle"}
  >
    {item.title}
    {icon}
  </LinkButton>
);
