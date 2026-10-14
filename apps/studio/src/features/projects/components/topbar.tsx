import { Sparkles } from "@cms/icons";
import { useAuth } from "../../auth";

export function TopBar() {
	const { user } = useAuth();
	const initial = user?.fullName?.charAt(0) || user?.email?.charAt(0) || "A";

	return (
		<header className="h-12 border-b border-ui-border-base bg-ui-bg-base px-6 flex items-center justify-end gap-3 shrink-0">
			{/* Agent Action Button */}
			<button
				type="button"
				className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium bg-ui-bg-subtle border border-ui-border-base hover:bg-ui-bg-component transition-colors text-ui-fg-subtle cursor-pointer"
			>
				<Sparkles className="h-3.5 w-3.5 text-amber-500" />
				<span>Agent</span>
			</button>

			{/* User Avatar */}
			<div className="h-7 w-7 rounded-full bg-ui-bg-subtle border border-ui-border-base flex items-center justify-center text-xs font-semibold text-ui-fg-base select-none">
				{initial.toUpperCase()}
			</div>
		</header>
	);
}
