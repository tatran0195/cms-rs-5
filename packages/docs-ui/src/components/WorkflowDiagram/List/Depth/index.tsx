"use client";

import type { WorkflowStepUi } from "../../types";
import { WorkflowDiagramLine } from "../../Common/Line";
import { WorkflowDiagramStepNode } from "../../Common/Node";

export type WorkflowDiagramListDepthProps = {
  cluster: WorkflowStepUi[];
};

export const WorkflowDiagramListDepth = ({
  cluster,
}: WorkflowDiagramListDepthProps) => (
  <div className="flex items-start workflow-node-group w-fit">
    <WorkflowDiagramLine step={cluster} />
    <div className="flex flex-col justify-center gap-y-docs_0.5">
      {cluster.map((step, index) => (
        <WorkflowDiagramStepNode key={`${step.name}-${index}`} step={step} />
      ))}
    </div>
  </div>
);
