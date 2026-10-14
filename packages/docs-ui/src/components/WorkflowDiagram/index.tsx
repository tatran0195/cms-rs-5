"use client";

import type { Workflow } from "../../types";
import { Suspense } from "react";
import { Loading } from "@/components/Loading";
import { WorkflowDiagramCanvas } from "./Canvas";
import { WorkflowDiagramList } from "./List";

export type WorkflowDiagramCommonOptionsProps = {
  hideLegend?: boolean;
};

export type WorkflowDiagramCommonProps = {
  workflow: Workflow;
} & WorkflowDiagramCommonOptionsProps;

export type WorkflowDiagramType = "canvas" | "list";

export type WorkflowDiagramProps = WorkflowDiagramCommonProps & {
  type?: WorkflowDiagramType;
};

export const WorkflowDiagram = ({
  type = "list",
  ...props
}: WorkflowDiagramProps) => (
  <Suspense fallback={<Loading />}>
    {type === "canvas" && <WorkflowDiagramCanvas {...props} />}
    {type === "list" && <WorkflowDiagramList {...props} />}
  </Suspense>
);
