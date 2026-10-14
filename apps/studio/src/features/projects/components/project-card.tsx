import {
	Adjustments,
	CodeBranch,
	Github,
	InformationCircleSolid,
	SquareTwoStack,
} from "@cms/icons";
import { Button, Heading } from "@cms/ui";
import type { ProjectSummary } from "../../workspace";
import { ProjectPreviewThumbnail } from "./project-preview-thumbnail";

interface ProjectCardProps {
	project: ProjectSummary;
}

export function ProjectCard({ project }: ProjectCardProps) {
	const { latestActivity, endpoints } = project;

	return (
		<div className="w-full bg-ui-bg-base border border-ui-border-base rounded-xl overflow-hidden shadow-xs hover:border-ui-border-interactive transition-colors flex flex-col">
			{/* Main Content Row */}
			<div className="p-6 grid grid-cols-1 md:grid-cols-12 gap-6 items-center">
				{/* Left Preview Canvas */}
				<div className="md:col-span-5 w-full">
					<ProjectPreviewThumbnail type={project.previewType} />
				</div>

				{/* Right Project Details */}
				<div className="md:col-span-7 flex flex-col justify-between space-y-4">
					<div>
						<Heading
							level="h2"
							className="text-base font-semibold text-ui-fg-base"
						>
							{project.name}
						</Heading>

						{/* Live Endpoints */}
						<div className="mt-3 space-y-1.5">
							{endpoints.map((ep) => (
								<div key={ep.label} className="flex items-center gap-2 text-xs">
									<span className="text-ui-fg-muted min-w-[75px]">
										{ep.label}
									</span>
									<span className="inline-flex items-center gap-1.5 font-medium text-ui-fg-subtle">
										<span className="h-2 w-2 rounded-full bg-emerald-500" />
										Live
									</span>
									<span className="text-ui-fg-muted font-mono text-[11px]">
										{ep.url}
									</span>
								</div>
							))}
						</div>
					</div>

					{/* Latest Activity */}
					<div className="pt-3 border-t border-ui-border-base">
						<div className="flex items-center gap-1.5 text-xs text-ui-fg-subtle mb-1.5">
							<span className="font-medium">Latest activity</span>
							<InformationCircleSolid className="h-3.5 w-3.5 text-ui-fg-muted" />
						</div>

						<div className="flex flex-wrap items-center gap-2 text-xs">
							<div className="inline-flex items-center gap-1.5 text-ui-fg-subtle">
								<span className="h-1.5 w-1.5 rounded-full bg-emerald-500" />
								<span>{latestActivity.statusText}</span>
							</div>

							<div className="inline-flex items-center gap-1 font-mono text-[11px] text-ui-fg-subtle bg-ui-bg-subtle px-1.5 py-0.5 rounded border border-ui-border-base">
								<CodeBranch className="h-3 w-3 text-ui-fg-muted" />
								<span>{latestActivity.commitSha}</span>
							</div>
						</div>

						<div className="flex items-center gap-2 mt-2 text-xs text-ui-fg-muted">
							<span className="h-4 w-4 rounded-full bg-ui-bg-subtle border border-ui-border-base flex items-center justify-center font-semibold text-[10px] text-ui-fg-base">
								{latestActivity.authorInitials}
							</span>
							<span className="font-medium text-ui-fg-base">
								{latestActivity.authorName}
							</span>
							<span>&bull;</span>
							<span>{latestActivity.timestamp}</span>
							<span>&bull;</span>
							<span className="truncate max-w-[280px]">
								{latestActivity.commitMessage}
							</span>
						</div>
					</div>
				</div>
			</div>

			{/* Card Bottom Actions Bar */}
			<div className="bg-ui-bg-subtle px-6 py-2.5 border-t border-ui-border-base flex items-center justify-between">
				<Button variant="secondary" size="small" className="text-xs">
					Project details
				</Button>

				<div className="flex items-center gap-1">
					<button
						type="button"
						aria-label="View layout"
						className="p-1.5 text-ui-fg-muted hover:text-ui-fg-base rounded transition-colors cursor-pointer"
					>
						<SquareTwoStack className="h-4 w-4" />
					</button>
					<button
						type="button"
						aria-label="Settings"
						className="p-1.5 text-ui-fg-muted hover:text-ui-fg-base rounded transition-colors cursor-pointer"
					>
						<Adjustments className="h-4 w-4" />
					</button>
					<button
						type="button"
						aria-label="Git Repository"
						className="p-1.5 text-ui-fg-muted hover:text-ui-fg-base rounded transition-colors cursor-pointer"
					>
						<Github className="h-4 w-4" />
					</button>
				</div>
			</div>
		</div>
	);
}
