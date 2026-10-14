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
