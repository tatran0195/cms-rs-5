import {
	Adjustments,
	CodeBranch,
	Github,
	InformationCircleSolid,
	SquareTwoStack,
} from "@cms/icons";
import {
	Avatar,
	Badge,
	Button,
	Container,
	Divider,
	Heading,
	IconButton,
	StatusBadge,
	Text,
} from "@cms/ui";
import type { ProjectSummary } from "../../workspace";
import { ProjectPreviewThumbnail } from "./project-preview-thumbnail";

interface ProjectCardProps {
	project: ProjectSummary;
}

export function ProjectCard({ project }: ProjectCardProps) {
	const { latestActivity, endpoints } = project;

	return (
		<Container className="p-0 overflow-hidden border border-ui-border-base rounded-xl shadow-xs hover:border-ui-border-interactive transition-colors">
			{/* Main Content Area */}
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

						{/* Live Endpoints using StatusBadge */}
						<div className="mt-3 space-y-2">
							{endpoints.map((ep) => (
								<div key={ep.label} className="flex items-center gap-2.5">
									<Text size="small" className="text-ui-fg-muted min-w-[70px]">
										{ep.label}
									</Text>
									<StatusBadge color="green">Live</StatusBadge>
									<Text size="small" family="mono" className="text-ui-fg-muted">
										{ep.url}
									</Text>
								</div>
							))}
						</div>
					</div>

					{/* Divider before activity */}
					<Divider />

					{/* Latest Activity subsection */}
					<div className="space-y-2">
						<div className="flex items-center gap-1.5">
							<Text size="small" weight="plus" className="text-ui-fg-subtle">
								Latest activity
							</Text>
							<InformationCircleSolid className="h-3.5 w-3.5 text-ui-fg-muted" />
						</div>

						<div className="flex flex-wrap items-center gap-2">
							<div className="inline-flex items-center gap-1.5 text-xs text-ui-fg-subtle">
								<span className="h-1.5 w-1.5 rounded-full bg-emerald-500" />
								<span>{latestActivity.statusText}</span>
							</div>

							<span className="text-ui-fg-muted">&bull;</span>

							<Badge
								size="2xsmall"
								color="grey"
								rounded="base"
								className="gap-x-1 font-mono"
							>
								<CodeBranch className="h-3 w-3 text-ui-fg-muted" />
								<span>{latestActivity.commitSha}</span>
							</Badge>
						</div>

						<div className="flex items-center gap-2 text-xs text-ui-fg-muted">
							<Avatar
								fallback={latestActivity.authorInitials}
								size="2xsmall"
								variant="rounded"
							/>
							<Text size="xsmall" weight="plus" className="text-ui-fg-base">
								{latestActivity.authorName}
							</Text>
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

			{/* Divider between body and actions */}
			<Divider />

			{/* Card Bottom Actions Bar using Button and IconButton */}
			<div className="bg-ui-bg-subtle px-6 py-2 flex items-center justify-between">
				<Button variant="secondary" size="small">
					Project details
				</Button>

				<div className="flex items-center gap-1">
					<IconButton
						variant="transparent"
						size="small"
						aria-label="View layout"
					>
						<SquareTwoStack className="h-4 w-4" />
					</IconButton>
					<IconButton variant="transparent" size="small" aria-label="Settings">
						<Adjustments className="h-4 w-4" />
					</IconButton>
					<IconButton
						variant="transparent"
						size="small"
						aria-label="Git repository"
					>
						<Github className="h-4 w-4" />
					</IconButton>
				</div>
			</div>
		</Container>
	);
}
