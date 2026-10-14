import { createFileRoute } from "@tanstack/react-router";
import { SetupOrganizationPage } from "../features/organization";

export const Route = createFileRoute("/setup-organization")({
	component: SetupOrganizationPage,
});
