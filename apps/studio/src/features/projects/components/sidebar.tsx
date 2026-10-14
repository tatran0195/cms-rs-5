import {
	Adjustments,
	ArrowRightOnRectangle,
	ChartBar,
	Check,
	ChevronRightMini,
	ChevronUpDown,
	Cms,
	EllipsisHorizontal,
	Key,
	Plug,
	Plus,
	SquareTwoStack,
	User as UserIcon,
	Users,
} from "@cms/icons";
import { Avatar, Divider, DropdownMenu, Input, Kbd, Text } from "@cms/ui";
import { Link, useNavigate } from "@tanstack/react-router";
import { useAuth } from "../../auth";
import { useWorkspaceStore } from "../../workspace";

interface SidebarProps {
	searchQuery: string;
	onSearchChange: (q: string) => void;
}

export function Sidebar({ searchQuery, onSearchChange }: SidebarProps) {
	const navigate = useNavigate();
	const { user, logout } = useAuth();
	const { workspaces, getActiveWorkspace, switchOrganization } =
		useWorkspaceStore();
	const activeWorkspace = getActiveWorkspace();

	const displayName = user?.fullName || user?.email || "Admin User";
	const userEmail = user?.email || "admin@example.com";
	const userInitial = displayName.charAt(0).toUpperCase();

	const handleLogout = async () => {
		await logout();
		navigate({ to: "/login" });
	};

	return (
		<aside className="w-60 border-r border-ui-border-base bg-ui-bg-subtle flex flex-col h-screen shrink-0 select-none">
			{/* Top Bar Header with Logo & Organization Switcher */}
			<div className="h-12 border-b border-ui-border-base flex items-stretch shrink-0 bg-ui-bg-base">
				{/* Square Logo Box on Left with vertical border */}
				<div className="w-12 h-12 flex items-center justify-center border-r border-ui-border-base shrink-0 bg-ui-bg-base">
					<Cms className="h-5 w-5 text-ui-fg-base" />
				</div>

				{/* Organization Switcher Dropdown Trigger */}
				<DropdownMenu>
					<DropdownMenu.Trigger asChild>
						<button
							type="button"
							className="flex-1 flex items-center justify-between px-3 h-12 hover:bg-ui-bg-subtle-hover transition-colors text-left cursor-pointer outline-none focus-visible:shadow-borders-focus"
						>
							<Text
								size="small"
								weight="plus"
								className="truncate text-ui-fg-base font-semibold"
							>
								{activeWorkspace?.name || "Select Workspace"}
							</Text>
							<ChevronUpDown className="h-4 w-4 text-ui-fg-muted shrink-0 ml-1.5" />
						</button>
					</DropdownMenu.Trigger>

					<DropdownMenu.Content className="w-56" align="start">
						{workspaces.map((ws) => (
							<DropdownMenu.Item
								key={ws.id}
								onClick={() => switchOrganization(ws.id)}
								className="flex items-center gap-2.5 cursor-pointer py-1.5"
							>
								<Avatar
									size="xsmall"
									variant="squared"
									fallback={ws.name.charAt(0) || "W"}
								/>
								<span className="truncate flex-1 font-medium">{ws.name}</span>
								{ws.id === activeWorkspace?.id && (
									<Check className="h-3.5 w-3.5 text-ui-fg-interactive shrink-0" />
								)}
							</DropdownMenu.Item>
						))}

						<DropdownMenu.Separator />

						<DropdownMenu.Item
							onClick={() => navigate({ to: "/setup-organization" })}
							className="flex items-center gap-2.5 cursor-pointer py-1.5 text-ui-fg-base hover:text-ui-fg-base"
						>
							<div className="flex size-5 items-center justify-center">
								<Plus className="h-4 w-4 text-ui-fg-subtle" />
							</div>
							<span>Create Organization</span>
						</DropdownMenu.Item>
					</DropdownMenu.Content>
				</DropdownMenu>
			</div>

			{/* Search Box */}
			<div className="px-3 pt-3 pb-1">
				<div className="relative flex items-center">
					<Input
						type="search"
						size="small"
						placeholder="Search..."
						value={searchQuery}
						onChange={(e) => onSearchChange(e.target.value)}
						className="w-full pr-16 bg-ui-bg-field"
					/>
					<div className="absolute right-2 inset-y-0 flex items-center gap-1 pointer-events-none">
						<Kbd className="text-[10px] h-4.5 px-1 leading-none">Ctrl</Kbd>
						<Kbd className="text-[10px] h-4.5 px-1 leading-none">K</Kbd>
					</div>
				</div>
			</div>

			{/* Navigation Links (Note: NO Payments/Billing per ADR 0001 constraint) */}
			<nav className="flex-1 px-3 py-2 space-y-0.5 overflow-y-auto">
				<Link
					to="/"
					className="bg-ui-bg-base shadow-elevation-card-rest text-ui-fg-base hover:bg-ui-bg-base flex items-center gap-x-2.5 rounded-md px-2.5 py-1.5 outline-none transition-colors"
				>
					<div className="flex size-5 items-center justify-center shrink-0">
						<SquareTwoStack className="h-4 w-4 text-ui-fg-base" />
					</div>
					<Text size="small" weight="plus">
						Projects
					</Text>
				</Link>

				<button
					type="button"
					className="text-ui-fg-subtle hover:text-ui-fg-base hover:bg-ui-bg-subtle-hover flex items-center gap-x-2.5 rounded-md px-2.5 py-1.5 outline-none transition-colors w-full cursor-pointer"
				>
					<div className="flex size-5 items-center justify-center shrink-0">
						<ChartBar className="h-4 w-4" />
					</div>
					<Text size="small">Usage</Text>
				</button>

				<button
					type="button"
					className="text-ui-fg-subtle hover:text-ui-fg-base hover:bg-ui-bg-subtle-hover flex items-center gap-x-2.5 rounded-md px-2.5 py-1.5 outline-none transition-colors w-full cursor-pointer"
				>
					<div className="flex size-5 items-center justify-center shrink-0">
						<Users className="h-4 w-4" />
					</div>
					<Text size="small">Team</Text>
				</button>

				<button
					type="button"
					className="text-ui-fg-subtle hover:text-ui-fg-base hover:bg-ui-bg-subtle-hover flex items-center justify-between rounded-md px-2.5 py-1.5 outline-none transition-colors w-full cursor-pointer"
				>
					<div className="flex items-center gap-x-2.5">
						<div className="flex size-5 items-center justify-center shrink-0">
							<Adjustments className="h-4 w-4" />
						</div>
						<Text size="small">Settings</Text>
					</div>
					<ChevronRightMini className="h-4 w-4 text-ui-fg-muted" />
				</button>

				<button
					type="button"
					className="text-ui-fg-subtle hover:text-ui-fg-base hover:bg-ui-bg-subtle-hover flex items-center justify-between rounded-md px-2.5 py-1.5 outline-none transition-colors w-full cursor-pointer"
				>
					<div className="flex items-center gap-x-2.5">
						<div className="flex size-5 items-center justify-center shrink-0">
							<Key className="h-4 w-4" />
						</div>
						<Text size="small">Access keys</Text>
					</div>
					<ChevronRightMini className="h-4 w-4 text-ui-fg-muted" />
				</button>

				<button
					type="button"
					className="text-ui-fg-subtle hover:text-ui-fg-base hover:bg-ui-bg-subtle-hover flex items-center justify-between rounded-md px-2.5 py-1.5 outline-none transition-colors w-full cursor-pointer"
				>
					<div className="flex items-center gap-x-2.5">
						<div className="flex size-5 items-center justify-center shrink-0">
							<UserIcon className="h-4 w-4" />
						</div>
						<Text size="small">Account</Text>
					</div>
					<ChevronRightMini className="h-4 w-4 text-ui-fg-muted" />
				</button>

				<button
					type="button"
					className="text-ui-fg-subtle hover:text-ui-fg-base hover:bg-ui-bg-subtle-hover flex items-center gap-x-2.5 rounded-md px-2.5 py-1.5 outline-none transition-colors w-full cursor-pointer"
				>
					<div className="flex size-5 items-center justify-center shrink-0">
						<Plug className="h-4 w-4" />
					</div>
					<Text size="small">Connections</Text>
				</button>
			</nav>

			{/* User Section at bottom (Medusa Dashboard pattern) */}
			<div className="bg-ui-bg-subtle sticky bottom-0">
				<div className="px-3">
					<Divider variant="dashed" />
				</div>
				<div className="p-3">
					<DropdownMenu>
						<DropdownMenu.Trigger asChild>
							<button
								type="button"
								className="bg-ui-bg-subtle hover:bg-ui-bg-subtle-hover grid w-full cursor-pointer grid-cols-[24px_1fr_15px] items-center gap-2 rounded-md py-1 pe-2 ps-1 outline-none transition-colors focus-visible:shadow-borders-focus"
							>
								<div className="flex size-6 items-center justify-center">
									<Avatar size="xsmall" fallback={userInitial} />
								</div>
								<div className="flex items-center overflow-hidden text-left">
									<Text
										size="xsmall"
										weight="plus"
										leading="compact"
										className="truncate text-ui-fg-base"
									>
										{displayName}
									</Text>
								</div>
								<EllipsisHorizontal className="h-4 w-4 text-ui-fg-muted" />
							</button>
						</DropdownMenu.Trigger>

						<DropdownMenu.Content className="w-52" align="start">
							<div className="flex items-center gap-x-2.5 px-2 py-2">
								<Avatar size="small" fallback={userInitial} />
								<div className="flex flex-col overflow-hidden">
									<Text
										size="small"
										weight="plus"
										className="truncate text-ui-fg-base"
									>
										{displayName}
									</Text>
									<Text size="xsmall" className="text-ui-fg-muted truncate">
										{userEmail}
									</Text>
								</div>
							</div>
							<DropdownMenu.Separator />
							<DropdownMenu.Item
								onClick={handleLogout}
								className="flex items-center gap-x-2 text-ui-fg-error cursor-pointer"
							>
								<ArrowRightOnRectangle className="h-4 w-4" />
								<span>Sign out</span>
							</DropdownMenu.Item>
						</DropdownMenu.Content>
					</DropdownMenu>
				</div>
			</div>
		</aside>
	);
}
