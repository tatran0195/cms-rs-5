import { Sparkles } from "@cms/icons";
import { Avatar, Badge } from "@cms/ui";
import { useAuth } from "../../auth";

export function TopBar() {
	const { user } = useAuth();
	const initial = user?.fullName?.charAt(0) || user?.email?.charAt(0) || "A";

	return (
		<header className="h-12 border-b border-ui-border-base bg-ui-bg-base px-6 flex items-center justify-end gap-3 shrink-0">
			{/* Agent Badge using Badge from @cms/ui */}
			<Badge
				size="small"
				color="grey"
				rounded="full"
				className="gap-x-1.5 cursor-pointer hover:bg-ui-bg-subtle-hover transition-colors"
			>
				<Sparkles className="h-3.5 w-3.5 text-amber-500" />
				<span>Agent</span>
			</Badge>

			{/* User Avatar using Avatar from @cms/ui */}
			<Avatar size="small" variant="rounded" fallback={initial} />
		</header>
	);
}
