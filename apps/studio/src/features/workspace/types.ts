export interface ProjectActivity {
	statusText: string;
	branch: string;
	commitSha: string;
	authorName: string;
	authorInitials: string;
	timestamp: string;
	commitMessage: string;
}

export interface ProjectEndpoint {
	label: string;
	status: "live" | "building" | "failed";
	url: string;
}

export interface ProjectSummary {
	id: string;
	workspaceId: string;
	name: string;
	slug: string;
	endpoints: ProjectEndpoint[];
	latestActivity: ProjectActivity;
	previewType: "dtc" | "b2b" | "docs";
	updatedAt: string;
}

export interface Workspace {
	id: string;
	name: string;
	slug: string;
	role: "owner" | "admin" | "member";
	createdAt: string;
}

export interface WorkspaceState {
	workspaces: Workspace[];
	projects: Record<string, ProjectSummary[]>;
	lastVisitedWorkspaceId: string | null;
	getActiveWorkspace: () => Workspace | null;
	getActiveProjects: () => ProjectSummary[];
	createOrganization: (name: string) => Workspace;
	switchOrganization: (workspaceId: string) => void;
	resetToDefaults: () => void;
}
