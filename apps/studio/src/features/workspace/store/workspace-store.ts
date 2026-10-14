import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { ProjectSummary, Workspace, WorkspaceState } from "../types";

export const DEFAULT_WORKSPACES: Workspace[] = [
	{
		id: "ws-acme",
		name: "Acme Inc",
		slug: "acme-inc",
		role: "owner",
		createdAt: new Date().toISOString(),
	},
	{
		id: "ws-personal",
		name: "Personal Workspace",
		slug: "personal-workspace",
		role: "owner",
		createdAt: new Date().toISOString(),
	},
];

export const DEFAULT_PROJECTS: Record<string, ProjectSummary[]> = {
	"ws-acme": [
		{
			id: "proj-acme-dtc",
			workspaceId: "ws-acme",
			name: "Acme DTC",
			slug: "acme-dtc",
			previewType: "dtc",
			endpoints: [
				{ label: "Backend", status: "live", url: "demo.medusajs.com" },
				{ label: "Storefront", status: "live", url: "next.medusajs.com" },
			],
			latestActivity: {
				statusText: "Ready - to develop",
				branch: "develop",
				commitSha: "cf750e1",
				authorName: "Alex Rivera",
				authorInitials: "A",
				timestamp: "5m ago",
				commitMessage: "Handle partial refunds on split payments",
			},
			updatedAt: new Date().toISOString(),
		},
		{
			id: "proj-acme-b2b",
			workspaceId: "ws-acme",
			name: "Acme B2B",
			slug: "acme-b2b",
			previewType: "b2b",
			endpoints: [
				{ label: "Backend", status: "live", url: "demo.medusajs.com" },
				{ label: "Storefront", status: "live", url: "next.medusajs.com" },
			],
			latestActivity: {
				statusText: "Ready - to main",
				branch: "main",
				commitSha: "532c4e3",
				authorName: "Alex Rivera",
				authorInitials: "A",
				timestamp: "5m ago",
				commitMessage: "Reduce cold start by trimming the bundle",
			},
			updatedAt: new Date().toISOString(),
		},
	],
	"ws-personal": [],
};

function slugify(name: string): string {
	return name
		.toLowerCase()
		.trim()
		.replace(/[^a-z0-9]+/g, "-")
		.replace(/^-+|-+$/g, "");
}

export const useWorkspaceStore = create<WorkspaceState>()(
	persist(
		(set, get) => ({
			workspaces: DEFAULT_WORKSPACES,
			projects: DEFAULT_PROJECTS,
			lastVisitedWorkspaceId: "ws-acme",

			getActiveWorkspace: () => {
				const { workspaces, lastVisitedWorkspaceId } = get();
				if (!workspaces.length) return null;
				if (lastVisitedWorkspaceId) {
					const matched = workspaces.find((w) => w.id === lastVisitedWorkspaceId);
					if (matched) return matched;
				}
				return workspaces[0];
			},

			getActiveProjects: () => {
				const active = get().getActiveWorkspace();
				if (!active) return [];
				return get().projects[active.id] || [];
			},

			createOrganization: (name: string) => {
				const id = `ws-${Date.now()}`;
				const slug = slugify(name) || `workspace-${Date.now()}`;
				const newWorkspace: Workspace = {
					id,
					name,
					slug,
					role: "owner",
					createdAt: new Date().toISOString(),
				};

				set((state) => ({
					workspaces: [newWorkspace, ...state.workspaces],
					lastVisitedWorkspaceId: id,
					projects: {
						...state.projects,
						[id]: [],
					},
				}));

				return newWorkspace;
			},

			switchOrganization: (workspaceId: string) => {
				set({ lastVisitedWorkspaceId: workspaceId });
			},

			resetToDefaults: () => {
				set({
					workspaces: DEFAULT_WORKSPACES,
					projects: DEFAULT_PROJECTS,
					lastVisitedWorkspaceId: "ws-acme",
				});
			},
		}),
		{
			name: "cms_workspaces_v1",
		},
	),
);
