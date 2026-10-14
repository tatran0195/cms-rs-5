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
	const { user, isLoading } = useAuth();
	const { workspaces, getActiveWorkspace } = useWorkspaceStore();

	const activeWorkspace = getActiveWorkspace();

	useEffect(() => {
		if (isLoading) return;
		if (!user) {
			navigate({ to: "/login" });
			return;
		}
		// If no workspace exists, redirect to setup organization
		if (!workspaces.length || !activeWorkspace) {
			navigate({ to: "/setup-organization" });
		}
	}, [user, isLoading, workspaces, activeWorkspace, navigate]);

	if (isLoading) {
		return null;
	}

	if (!user || !workspaces.length || !activeWorkspace) {
		return null;
	}

	return <ProjectsDashboard />;
}
