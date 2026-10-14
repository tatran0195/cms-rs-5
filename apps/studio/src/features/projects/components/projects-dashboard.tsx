import { CirclePlus, MagnifyingGlass } from "@cms/icons";
import { Button, Heading, Text } from "@cms/ui";
import { useNavigate } from "@tanstack/react-router";
import { useMemo, useState } from "react";
import { useWorkspaceStore } from "../../workspace";
import { ProjectCard } from "./project-card";
import { Sidebar } from "./sidebar";
import { TopBar } from "./topbar";

export function ProjectsDashboard() {
	const navigate = useNavigate();
	const { getActiveWorkspace, getActiveProjects } = useWorkspaceStore();
	const activeWorkspace = getActiveWorkspace();
	const projects = getActiveProjects();

	const [projectSearch, setProjectSearch] = useState("");
	const [globalSearch, setGlobalSearch] = useState("");

	const filteredProjects = useMemo(() => {
		if (!projectSearch.trim()) return projects;
		const q = projectSearch.toLowerCase();
		return projects.filter(
			(p) =>
				p.name.toLowerCase().includes(q) ||
				p.slug.toLowerCase().includes(q) ||
				p.endpoints.some((ep) => ep.url.toLowerCase().includes(q)),
		);
	}, [projects, projectSearch]);

	return (
		<div className="flex h-screen bg-ui-bg-subtle overflow-hidden">
			{/* Left Sidebar */}
			<Sidebar
				searchQuery={globalSearch}
				onSearchChange={setGlobalSearch}
			/>

			{/* Main Workspace Column */}
			<div className="flex-1 flex flex-col h-screen overflow-hidden">
				<TopBar />

				<main className="flex-1 overflow-y-auto p-8">
					<div className="max-w-5xl mx-auto space-y-6">
						{/* Header Row */}
						<div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
							<Heading level="h1" className="text-xl font-semibold text-ui-fg-base">
								All projects
							</Heading>

							<div className="flex items-center gap-3">
								<div className="relative">
									<MagnifyingGlass className="absolute left-2.5 top-2.5 h-3.5 w-3.5 text-ui-fg-muted pointer-events-none" />
									<input
										type="text"
										value={projectSearch}
										onChange={(e) => setProjectSearch(e.target.value)}
										placeholder="Search projects"
										className="bg-ui-bg-base text-xs pl-8 pr-3 py-1.5 rounded-md border border-ui-border-base focus:outline-none focus:border-ui-border-interactive placeholder:text-ui-fg-muted text-ui-fg-base w-48 shadow-2xs"
									/>
								</div>

								<Button
									variant="primary"
									size="small"
									className="text-xs cursor-pointer"
								>
									Create Project
								</Button>
							</div>
						</div>

						{/* Projects List */}
						{filteredProjects.length > 0 ? (
							<div className="space-y-4">
								{filteredProjects.map((p) => (
									<ProjectCard key={p.id} project={p} />
								))}
							</div>
						) : (
							<div className="bg-ui-bg-base border border-ui-border-base rounded-xl p-12 text-center flex flex-col items-center justify-center shadow-xs">
								<Heading level="h2" className="text-sm font-semibold text-ui-fg-base">
									No projects found
								</Heading>
								<Text size="small" className="text-ui-fg-muted mt-1 max-w-sm">
									{activeWorkspace?.name} doesn't have any projects yet. Create your first project to get started.
								</Text>
								<Button
									variant="secondary"
									size="small"
									className="mt-4 cursor-pointer"
								>
									Create Project
								</Button>
							</div>
						)}
					</div>
				</main>
			</div>
		</div>
	);
}
