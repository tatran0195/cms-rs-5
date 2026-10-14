import { Sparkles } from "@cms/icons";
import { Avatar, Badge, Text } from "@cms/ui";
import { useAuth } from "../../auth";
import { useWorkspaceStore } from "../../workspace";

export function TopBar() {
	const { user } = useAuth();
	const activeWorkspace = useWorkspaceStore((state) =>
		state.getActiveWorkspace(),
	);
	const initial = user?.fullName?.charAt(0) || user?.email?.charAt(0) || "A";

	return (
		<header className="h-12 border-b border-ui-border-base bg-ui-bg-base px-6 flex items-center justify-between gap-3 shrink-0">
			{/* Breadcrumbs matching Medusa Dashboard Shell */}
			<div className="flex items-center gap-2 select-none">
				<Text size="small" className="text-ui-fg-muted">
					{activeWorkspace?.name || "Acme Inc"}
				</Text>
				<span className="text-ui-fg-muted text-xs">/</span>
				<Text size="small" weight="plus" className="text-ui-fg-base">
					Projects
				</Text>
			</div>

			{/* Right actions: Agent Badge & Avatar */}
			<div className="flex items-center gap-3">
				<Badge
					size="small"
					color="grey"
					rounded="full"
					className="gap-x-1.5 cursor-pointer hover:bg-ui-bg-subtle-hover transition-colors"
				>
					<Sparkles className="h-3.5 w-3.5 text-amber-500" />
					<span>Agent</span>
				</Badge>

				<Avatar size="small" variant="rounded" fallback={initial} />
			</div>
		</header>
	);
}
