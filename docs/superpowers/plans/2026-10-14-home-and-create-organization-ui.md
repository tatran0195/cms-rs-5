# Home Page, Create Organization Page, and Workspace Redirect Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the Home / Projects overview page and the Create Organization page in `@cms/studio` matching the Medusa Cloud reference mockups, with automatic post-login redirection to the projects page of the last-visited workspace.

**Architecture:** A durable Zustand client store (`workspace-store`) manages active and last-visited workspaces with `localStorage` persistence. The root route (`/`) serves the Home/Projects dashboard when an active workspace is present, or redirects to `/setup-organization` if none exists. React Hook Form + Zod validate organization creation, and `@cms/ui` + `@cms/icons` provide pixel-level styling matching the reference mockups.

**Tech Stack:** React 19, `@tanstack/react-router`, Zustand, React Hook Form, Zod, `@cms/ui`, `@cms/icons`, Biome, Bun.

## Global Constraints

- Prohibit any plan, billing, or pricing features (ADR 0001 constraint).
- All user-editable form payloads must use React Hook Form + Zod (`zodResolver`).
- Use `@cms/ui` components and `@cms/icons` rather than raw HTML or ad-hoc Tailwind buttons.
- Format and lint all modified code using Biome (`biome check`).
- Regenerate router definitions via `bun --filter @cms/studio generate-routes` when adding new routes.

---

### Task 1: Workspace Types and Persistent Zustand Store

**Files:**
- Create: `apps/studio/src/features/workspace/types.ts`
- Create: `apps/studio/src/features/workspace/store/workspace-store.ts`
- Create: `apps/studio/src/features/workspace/index.ts`
- Test: `apps/studio/src/features/workspace/__tests__/workspace-store.test.ts`

**Interfaces:**
- Consumes: None (domain primitives)
- Produces: `useWorkspaceStore`, `Workspace`, `ProjectSummary`, `WorkspaceState`

- [ ] **Step 1: Write test for workspace store**

Create `apps/studio/src/features/workspace/__tests__/workspace-store.test.ts`:
```ts
import { beforeEach, describe, expect, it } from "bun:test";
import { useWorkspaceStore } from "../store/workspace-store";

describe("WorkspaceStore", () => {
	beforeEach(() => {
		localStorage.clear();
		useWorkspaceStore.getState().resetToDefaults();
	});

	it("initializes with default seeded workspaces including Acme Inc", () => {
		const state = useWorkspaceStore.getState();
		expect(state.workspaces.length).toBeGreaterThan(0);
		const acme = state.workspaces.find((w) => w.slug === "acme-inc");
		expect(acme).toBeDefined();
		expect(acme?.name).toBe("Acme Inc");
	});

	it("identifies active workspace from last visited or defaults to first", () => {
		const state = useWorkspaceStore.getState();
		const active = state.getActiveWorkspace();
		expect(active).toBeDefined();
		expect(active?.slug).toBe("acme-inc");
	});

	it("creates a new organization and marks it as active and last-visited", () => {
		const store = useWorkspaceStore.getState();
		const newWs = store.createOrganization("Rivera Tech");
		expect(newWs.name).toBe("Rivera Tech");
		expect(newWs.slug).toBe("rivera-tech");

		const active = useWorkspaceStore.getState().getActiveWorkspace();
		expect(active?.id).toBe(newWs.id);
		expect(useWorkspaceStore.getState().lastVisitedWorkspaceId).toBe(newWs.id);
	});

	it("switches active organization and updates last-visited", () => {
		const store = useWorkspaceStore.getState();
		store.switchOrganization("ws-personal");

		const active = useWorkspaceStore.getState().getActiveWorkspace();
		expect(active?.id).toBe("ws-personal");
		expect(useWorkspaceStore.getState().lastVisitedWorkspaceId).toBe("ws-personal");
	});
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `bun test apps/studio/src/features/workspace/__tests__/workspace-store.test.ts`
Expected: FAIL (Cannot find module '../store/workspace-store')

- [ ] **Step 3: Implement workspace domain types**

Create `apps/studio/src/features/workspace/types.ts`:
```ts
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
```

- [ ] **Step 4: Implement persistent workspace store**

Create `apps/studio/src/features/workspace/store/workspace-store.ts`:
```ts
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
```

- [ ] **Step 5: Create feature index export**

Create `apps/studio/src/features/workspace/index.ts`:
```ts
export * from "./types";
export * from "./store/workspace-store";
```

- [ ] **Step 6: Run tests to verify they pass**

Run: `bun test apps/studio/src/features/workspace/__tests__/workspace-store.test.ts`
Expected: PASS (4 tests passed)

- [ ] **Step 7: Commit**

```bash
git add apps/studio/src/features/workspace/
git commit -m "feat(studio): add workspace types and persistent workspace store"
```

---

### Task 2: Create Organization Feature (Setup Wizard Page)

**Files:**
- Create: `apps/studio/src/features/organization/schemas/organization-schema.ts`
- Create: `apps/studio/src/features/organization/components/setup-organization-page.tsx`
- Create: `apps/studio/src/features/organization/index.ts`
- Test: `apps/studio/src/features/organization/__tests__/organization-schema.test.ts`

**Interfaces:**
- Consumes: `useWorkspaceStore` from `features/workspace`, `useAuth` from `features/auth`
- Produces: `SetupOrganizationPage`, `createOrganizationSchema`

- [ ] **Step 1: Write schema validation test**

Create `apps/studio/src/features/organization/__tests__/organization-schema.test.ts`:
```ts
import { describe, expect, it } from "bun:test";
import { createOrganizationSchema } from "../schemas/organization-schema";

describe("createOrganizationSchema", () => {
	it("validates non-empty organization names", () => {
		const valid = createOrganizationSchema.safeParse({ name: "Acme Corp" });
		expect(valid.success).toBe(true);
		if (valid.success) {
			expect(valid.data.name).toBe("Acme Corp");
		}
	});

	it("rejects empty or single character names", () => {
		const empty = createOrganizationSchema.safeParse({ name: "" });
		expect(empty.success).toBe(false);

		const short = createOrganizationSchema.safeParse({ name: "A" });
		expect(short.success).toBe(false);
	});
});
```

- [ ] **Step 2: Run schema test to verify it fails**

Run: `bun test apps/studio/src/features/organization/__tests__/organization-schema.test.ts`
Expected: FAIL (Cannot find module '../schemas/organization-schema')

- [ ] **Step 3: Implement Zod organization schema**

Create `apps/studio/src/features/organization/schemas/organization-schema.ts`:
```ts
import { z } from "zod";

export const createOrganizationSchema = z.object({
	name: z
		.string()
		.trim()
		.min(2, "Organization name must be at least 2 characters")
		.max(64, "Organization name cannot exceed 64 characters"),
});

export type CreateOrganizationInput = z.infer<typeof createOrganizationSchema>;
```

- [ ] **Step 4: Implement SetupOrganizationPage component**

Create `apps/studio/src/features/organization/components/setup-organization-page.tsx`:
```tsx
import { ArrowRightOnRectangle, BuildingStorefront } from "@cms/icons";
import { Button, Heading, Input, Text, toast } from "@cms/ui";
import { zodResolver } from "@hookform/resolvers/zod";
import { useNavigate } from "@tanstack/react-router";
import { Controller, useForm } from "react-hook-form";
import { useAuth } from "../../auth";
import { useWorkspaceStore } from "../../workspace";
import {
	type CreateOrganizationInput,
	createOrganizationSchema,
} from "../schemas/organization-schema";

export function SetupOrganizationPage() {
	const navigate = useNavigate();
	const { user, logout } = useAuth();
	const createOrganization = useWorkspaceStore(
		(state) => state.createOrganization,
	);

	const defaultName = user?.fullName
		? `${user.fullName}'s projects`
		: "Alex Rivera's projects";

	const {
		control,
		handleSubmit,
		formState: { errors, isSubmitting },
	} = useForm<CreateOrganizationInput>({
		resolver: zodResolver(createOrganizationSchema),
		defaultValues: {
			name: defaultName,
		},
	});

	const onSubmit = (data: CreateOrganizationInput) => {
		const newWs = createOrganization(data.name);
		toast.success(`Organization "${newWs.name}" created!`);
		navigate({ to: "/" });
	};

	const handleSignOut = async () => {
		await logout();
		navigate({ to: "/login" });
	};

	const userEmail = user?.email || "alex@acme.example";

	return (
		<div className="min-h-screen bg-ui-bg-subtle flex flex-col justify-between selection:bg-ui-bg-highlight">
			{/* Top Bar: Sign Out */}
			<header className="w-full flex justify-end px-8 py-4">
				<button
					type="button"
					onClick={handleSignOut}
					className="flex items-center gap-1.5 text-xs text-ui-fg-muted hover:text-ui-fg-base transition-colors font-mono"
				>
					<ArrowRightOnRectangle className="h-3.5 w-3.5" />
					<span>Sign out ({userEmail})</span>
				</button>
			</header>

			{/* Main Center Content */}
			<main className="flex-1 flex flex-col items-center justify-center px-4 -mt-10">
				<div className="w-full max-w-2xl flex flex-col items-center">
					{/* Icon Header */}
					<div className="h-10 w-10 rounded-full border border-ui-border-base bg-ui-bg-base flex items-center justify-center text-ui-fg-subtle mb-4 shadow-sm">
						<BuildingStorefront className="h-5 w-5" />
					</div>

					{/* Title & Subtitle */}
					<Heading level="h1" className="text-lg font-semibold text-ui-fg-base text-center">
						Setup your organization
					</Heading>
					<Text size="small" className="text-ui-fg-subtle text-center mt-1">
						Specify the name of your organization
					</Text>

					{/* Card Container */}
					<div className="w-full mt-8 pt-8 pb-6 border-t border-b border-ui-border-base">
						<form onSubmit={handleSubmit(onSubmit)} className="space-y-6">
							<div className="grid grid-cols-1 md:grid-cols-2 gap-8 items-start">
								<div>
									<Text className="text-sm font-medium text-ui-fg-base">
										Setup your organization
									</Text>
									<Text size="small" className="text-ui-fg-muted mt-0.5">
										Specify the name of your organization
									</Text>
								</div>

								<div className="space-y-2">
									<label
										htmlFor="org-name"
										className="block text-xs font-medium text-ui-fg-subtle"
									>
										Organization name
									</label>
									<Controller
										name="name"
										control={control}
										render={({ field }) => (
											<Input
												{...field}
												id="org-name"
												aria-invalid={Boolean(errors.name)}
												placeholder="e.g. Acme Corp"
												className="bg-ui-bg-base"
											/>
										)}
									/>
									{errors.name && (
										<p className="text-xs text-ui-fg-error">
											{errors.name.message}
										</p>
									)}
								</div>
							</div>

							<div className="pt-4 flex justify-end">
								<Button
									type="submit"
									variant="primary"
									isLoading={isSubmitting}
									className="w-full md:w-auto px-8"
								>
									Create
								</Button>
							</div>
						</form>
					</div>
				</div>
			</main>

			{/* Footer Step Indicator */}
			<footer className="w-full text-center py-6 text-xs text-ui-fg-muted font-mono">
				Step 01 / 02
			</footer>
		</div>
	);
}
```

- [ ] **Step 5: Create organization feature index**

Create `apps/studio/src/features/organization/index.ts`:
```ts
export * from "./schemas/organization-schema";
export * from "./components/setup-organization-page.tsx";
```

- [ ] **Step 6: Run tests to verify they pass**

Run: `bun test apps/studio/src/features/organization/__tests__/organization-schema.test.ts`
Expected: PASS

- [ ] **Step 7: Commit**

```bash
git add apps/studio/src/features/organization/
git commit -m "feat(studio): add create organization page with RHF and zod validation"
```

---

### Task 3: Projects Feature Components (Sidebar, TopNav, ProjectCard, and Dashboard)

**Files:**
- Create: `apps/studio/src/features/projects/components/project-preview-thumbnail.tsx`
- Create: `apps/studio/src/features/projects/components/project-card.tsx`
- Create: `apps/studio/src/features/projects/components/sidebar.tsx`
- Create: `apps/studio/src/features/projects/components/topbar.tsx`
- Create: `apps/studio/src/features/projects/components/projects-dashboard.tsx`
- Create: `apps/studio/src/features/projects/index.ts`

**Interfaces:**
- Consumes: `useWorkspaceStore`, `useAuth`
- Produces: `ProjectsDashboard`, `Sidebar`, `ProjectCard`

- [ ] **Step 1: Implement ProjectPreviewThumbnail component**

Create `apps/studio/src/features/projects/components/project-preview-thumbnail.tsx`:
```tsx
interface ProjectPreviewThumbnailProps {
	type: "dtc" | "b2b" | "docs";
}

export function ProjectPreviewThumbnail({ type }: ProjectPreviewThumbnailProps) {
	if (type === "dtc") {
		return (
			<div className="w-full h-full min-h-[140px] bg-[#FAF8F5] p-3 rounded-lg flex flex-col justify-between border border-ui-border-base select-none">
				<div className="flex items-center justify-between text-[10px] text-stone-500 border-b border-stone-200/80 pb-1.5">
					<span className="font-serif tracking-wider font-semibold text-stone-700">
						Autumn Essentials
					</span>
					<div className="flex gap-1.5">
						<span className="h-1.5 w-1.5 rounded-full bg-stone-300" />
						<span className="h-1.5 w-1.5 rounded-full bg-stone-300" />
					</div>
				</div>
				<div className="grid grid-cols-3 gap-2 my-auto py-2">
					<div className="aspect-square bg-[#E8E2D9] rounded flex items-center justify-center">
						<div className="h-5 w-5 rounded bg-stone-400/40" />
					</div>
					<div className="aspect-square bg-[#D8E0DA] rounded flex items-center justify-center">
						<div className="h-5 w-5 rounded-full bg-emerald-700/20" />
					</div>
					<div className="aspect-square bg-[#E4DDD3] rounded flex items-center justify-center">
						<div className="h-5 w-5 bg-amber-700/20 rotate-45" />
					</div>
				</div>
				<div className="flex justify-between items-center text-[8px] text-stone-400">
					<span>Medusa Storefront</span>
					<span>v2.4.0</span>
				</div>
			</div>
		);
	}

	return (
		<div className="w-full h-full min-h-[140px] bg-[#F7F9FB] p-3 rounded-lg flex flex-col justify-between border border-ui-border-base select-none">
			<div className="flex items-center justify-between text-[10px] text-slate-600 border-b border-slate-200 pb-1.5">
				<span className="font-semibold text-slate-800">Wholesale orders</span>
				<div className="flex gap-2 text-[9px] text-slate-400">
					<span>Active</span>
					<span>Pending</span>
				</div>
			</div>
			<div className="grid grid-cols-3 gap-2 my-auto py-2">
				<div className="bg-white border border-slate-200/80 rounded p-1.5">
					<span className="text-[8px] text-slate-400">Orders</span>
					<div className="text-[11px] font-bold text-slate-700">12</div>
				</div>
				<div className="bg-white border border-slate-200/80 rounded p-1.5">
					<span className="text-[8px] text-slate-400">Quotes</span>
					<div className="text-[11px] font-bold text-slate-700">3</div>
				</div>
				<div className="bg-white border border-slate-200/80 rounded p-1.5">
					<span className="text-[8px] text-slate-400">Volume</span>
					<div className="text-[11px] font-bold text-emerald-600">$41,200</div>
				</div>
			</div>
			<div className="flex justify-between items-center text-[8px] text-slate-400">
				<span>B2B Portal</span>
				<span>Connected</span>
			</div>
		</div>
	);
}
```

- [ ] **Step 2: Implement ProjectCard component matching reference screenshot**

Create `apps/studio/src/features/projects/components/project-card.tsx`:
```tsx
import {
	Adjustments,
	BuildingStorefront,
	CommandLine,
	Eye,
	GitBranch,
	InformationCircleSolid,
} from "@cms/icons";
import { Button, Heading, StatusBadge, Text } from "@cms/ui";
import type { ProjectSummary } from "../../workspace";
import { ProjectPreviewThumbnail } from "./project-preview-thumbnail";

interface ProjectCardProps {
	project: ProjectSummary;
}

export function ProjectCard({ project }: ProjectCardProps) {
	const { latestActivity, endpoints } = project;

	return (
		<div className="w-full bg-ui-bg-base border border-ui-border-base rounded-xl overflow-hidden shadow-xs hover:border-ui-border-interactive transition-colors flex flex-col">
			{/* Main Row */}
			<div className="p-6 grid grid-cols-1 md:grid-cols-12 gap-6 items-center">
				{/* Left Preview Canvas */}
				<div className="md:col-span-5 w-full">
					<ProjectPreviewThumbnail type={project.previewType} />
				</div>

				{/* Right Project Details */}
				<div className="md:col-span-7 flex flex-col justify-between space-y-4">
					<div>
						<Heading level="h2" className="text-base font-semibold text-ui-fg-base">
							{project.name}
						</Heading>

						{/* Live Endpoints */}
						<div className="mt-3 space-y-1.5">
							{endpoints.map((ep) => (
								<div key={ep.label} className="flex items-center gap-2 text-xs">
									<span className="text-ui-fg-muted min-w-[70px]">{ep.label}</span>
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
					<div className="pt-2 border-t border-ui-border-base">
						<div className="flex items-center gap-1.5 text-xs text-ui-fg-subtle mb-1.5">
							<span className="font-medium">Latest activity</span>
							<InformationCircleSolid className="h-3.5 w-3.5 text-ui-fg-muted" />
						</div>

						<div className="flex flex-wrap items-center gap-2 text-xs">
							<div className="inline-flex items-center gap-1.5 text-ui-fg-subtle">
								<span className="h-1.5 w-1.5 rounded-full bg-emerald-500" />
								<span>{latestActivity.statusText}</span>
							</div>

							<span className="text-ui-fg-muted">&bull;</span>

							<div className="inline-flex items-center gap-1 font-mono text-[11px] text-ui-fg-subtle bg-ui-bg-subtle px-1.5 py-0.5 rounded border border-ui-border-base">
								<GitBranch className="h-3 w-3 text-ui-fg-muted" />
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
						className="p-1.5 text-ui-fg-muted hover:text-ui-fg-base rounded transition-colors"
					>
						<Eye className="h-4 w-4" />
					</button>
					<button
						type="button"
						aria-label="Settings"
						className="p-1.5 text-ui-fg-muted hover:text-ui-fg-base rounded transition-colors"
					>
						<Adjustments className="h-4 w-4" />
					</button>
					<button
						type="button"
						aria-label="Git Repository"
						className="p-1.5 text-ui-fg-muted hover:text-ui-fg-base rounded transition-colors"
					>
						<CommandLine className="h-4 w-4" />
					</button>
				</div>
			</div>
		</div>
	);
}
```

- [ ] **Step 3: Implement Sidebar with Workspace Switcher**

Create `apps/studio/src/features/projects/components/sidebar.tsx`:
```tsx
import {
	Adjustments,
	ArrowRightOnRectangle,
	BuildingStorefront,
	Check,
	ChevronDown,
	ChevronUpDown,
	CirclePlus,
	Folder,
	Key,
	MagnifyingGlass,
	SquareTwoStack,
	Users,
} from "@cms/icons";
import { DropdownMenu, Kbd } from "@cms/ui";
import { Link, useNavigate } from "@tanstack/react-router";
import { useWorkspaceStore } from "../../workspace";

interface SidebarProps {
	searchQuery: string;
	onSearchChange: (q: string) => void;
}

export function Sidebar({ searchQuery, onSearchChange }: SidebarProps) {
	const navigate = useNavigate();
	const { workspaces, getActiveWorkspace, switchOrganization } =
		useWorkspaceStore();
	const activeWorkspace = getActiveWorkspace();

	return (
		<aside className="w-64 border-r border-ui-border-base bg-ui-bg-base flex flex-col h-screen shrink-0 select-none">
			{/* Organization Switcher */}
			<div className="p-3 border-b border-ui-border-base">
				<DropdownMenu>
					<DropdownMenu.Trigger asChild>
						<button
							type="button"
							className="w-full flex items-center justify-between px-2.5 py-1.5 rounded-lg border border-ui-border-base hover:bg-ui-bg-subtle transition-colors text-left"
						>
							<div className="flex items-center gap-2 truncate">
								<div className="h-6 w-6 rounded bg-ui-bg-subtle border border-ui-border-base flex items-center justify-center text-xs font-semibold text-ui-fg-base shrink-0">
									{activeWorkspace?.name.charAt(0) || "W"}
								</div>
								<span className="text-xs font-semibold text-ui-fg-base truncate">
									{activeWorkspace?.name || "Select Workspace"}
								</span>
							</div>
							<ChevronUpDown className="h-3.5 w-3.5 text-ui-fg-muted shrink-0" />
						</button>
					</DropdownMenu.Trigger>

					<DropdownMenu.Content className="w-56" align="start">
						<DropdownMenu.Label className="text-[10px] uppercase tracking-wider text-ui-fg-muted px-2 py-1">
							Organizations
						</DropdownMenu.Label>
						{workspaces.map((ws) => (
							<DropdownMenu.Item
								key={ws.id}
								onClick={() => switchOrganization(ws.id)}
								className="flex items-center justify-between cursor-pointer"
							>
								<span className="truncate">{ws.name}</span>
								{ws.id === activeWorkspace?.id && (
									<Check className="h-3.5 w-3.5 text-ui-fg-interactive shrink-0" />
								)}
							</DropdownMenu.Item>
						))}

						<DropdownMenu.Separator />

						<DropdownMenu.Item
							onClick={() => navigate({ to: "/setup-organization" })}
							className="flex items-center gap-2 text-ui-fg-interactive cursor-pointer"
						>
							<CirclePlus className="h-3.5 w-3.5" />
							<span>Create Organization</span>
						</DropdownMenu.Item>
					</DropdownMenu.Content>
				</DropdownMenu>

				{/* Quick Search Input */}
				<div className="mt-3 relative">
					<MagnifyingGlass className="absolute left-2.5 top-2.5 h-3.5 w-3.5 text-ui-fg-muted" />
					<input
						type="text"
						value={searchQuery}
						onChange={(e) => onSearchChange(e.target.value)}
						placeholder="Search..."
						className="w-full bg-ui-bg-subtle text-xs pl-8 pr-12 py-1.5 rounded-md border border-ui-border-base focus:outline-none focus:border-ui-border-interactive placeholder:text-ui-fg-muted text-ui-fg-base"
					/>
					<div className="absolute right-2 top-2">
						<Kbd className="text-[9px] px-1 py-0.5">Ctrl K</Kbd>
					</div>
				</div>
			</div>

			{/* Navigation Links */}
			<nav className="flex-1 p-2 space-y-0.5 overflow-y-auto">
				<Link
					to="/"
					className="flex items-center gap-2.5 px-3 py-1.5 rounded-md text-xs font-medium text-ui-fg-base bg-ui-bg-subtle"
				>
					<SquareTwoStack className="h-4 w-4 text-ui-fg-base" />
					<span>Projects</span>
				</Link>

				<button
					type="button"
					className="w-full flex items-center gap-2.5 px-3 py-1.5 rounded-md text-xs font-medium text-ui-fg-subtle hover:text-ui-fg-base hover:bg-ui-bg-subtle transition-colors text-left"
				>
					<BuildingStorefront className="h-4 w-4" />
					<span>Usage</span>
				</button>

				<button
					type="button"
					className="w-full flex items-center gap-2.5 px-3 py-1.5 rounded-md text-xs font-medium text-ui-fg-subtle hover:text-ui-fg-base hover:bg-ui-bg-subtle transition-colors text-left"
				>
					<Users className="h-4 w-4" />
					<span>Team</span>
				</button>

				<div className="pt-2 pb-1">
					<div className="h-px bg-ui-border-base mx-2" />
				</div>

				<button
					type="button"
					className="w-full flex items-center justify-between px-3 py-1.5 rounded-md text-xs font-medium text-ui-fg-subtle hover:text-ui-fg-base hover:bg-ui-bg-subtle transition-colors text-left"
				>
					<div className="flex items-center gap-2.5">
						<Adjustments className="h-4 w-4" />
						<span>Settings</span>
					</div>
					<ChevronDown className="h-3 w-3 text-ui-fg-muted" />
				</button>

				<button
					type="button"
					className="w-full flex items-center justify-between px-3 py-1.5 rounded-md text-xs font-medium text-ui-fg-subtle hover:text-ui-fg-base hover:bg-ui-bg-subtle transition-colors text-left"
				>
					<div className="flex items-center gap-2.5">
						<Key className="h-4 w-4" />
						<span>Access keys</span>
					</div>
					<ChevronDown className="h-3 w-3 text-ui-fg-muted" />
				</button>

				<button
					type="button"
					className="w-full flex items-center gap-2.5 px-3 py-1.5 rounded-md text-xs font-medium text-ui-fg-subtle hover:text-ui-fg-base hover:bg-ui-bg-subtle transition-colors text-left"
				>
					<Users className="h-4 w-4" />
					<span>Account</span>
				</button>

				<button
					type="button"
					className="w-full flex items-center gap-2.5 px-3 py-1.5 rounded-md text-xs font-medium text-ui-fg-subtle hover:text-ui-fg-base hover:bg-ui-bg-subtle transition-colors text-left"
				>
					<Folder className="h-4 w-4" />
					<span>Connections</span>
				</button>
			</nav>

			{/* Sidebar Footer */}
			<div className="p-3 border-t border-ui-border-base text-[11px] text-ui-fg-muted flex items-center justify-between">
				<span>Medusa Cloud v2</span>
				<span className="font-mono text-[10px]">Ready</span>
			</div>
		</aside>
	);
}
```

- [ ] **Step 4: Implement TopNav component**

Create `apps/studio/src/features/projects/components/topbar.tsx`:
```tsx
import { Sparkles } from "@cms/icons";
import { Badge } from "@cms/ui";
import { useAuth } from "../../auth";

export function TopBar() {
	const { user } = useAuth();
	const initial = user?.fullName?.charAt(0) || user?.email?.charAt(0) || "A";

	return (
		<header className="h-12 border-b border-ui-border-base bg-ui-bg-base px-6 flex items-center justify-end gap-3 shrink-0">
			{/* Agent Badge */}
			<button
				type="button"
				className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium bg-ui-bg-subtle border border-ui-border-base hover:bg-ui-bg-component transition-colors text-ui-fg-subtle"
			>
				<Sparkles className="h-3.5 w-3.5 text-amber-500" />
				<span>Agent</span>
			</button>

			{/* User Avatar */}
			<div className="h-7 w-7 rounded-full bg-ui-bg-subtle border border-ui-border-base flex items-center justify-center text-xs font-semibold text-ui-fg-base">
				{initial.toUpperCase()}
			</div>
		</header>
	);
}
```

- [ ] **Step 5: Implement ProjectsDashboard component**

Create `apps/studio/src/features/projects/components/projects-dashboard.tsx`:
```tsx
import { CirclePlus, MagnifyingGlass } from "@cms/icons";
import { Button, Heading, Input, Text } from "@cms/ui";
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
									<MagnifyingGlass className="absolute left-2.5 top-2.5 h-3.5 w-3.5 text-ui-fg-muted" />
									<input
										type="text"
										value={projectSearch}
										onChange={(e) => setProjectSearch(e.target.value)}
										placeholder="Search projects"
										className="bg-ui-bg-base text-xs pl-8 pr-3 py-1.5 rounded-md border border-ui-border-base focus:outline-none focus:border-ui-border-interactive placeholder:text-ui-fg-muted text-ui-fg-base w-48"
									/>
								</div>

								<Button
									variant="primary"
									size="small"
									onClick={() => {
										// Demonstration action
									}}
									className="text-xs"
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
							<div className="bg-ui-bg-base border border-ui-border-base rounded-xl p-12 text-center flex flex-col items-center justify-center">
								<Heading level="h2" className="text-sm font-semibold text-ui-fg-base">
									No projects found
								</Heading>
								<Text size="small" className="text-ui-fg-muted mt-1 max-w-sm">
									{activeWorkspace?.name} doesn't have any projects yet. Create your first project to get started.
								</Text>
								<Button
									variant="secondary"
									size="small"
									className="mt-4"
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
```

- [ ] **Step 6: Create projects feature index**

Create `apps/studio/src/features/projects/index.ts`:
```ts
export * from "./components/projects-dashboard";
export * from "./components/project-card";
export * from "./components/sidebar";
export * from "./components/topbar";
```

- [ ] **Step 7: Commit**

```bash
git add apps/studio/src/features/projects/
git commit -m "feat(studio): add projects dashboard, project card, and sidebar components"
```

---

### Task 4: Routes Wiring and Post-Login Redirection

**Files:**
- Create: `apps/studio/src/routes/setup-organization.tsx`
- Modify: `apps/studio/src/routes/index.tsx`
- Modify: `apps/studio/src/features/auth/components/login-page.tsx`
- Update: `apps/studio/src/routeTree.gen.ts`

**Interfaces:**
- Consumes: `SetupOrganizationPage`, `ProjectsDashboard`, `useWorkspaceStore`
- Produces: Updated TanStack router tree

- [ ] **Step 1: Create setup-organization route**

Create `apps/studio/src/routes/setup-organization.tsx`:
```tsx
import { createFileRoute } from "@tanstack/react-router";
import { SetupOrganizationPage } from "../features/organization";

export const Route = createFileRoute("/setup-organization")({
	component: SetupOrganizationPage,
});
```

- [ ] **Step 2: Update index route to check workspace and render ProjectsDashboard**

Update `apps/studio/src/routes/index.tsx`:
```tsx
import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { useEffect } from "react";
import { useAuth } from "../features/auth";
import { ProjectsDashboard } from "../features/projects";
import { useWorkspaceStore } from "../features/workspace";

export const Route = createFileRoute("/")({
	component: HomePage,
});

function HomePage() {
	const navigate = useNavigate();
	const { user } = useAuth();
	const { workspaces, getActiveWorkspace } = useWorkspaceStore();

	const activeWorkspace = getActiveWorkspace();

	useEffect(() => {
		// If no workspace exists, redirect to setup organization
		if (!workspaces.length || !activeWorkspace) {
			navigate({ to: "/setup-organization" });
		}
	}, [workspaces, activeWorkspace, navigate]);

	if (!workspaces.length || !activeWorkspace) {
		return null;
	}

	return <ProjectsDashboard />;
}
```

- [ ] **Step 3: Update LoginPage to redirect to last-visited workspace projects**

Update `apps/studio/src/features/auth/components/login-page.tsx`:
Replace lines 24-58 with:
```tsx
	const handleLoginSubmit = async (credentials: LoginCredentials) => {
		const res = await login(credentials);
		if (res?.user) {
			const displayName =
				res.user.fullName || res.user.full_name || res.user.email;
			toast.success(`Welcome back, ${displayName}!`);

			const activeWs = useWorkspaceStore.getState().getActiveWorkspace();
			const destination = activeWs ? "/" : "/setup-organization";

			setTimeout(() => {
				navigate({ to: destination });
			}, 600);
		}
	};

	const handleMfaVerify = async (
		code: string,
		method: "totp" | "recovery_code",
	) => {
		const res = await verifyMfa(code, method);
		if (res?.token && res?.user) {
			toast.success("Two-factor authentication verified successfully!");

			const activeWs = useWorkspaceStore.getState().getActiveWorkspace();
			const destination = activeWs ? "/" : "/setup-organization";

			setTimeout(() => {
				navigate({ to: destination });
			}, 600);
		}
	};

	const handleSsoClick = async (provider: AuthProviderId) => {
		const res = await ssoLogin(provider);
		if (res?.token && res?.user) {
			toast.success(`Connected via ${provider.toUpperCase()}`);

			const activeWs = useWorkspaceStore.getState().getActiveWorkspace();
			const destination = activeWs ? "/" : "/setup-organization";

			setTimeout(() => {
				navigate({ to: destination });
			}, 600);
		}
	};
```

- [ ] **Step 4: Regenerate TanStack router tree**

Run: `bun --filter @cms/studio generate-routes`
Expected: Successfully generates `routeTree.gen.ts` including `/setup-organization`

- [ ] **Step 5: Run tests across workspace to ensure no regressions**

Run: `bun test apps/studio/src/features/`
Expected: All tests pass

- [ ] **Step 6: Commit**

```bash
git add apps/studio/src/routes/ apps/studio/src/features/auth/ apps/studio/src/routeTree.gen.ts
git commit -m "feat(studio): wire setup-organization route and post-login redirect to last-visited workspace"
```

---

### Task 5: Biome Formatting, Linting, and Verification

**Files:**
- Check: All modified files in `apps/studio`

- [ ] **Step 1: Run Biome check and format**

Run: `bun --filter @cms/studio check`
Expected: Biome check passes with 0 errors

- [ ] **Step 2: Run all unit tests**

Run: `bun test apps/studio/src/features/`
Expected: PASS

- [ ] **Step 3: Commit any formatting adjustments**

```bash
git commit -am "chore(studio): format code with biome"
```
