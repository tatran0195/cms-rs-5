import {
	Adjustments,
	BuildingStorefront,
	Check,
	ChevronDown,
	ChevronUpDown,
	CirclePlus,
	Folder,
	Key,
	SquareTwoStack,
	Users,
} from "@cms/icons";
import { Avatar, Divider, DropdownMenu, Input, Kbd, Text } from "@cms/ui";
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
							className="w-full flex items-center justify-between px-2 py-1.5 rounded-md border border-ui-border-base hover:bg-ui-bg-subtle transition-colors text-left cursor-pointer outline-none focus-visible:shadow-borders-interactive-with-active"
						>
							<div className="flex items-center gap-2 truncate">
								<Avatar
									size="xsmall"
									variant="squared"
									fallback={activeWorkspace?.name.charAt(0) || "W"}
								/>
								<Text size="small" weight="plus" className="truncate">
									{activeWorkspace?.name || "Select Workspace"}
								</Text>
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

				{/* Search Input using @cms/ui Input */}
				<div className="mt-3 relative">
					<Input
						type="search"
						size="small"
						placeholder="Search..."
						value={searchQuery}
						onChange={(e) => onSearchChange(e.target.value)}
						className="pr-12"
					/>
					<div className="absolute right-2 top-1.5 pointer-events-none">
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
					<Text size="small" weight="plus">
						Projects
					</Text>
				</Link>

				<button
					type="button"
					className="w-full flex items-center gap-2.5 px-3 py-1.5 rounded-md text-ui-fg-subtle hover:text-ui-fg-base hover:bg-ui-bg-subtle transition-colors text-left cursor-pointer"
				>
					<BuildingStorefront className="h-4 w-4" />
					<Text size="small">Usage</Text>
				</button>

				<button
					type="button"
					className="w-full flex items-center gap-2.5 px-3 py-1.5 rounded-md text-ui-fg-subtle hover:text-ui-fg-base hover:bg-ui-bg-subtle transition-colors text-left cursor-pointer"
				>
					<Users className="h-4 w-4" />
					<Text size="small">Team</Text>
				</button>

				<Divider className="my-2" />

				<button
					type="button"
					className="w-full flex items-center justify-between px-3 py-1.5 rounded-md text-ui-fg-subtle hover:text-ui-fg-base hover:bg-ui-bg-subtle transition-colors text-left cursor-pointer"
				>
					<div className="flex items-center gap-2.5">
						<Adjustments className="h-4 w-4" />
						<Text size="small">Settings</Text>
					</div>
					<ChevronDown className="h-3 w-3 text-ui-fg-muted" />
				</button>

				<button
					type="button"
					className="w-full flex items-center justify-between px-3 py-1.5 rounded-md text-ui-fg-subtle hover:text-ui-fg-base hover:bg-ui-bg-subtle transition-colors text-left cursor-pointer"
				>
					<div className="flex items-center gap-2.5">
						<Key className="h-4 w-4" />
						<Text size="small">Access keys</Text>
					</div>
					<ChevronDown className="h-3 w-3 text-ui-fg-muted" />
				</button>

				<button
					type="button"
					className="w-full flex items-center gap-2.5 px-3 py-1.5 rounded-md text-ui-fg-subtle hover:text-ui-fg-base hover:bg-ui-bg-subtle transition-colors text-left cursor-pointer"
				>
					<Users className="h-4 w-4" />
					<Text size="small">Account</Text>
				</button>

				<button
					type="button"
					className="w-full flex items-center gap-2.5 px-3 py-1.5 rounded-md text-ui-fg-subtle hover:text-ui-fg-base hover:bg-ui-bg-subtle transition-colors text-left cursor-pointer"
				>
					<Folder className="h-4 w-4" />
					<Text size="small">Connections</Text>
				</button>
			</nav>

			{/* Sidebar Footer using Divider and Text */}
			<Divider />
			<div className="p-3 text-[11px] text-ui-fg-muted flex items-center justify-between">
				<Text size="xsmall" className="text-ui-fg-muted">
					Medusa Cloud v2
				</Text>
				<Text size="xsmall" family="mono" className="text-ui-fg-muted">
					Ready
				</Text>
			</div>
		</aside>
	);
}
