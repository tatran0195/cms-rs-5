import {
	Adjustments,
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
			{/* Organization Switcher Dropdown */}
			<div className="p-3 border-b border-ui-border-base">
				<DropdownMenu>
					<DropdownMenu.Trigger asChild>
						<button
							type="button"
							className="w-full flex items-center justify-between px-2.5 py-1.5 rounded-lg border border-ui-border-base hover:bg-ui-bg-subtle transition-colors text-left cursor-pointer"
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
					<MagnifyingGlass className="absolute left-2.5 top-2.5 h-3.5 w-3.5 text-ui-fg-muted pointer-events-none" />
					<input
						type="text"
						value={searchQuery}
						onChange={(e) => onSearchChange(e.target.value)}
						placeholder="Search..."
						className="w-full bg-ui-bg-subtle text-xs pl-8 pr-12 py-1.5 rounded-md border border-ui-border-base focus:outline-none focus:border-ui-border-interactive placeholder:text-ui-fg-muted text-ui-fg-base"
					/>
					<div className="absolute right-2 top-2 pointer-events-none">
						<Kbd className="text-[9px] px-1 py-0.5">Ctrl K</Kbd>
					</div>
				</div>
			</div>

			{/* Navigation Links (Note: NO Payments/Billing per ADR 0001 constraint) */}
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
					className="w-full flex items-center gap-2.5 px-3 py-1.5 rounded-md text-xs font-medium text-ui-fg-subtle hover:text-ui-fg-base hover:bg-ui-bg-subtle transition-colors text-left cursor-pointer"
				>
					<BuildingStorefront className="h-4 w-4" />
					<span>Usage</span>
				</button>

				<button
					type="button"
					className="w-full flex items-center gap-2.5 px-3 py-1.5 rounded-md text-xs font-medium text-ui-fg-subtle hover:text-ui-fg-base hover:bg-ui-bg-subtle transition-colors text-left cursor-pointer"
				>
					<Users className="h-4 w-4" />
					<span>Team</span>
				</button>

				<div className="pt-2 pb-1">
					<div className="h-px bg-ui-border-base mx-2" />
				</div>

				<button
					type="button"
					className="w-full flex items-center justify-between px-3 py-1.5 rounded-md text-xs font-medium text-ui-fg-subtle hover:text-ui-fg-base hover:bg-ui-bg-subtle transition-colors text-left cursor-pointer"
				>
					<div className="flex items-center gap-2.5">
						<Adjustments className="h-4 w-4" />
						<span>Settings</span>
					</div>
					<ChevronDown className="h-3 w-3 text-ui-fg-muted" />
				</button>

				<button
					type="button"
					className="w-full flex items-center justify-between px-3 py-1.5 rounded-md text-xs font-medium text-ui-fg-subtle hover:text-ui-fg-base hover:bg-ui-bg-subtle transition-colors text-left cursor-pointer"
				>
					<div className="flex items-center gap-2.5">
						<Key className="h-4 w-4" />
						<span>Access keys</span>
					</div>
					<ChevronDown className="h-3 w-3 text-ui-fg-muted" />
				</button>

				<button
					type="button"
					className="w-full flex items-center gap-2.5 px-3 py-1.5 rounded-md text-xs font-medium text-ui-fg-subtle hover:text-ui-fg-base hover:bg-ui-bg-subtle transition-colors text-left cursor-pointer"
				>
					<Users className="h-4 w-4" />
					<span>Account</span>
				</button>

				<button
					type="button"
					className="w-full flex items-center gap-2.5 px-3 py-1.5 rounded-md text-xs font-medium text-ui-fg-subtle hover:text-ui-fg-base hover:bg-ui-bg-subtle transition-colors text-left cursor-pointer"
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
