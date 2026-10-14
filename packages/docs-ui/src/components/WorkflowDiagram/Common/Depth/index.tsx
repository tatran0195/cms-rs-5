"use client";

import type { WorkflowStepUi } from "../../types";
import { WorkflowDiagramLine } from "../../Common/Line";
import { WorkflowDiagramStepNode } from "../../Common/Node";

export type WorkflowDiagramDepthProps = {
  cluster: WorkflowStepUi[];
  next: WorkflowStepUi[];
};

export const WorkflowDiagramDepth = ({
  cluster,
  next,
}: WorkflowDiagramDepthProps) => (
  <div className="flex items-start">
    <div className="flex flex-col justify-center gap-y-docs_0.5">
      {cluster.map((step, index) => (
        <WorkflowDiagramStepNode key={`${step.name}-${index}`} step={step} />
      ))}
    </div>
    <WorkflowDiagramLine step={next} />
  </div>
);
