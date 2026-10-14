import { beforeEach, describe, expect, it } from "bun:test";
import { useWorkspaceStore } from "../store/workspace-store";

describe("WorkspaceStore", () => {
	beforeEach(() => {
		// Reset store state
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
