import {
	useCreateDocPage,
	useCreateSite,
	useDeleteDocPage,
	useDeleteSite,
	useDocPages,
	useListSites,
	useMarkAllNotificationsRead,
	useMarkNotificationRead,
	useNotifications,
	useUnreadNotificationCount,
} from "@cms/client/react";
import { globalConfig } from "@cms/docs-utils/global-config";
// 3. @cms/i18n
import { INTERFACE_LOCALES, type Locale } from "@cms/i18n/locales";
import { useLocale } from "@cms/i18n/react";
// 2. @cms/icons
import {
	ArrowLeft,
	BellAlert,
	Bolt,
	BookOpen,
	Check,
	CirclePlus,
	Cms,
	CommandLine,
	Globe,
	Layers3,
	MagnifyingGlass,
	Moon,
	ShieldCheck,
	Sparkles,
	Sun,
} from "@cms/icons";
// 4. @cms/types
import type { DocsConfig, NavigationItem } from "@cms/types";
// 1. @cms/ui
import {
	Badge,
	Button,
	CodeBlock,
	Container,
	Heading,
	InlineTip,
	Input,
	Kbd,
	StatusBadge,
	Switch,
	Text,
	Textarea,
	toast,
} from "@cms/ui";
import { createFileRoute, Link } from "@tanstack/react-router";
import { useState } from "react";
import { DEMO_CREDENTIALS, useAuth } from "../features/auth";

// Validated DocsConfig type contract
const docsMeta: Pick<DocsConfig, "version"> = globalConfig;

// 6. @cms/docs-ui
import { Badge as DocsBadge, Kbd as DocsKbd, Note } from "@cms/docs-ui";

export const Route = createFileRoute("/test")({ component: StudioTestDashboard });

// Mock data typed with @cms/types NavigationItem
const testNavigation: NavigationItem[] = [
	{
		type: "link",
		title: "Documentation Overview",
		link: "/docs/overview",
		sidebar_id: "docs_overview",
	},
	{
		type: "dropdown",
		title: "API Reference",
		children: [
			{ type: "link", title: "Store API", link: "/api/store" },
			{ type: "link", title: "Admin API", link: "/api/admin" },
		],
		sidebar_id: "api_ref",
	},
];

const packagesList = [
	{
		name: "@cms/ui",
		description: "Core UI Component library (Radix + Tailwind CSS)",
		status: "Operational",
		type: "Buildable Package",
	},
	{
		name: "@cms/icons",
		description: "SVG icon collection with 460+ React icon components",
		status: "Operational",
		type: "Source Package",
	},
	{
		name: "@cms/i18n",
		description: "Internationalization runtime & React hooks",
		status: "Operational",
		type: "Buildable Package",
	},
	{
		name: "@cms/ui-preset",
		description: "Tailwind CSS v4 theme tokens, color scales & CSS variables",
		status: "Operational",
		type: "Theme Package",
	},
	{
		name: "@cms/types",
		description: "Shared TypeScript interfaces, navigation & docs models",
		status: "Operational",
		type: "Buildable Package",
	},
	{
		name: "@cms/docs-utils",
		description: "Markdown AST, configuration & slug utility functions",
		status: "Operational",
		type: "Buildable Package",
	},
	{
		name: "@cms/docs-ui",
		description: "Documentation components (Notes, Cards, Badges)",
		status: "Operational",
		type: "Buildable Package",
	},
	{
		name: "@cms/tailwind",
		description: "Tailwind CSS v4 theme, medusa color tokens & docs spacing",
		status: "Operational",
		type: "Theme Package",
	},
];

const snippetSample = [
	{
		label: "test-imports.ts",
		language: "typescript",
		code: `// Verified Workspace Package Imports in @cms/studio
import { Button, Badge, CodeBlock, toast } from "@cms/ui";
import { Cms, Sparkles, ShieldCheck } from "@cms/icons";
import { useLocale } from "@cms/i18n/react";
import type { NavigationItem } from "@cms/types";
import { globalConfig } from "@cms/docs-utils/global-config";
import { Note, Badge as DocsBadge } from "@cms/docs-ui";

console.log("All packages compiled & bundled cleanly with Vite+!");`,
	},
];

function StudioTestDashboard() {
	const { locale, setLocale } = useLocale();
	const { user, logout, login } = useAuth();
	const [showNotifications, setShowNotifications] = useState<boolean>(false);
	const [activeTab, setActiveTab] = useState<
		"overview" | "sites" | "ui" | "icons" | "i18n" | "docs"
	>("overview");
	const [switchEnabled, setSwitchEnabled] = useState(true);
	const [inputText, setInputText] = useState("Testing @cms packages");

	// Site & Page management state
	const [selectedSiteId, setSelectedSiteId] = useState<string | null>(null);
	const [selectedPageId, setSelectedPageId] = useState<string | null>(null);
	const [isCreatingPage, setIsCreatingPage] = useState<boolean>(false);
	const [newPageTitle, setNewPageTitle] = useState("");
	const [newPageSlug, setNewPageSlug] = useState("");
	const [newPageContent, setNewPageContent] = useState("");

	const { data: sitesData = [] } = useListSites({ enabled: Boolean(user) });
	const { data: notifData } = useNotifications(
		{ limit: 10 },
		{ enabled: Boolean(user) },
	);
	const { data: unreadData } = useUnreadNotificationCount({
		enabled: Boolean(user),
	});

	const createSiteMutation = useCreateSite();
	const deleteSiteMutation = useDeleteSite();
	const createDocPageMutation = useCreateDocPage();
	const deleteDocPageMutation = useDeleteDocPage();
	const markReadMutation = useMarkNotificationRead();
	const markAllReadMutation = useMarkAllNotificationsRead();

	const sites = user ? sitesData : [];
	const selectedSite = sites.find((s) => s.id === selectedSiteId) ?? null;

	const { data: docPages = [], isLoading: isDocPagesLoading } = useDocPages(
		selectedSiteId ?? "",
		{ enabled: Boolean(selectedSiteId && user) },
	);

	const activeDocPage =
		docPages.find((p) => p.id === selectedPageId) || docPages[0] || null;

	const notifications = user ? (notifData?.notifications ?? []) : [];
	const unreadCount = user
		? (unreadData?.unread_count ?? notifData?.unread_count ?? 0)
		: 0;

	return (
		<div className="min-h-screen bg-ui-bg-subtle text-ui-fg-base p-6 md:p-10 font-sans">
			<div className="max-w-6xl mx-auto space-y-8">
				{/* Header */}
				<header className="flex flex-col md:flex-row md:items-center justify-between gap-4 border-b border-ui-border-base pb-6">
					<div className="flex items-center gap-3">
						<div className="p-2.5 bg-ui-bg-base border border-ui-border-base rounded-xl shadow-xs">
							<Cms className="w-8 h-8 text-ui-fg-base" />
						</div>
						<div>
							<div className="flex items-center gap-2">
								<Heading level="h1" className="text-xl md:text-2xl font-bold">
									CMS Documentation Studio
								</Heading>
								<Badge color="green">v0.1.0-alpha</Badge>
							</div>
							<Text className="text-ui-fg-subtle text-sm">
								Multi-site enterprise documentation platform & developer portal
							</Text>
						</div>
					</div>

					{/* Global Action / Status & Auth Links */}
					<div className="flex items-center gap-2">
						{user ? (
							<div className="flex items-center gap-3">
								{/* Notification Bell */}
								<div className="relative">
									<Button
										size="small"
										variant="secondary"
										onClick={() => setShowNotifications((prev) => !prev)}
										className="relative flex items-center gap-1.5"
									>
										<BellAlert className="w-4 h-4" />
										{unreadCount > 0 && (
											<span className="bg-rose-500 text-white text-[10px] font-bold px-1.5 py-0.2 rounded-full leading-none">
												{unreadCount}
											</span>
										)}
									</Button>

									{showNotifications && (
										<div className="absolute right-0 mt-2 w-80 bg-ui-bg-base border border-ui-border-base rounded-xl shadow-lg p-3 z-50">
											<div className="flex items-center justify-between pb-2 mb-2 border-b border-ui-border-base">
												<span className="text-xs font-semibold text-ui-fg-base">
													Notifications
												</span>
												{unreadCount > 0 && (
													<button
														type="button"
														onClick={() => {
															markAllReadMutation.mutate(undefined, {
																onSuccess: () => {
																	toast.success(
																		"All notifications marked as read",
																	);
																},
															});
														}}
														className="text-[11px] text-ui-fg-interactive hover:underline cursor-pointer"
													>
														Mark all as read
													</button>
												)}
											</div>
											{notifications.length === 0 ? (
												<p className="text-xs text-ui-fg-muted text-center py-4">
													No new notifications
												</p>
											) : (
												<div className="space-y-1.5 max-h-60 overflow-y-auto">
													{notifications.map((n) => (
														<button
															type="button"
															key={n.id}
															onClick={() => {
																if (!n.is_read) {
																	markReadMutation.mutate(n.id);
																}
															}}
															className={`w-full text-left p-2 rounded-lg text-xs cursor-pointer transition-colors ${
																n.is_read
																	? "opacity-60 hover:bg-ui-bg-subtle"
																	: "bg-ui-bg-field hover:bg-ui-bg-subtle border border-ui-border-base/50"
															}`}
														>
															<p className="font-semibold text-ui-fg-base">
																{n.title}
															</p>
															{n.body && (
																<p className="text-ui-fg-subtle text-[11px] mt-0.5">
																	{n.body}
																</p>
															)}
															<span className="text-[10px] text-ui-fg-muted mt-1 block">
																{new Date(n.created_at).toLocaleTimeString([], {
																	hour: "2-digit",
																	minute: "2-digit",
																})}
															</span>
														</button>
													))}
												</div>
											)}
										</div>
									)}
								</div>

								<div className="flex items-center gap-3 bg-ui-bg-base border border-ui-border-base px-3 py-1.5 rounded-xl shadow-xs">
									<div className="flex flex-col text-right">
										<span className="text-xs font-semibold text-ui-fg-base">
											{user.fullName || user.full_name || user.email}
										</span>
										<span className="text-[10px] text-ui-fg-muted uppercase tracking-wider">
											{user.role || "Member"}
										</span>
									</div>
									<Button
										size="small"
										variant="secondary"
										onClick={() => {
											logout();
											toast.info("Logged out successfully");
										}}
									>
										Log out
									</Button>
								</div>
							</div>
						) : (
							<>
								<Link to="/login">
									<Button variant="secondary">Log in</Button>
								</Link>
								<Link to="/signup">
									<Button variant="primary">Sign up</Button>
								</Link>
							</>
						)}
						<Button
							variant="transparent"
							onClick={() =>
								toast.info("Triggered from @cms/ui toast utility!")
							}
						>
							<Sparkles className="w-4 h-4 mr-1.5" />
							Toast
						</Button>
					</div>
				</header>

				{/* Navigation Tabs */}
				<nav className="flex flex-wrap gap-2 border-b border-ui-border-base pb-3">
					{(
						[
							{ id: "overview", label: "Overview & Packages" },
							{ id: "sites", label: `Documentation Sites (${sites.length})` },
							{ id: "ui", label: "@cms/ui Components" },
							{ id: "icons", label: "@cms/icons Gallery" },
							{ id: "i18n", label: "@cms/i18n Localization" },
							{ id: "docs", label: "@cms/docs-ui & Utils" },
						] as const
					).map((tab) => (
						<button
							key={tab.id}
							type="button"
							onClick={() => setActiveTab(tab.id)}
							className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors cursor-pointer border ${
								activeTab === tab.id
									? "bg-ui-bg-base text-ui-fg-base border-ui-border-strong shadow-xs"
									: "bg-transparent text-ui-fg-subtle border-transparent hover:text-ui-fg-base hover:bg-ui-bg-base-hover"
							}`}
						>
							{tab.label}
						</button>
					))}
				</nav>

				{/* TAB 1: OVERVIEW & STATUS */}
				{activeTab === "overview" && (
					<div className="space-y-6">
						{/* Monorepo Status Banner */}
						<div className="p-5 bg-ui-bg-base border border-ui-border-base rounded-2xl shadow-xs flex flex-col md:flex-row items-start md:items-center justify-between gap-4">
							<div className="flex items-start gap-3">
								<div className="p-2 bg-emerald-50 dark:bg-emerald-950/30 text-emerald-600 dark:text-emerald-400 rounded-lg">
									<ShieldCheck className="w-6 h-6" />
								</div>
								<div>
									<h2 className="font-semibold text-ui-fg-base">
										All Workspace Packages Linked & Loaded
									</h2>
									<p className="text-sm text-ui-fg-subtle mt-0.5">
										Build configurations (tsdown, Tailwind CSS v4, Biome, Vite+)
										are operating in harmony.
									</p>
								</div>
							</div>
							<div className="flex items-center gap-2">
								<StatusBadge color="green">8 / 8 Active</StatusBadge>
							</div>
						</div>

						{/* Packages Grid */}
						<div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
							{packagesList.map((pkg) => (
								<div
									key={pkg.name}
									className="p-5 bg-ui-bg-base border border-ui-border-base rounded-xl shadow-xs flex flex-col justify-between hover:border-ui-border-strong transition-all"
								>
									<div>
										<div className="flex items-center justify-between mb-2">
											<span className="font-mono text-sm font-semibold text-ui-fg-base">
												{pkg.name}
											</span>
											<StatusBadge color="green">Ready</StatusBadge>
										</div>
										<p className="text-xs text-ui-fg-subtle mb-3">
											{pkg.description}
										</p>
									</div>
									<div className="pt-3 border-t border-ui-border-base/50 flex items-center justify-between text-xs text-ui-fg-muted">
										<span>{pkg.type}</span>
										<span className="font-mono">workspace:*</span>
									</div>
								</div>
							))}
						</div>

						{/* Code Verification Block */}
						<div>
							<h3 className="text-sm font-semibold uppercase tracking-wider text-ui-fg-subtle mb-3">
								Package Import Verification Code
							</h3>
							<CodeBlock snippets={snippetSample}>
								<CodeBlock.Header />
								<CodeBlock.Body />
							</CodeBlock>
						</div>
					</div>
				)}

				{/* TAB: DOCUMENTATION SITES */}
				{activeTab === "sites" && (
					<div className="space-y-6">
						{/* Sites Overview / Header */}
						<div className="p-5 bg-ui-bg-base border border-ui-border-base rounded-2xl shadow-xs flex flex-col md:flex-row items-start md:items-center justify-between gap-4">
							<div className="flex items-start gap-3">
								<div className="p-2 bg-blue-50 dark:bg-blue-950/30 text-blue-600 dark:text-blue-400 rounded-lg">
									<Globe className="w-6 h-6" />
								</div>
								<div>
									<h2 className="font-semibold text-ui-fg-base">
										{selectedSite
											? selectedSite.name
											: "Multi-Site Documentation Workspaces"}
									</h2>
									<p className="text-sm text-ui-fg-subtle mt-0.5">
										{selectedSite
											? selectedSite.description || `/${selectedSite.slug}`
											: "Manage documentation sites, developer portals, and guides across your company."}
									</p>
								</div>
							</div>
							<div className="flex items-center gap-2">
								{selectedSite ? (
									<>
										<Button
											variant="secondary"
											onClick={() => {
												setSelectedSiteId(null);
												setSelectedPageId(null);
												setIsCreatingPage(false);
											}}
										>
											<ArrowLeft className="w-4 h-4 mr-1.5" />
											All Sites
										</Button>
										<Button
											variant="primary"
											onClick={() => setIsCreatingPage((prev) => !prev)}
										>
											<CirclePlus className="w-4 h-4 mr-1.5" />
											{isCreatingPage ? "Close Form" : "Add Page"}
										</Button>
										<Button
											variant="danger"
											isLoading={deleteSiteMutation.isPending}
											onClick={async () => {
												try {
													await deleteSiteMutation.mutateAsync(selectedSite.id);
													toast.success(`Deleted site: ${selectedSite.name}`);
													setSelectedSiteId(null);
												} catch {
													toast.error("Failed to delete site");
												}
											}}
										>
											Delete Site
										</Button>
									</>
								) : user ? (
									<Button
										variant="primary"
										isLoading={createSiteMutation.isPending}
										onClick={async () => {
											try {
												const newSlug = `docs-${Date.now().toString(36)}`;
												const created = await createSiteMutation.mutateAsync({
													name: `Documentation (${newSlug})`,
													slug: newSlug,
													description: "New multi-site documentation workspace",
													is_public: true,
												});
												toast.success(`Created site: ${created.name}`);
											} catch {
												toast.error("Failed to create site");
											}
										}}
									>
										<CirclePlus className="w-4 h-4 mr-1.5" />
										New Site
									</Button>
								) : (
									<Link to="/login">
										<Button variant="primary">Log in to manage sites</Button>
									</Link>
								)}
							</div>
						</div>

						{/* Detail View for Selected Site */}
						{selectedSite ? (
							<div className="space-y-6">
								{/* Create Page Form Drawer/Card */}
								{isCreatingPage && (
									<div className="p-6 bg-ui-bg-base border border-ui-border-base rounded-2xl shadow-xs space-y-4">
										<Heading level="h3" className="text-sm font-semibold">
											Create New Documentation Page in {selectedSite.name}
										</Heading>
										<div className="grid grid-cols-1 md:grid-cols-2 gap-4">
											<div>
												<label
													htmlFor="doc-title"
													className="block text-xs font-medium text-ui-fg-subtle mb-1"
												>
													Page Title
												</label>
												<Input
													id="doc-title"
													placeholder="e.g. Getting Started"
													value={newPageTitle}
													onChange={(e) => {
														setNewPageTitle(e.target.value);
														if (
															!newPageSlug ||
															newPageSlug.startsWith("page-")
														) {
															setNewPageSlug(
																e.target.value
																	.toLowerCase()
																	.replace(/[^a-z0-9]+/g, "-"),
															);
														}
													}}
												/>
											</div>
											<div>
												<label
													htmlFor="doc-slug"
													className="block text-xs font-medium text-ui-fg-subtle mb-1"
												>
													Page Slug (URL Path)
												</label>
												<Input
													id="doc-slug"
													placeholder="e.g. getting-started"
													value={newPageSlug}
													onChange={(e) => setNewPageSlug(e.target.value)}
												/>
											</div>
										</div>
										<div>
											<label
												htmlFor="doc-content"
												className="block text-xs font-medium text-ui-fg-subtle mb-1"
											>
												Markdown Content
											</label>
											<Textarea
												id="doc-content"
												rows={4}
												placeholder="# Header\n\nWrite documentation markdown here..."
												value={newPageContent}
												onChange={(e) => setNewPageContent(e.target.value)}
											/>
										</div>
										<div className="flex items-center justify-end gap-2 pt-2">
											<Button
												variant="secondary"
												onClick={() => setIsCreatingPage(false)}
											>
												Cancel
											</Button>
											<Button
												variant="primary"
												isLoading={createDocPageMutation.isPending}
												onClick={async () => {
													if (!newPageTitle.trim() || !newPageSlug.trim()) {
														toast.error("Please provide both title and slug");
														return;
													}
													try {
														const created =
															await createDocPageMutation.mutateAsync({
																site_id: selectedSite.id,
																title: newPageTitle.trim(),
																slug: newPageSlug.trim(),
																content:
																	newPageContent ||
																	`# ${newPageTitle}\n\nPage documentation content.`,
																status: "published",
															});
														toast.success(`Created page: ${created.title}`);
														setNewPageTitle("");
														setNewPageSlug("");
														setNewPageContent("");
														setIsCreatingPage(false);
														setSelectedPageId(created.id);
													} catch {
														toast.error("Failed to create page");
													}
												}}
											>
												Save & Publish Page
											</Button>
										</div>
									</div>
								)}

								{/* Pages Grid & Viewer */}
								<div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
									{/* Pages List (Sidebar) */}
									<div className="space-y-3">
										<div className="flex items-center justify-between pb-1">
											<span className="text-xs font-semibold text-ui-fg-subtle uppercase tracking-wider">
												Documentation Pages ({docPages.length})
											</span>
											{isDocPagesLoading && (
												<span className="text-xs text-ui-fg-muted">
													Loading...
												</span>
											)}
										</div>

										{docPages.length === 0 ? (
											<div className="p-6 text-center bg-ui-bg-base border border-ui-border-base rounded-xl">
												<BookOpen className="w-8 h-8 text-ui-fg-muted mx-auto mb-2" />
												<p className="text-xs text-ui-fg-subtle">
													No documentation pages yet. Click "+ Add Page" above.
												</p>
											</div>
										) : (
											<div className="space-y-2">
												{docPages.map((page) => (
													<div
														key={page.id}
														className={`p-3 rounded-xl border transition-all flex items-center justify-between gap-3 ${
															activeDocPage?.id === page.id
																? "bg-ui-bg-base border-ui-border-strong shadow-xs ring-1 ring-ui-border-strong"
																: "bg-ui-bg-base border-ui-border-base hover:border-ui-border-strong/70"
														}`}
													>
														<button
															type="button"
															onClick={() => setSelectedPageId(page.id)}
															className="text-left flex-1 cursor-pointer"
														>
															<div className="font-semibold text-xs text-ui-fg-base flex items-center gap-1.5">
																<span>{page.title}</span>
																<Badge color="green" size="small">
																	{page.status}
																</Badge>
															</div>
															<span className="text-[11px] font-mono text-ui-fg-muted block mt-0.5">
																/{page.slug}
															</span>
														</button>
														<Button
															size="small"
															variant="transparent"
															className="text-rose-500 hover:text-rose-600 text-xs px-2"
															onClick={async (e) => {
																e.stopPropagation();
																try {
																	await deleteDocPageMutation.mutateAsync({
																		siteId: selectedSite.id,
																		pageId: page.id,
																	});
																	toast.success(`Deleted ${page.title}`);
																	if (selectedPageId === page.id) {
																		setSelectedPageId(null);
																	}
																} catch {
																	toast.error("Failed to delete page");
																}
															}}
														>
															Delete
														</Button>
													</div>
												))}
											</div>
										)}
									</div>

									{/* Content Preview (Main pane) */}
									<div className="lg:col-span-2">
										{activeDocPage ? (
											<div className="p-6 bg-ui-bg-base border border-ui-border-base rounded-2xl shadow-xs space-y-4">
												<div className="flex items-center justify-between border-b border-ui-border-base pb-3">
													<div>
														<Heading
															level="h2"
															className="text-base font-semibold"
														>
															{activeDocPage.title}
														</Heading>
														<span className="text-xs font-mono text-ui-fg-subtle">
															/{selectedSite.slug}/{activeDocPage.slug}
														</span>
													</div>
													<Badge color="green">{activeDocPage.status}</Badge>
												</div>
												<div className="p-4 bg-ui-bg-subtle border border-ui-border-base rounded-xl font-mono text-xs leading-relaxed whitespace-pre-wrap">
													{activeDocPage.content}
												</div>
												<div className="flex items-center justify-between text-xs text-ui-fg-muted pt-2 border-t border-ui-border-base/50">
													<span>
														Site ID:{" "}
														<span className="font-mono">{selectedSite.id}</span>
													</span>
													<span>
														Updated:{" "}
														{new Date(
															activeDocPage.updated_at,
														).toLocaleTimeString()}
													</span>
												</div>
											</div>
										) : (
											<div className="p-12 text-center bg-ui-bg-base border border-ui-border-base rounded-2xl">
												<BookOpen className="w-10 h-10 text-ui-fg-muted mx-auto mb-2" />
												<p className="text-sm text-ui-fg-subtle">
													Select a page to preview content or add a new page
												</p>
											</div>
										)}
									</div>
								</div>
							</div>
						) : sites.length === 0 ? (
							<div className="p-12 text-center bg-ui-bg-base border border-ui-border-base rounded-2xl">
								<BookOpen className="w-12 h-12 text-ui-fg-muted mx-auto mb-3" />
								<h3 className="font-semibold text-ui-fg-base mb-1">
									No Documentation Sites Yet
								</h3>
								<p className="text-sm text-ui-fg-subtle mb-6">
									{user
										? "Create your first documentation site or API guide."
										: "Sign in or create an account to view and manage documentation sites."}
								</p>
								{!user && (
									<div className="flex flex-wrap items-center justify-center gap-3">
										<Button
											variant="primary"
											onClick={async () => {
												const res = await login({
													email: DEMO_CREDENTIALS.standardEmail,
													password: DEMO_CREDENTIALS.defaultPassword,
												});
												if (res?.user) {
													toast.success(
														"Signed in with Demo Admin credentials!",
													);
												}
											}}
										>
											<Sparkles className="w-4 h-4 mr-1.5" />
											Instant Demo Login
										</Button>
										<Link to="/login">
											<Button variant="secondary">Log in page</Button>
										</Link>
										<Link to="/signup">
											<Button variant="secondary">Sign up page</Button>
										</Link>
									</div>
								)}
							</div>
						) : (
							<div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
								{sites.map((site) => (
									<button
										type="button"
										key={site.id}
										onClick={() => {
											setSelectedSiteId(site.id);
											setSelectedPageId(null);
										}}
										className="w-full text-left p-5 bg-ui-bg-base border border-ui-border-base rounded-xl shadow-xs flex flex-col justify-between hover:border-ui-border-strong hover:shadow-sm cursor-pointer transition-all group"
									>
										<div>
											<div className="flex items-center justify-between mb-2">
												<span className="font-semibold text-sm text-ui-fg-base group-hover:text-blue-600 dark:group-hover:text-blue-400 transition-colors">
													{site.name}
												</span>
												<Badge color={site.is_public ? "green" : "orange"}>
													{site.is_public ? "Public" : "Private"}
												</Badge>
											</div>
											<p className="text-xs text-ui-fg-subtle mb-3">
												{site.description || "No description provided"}
											</p>
										</div>
										<div className="w-full pt-3 border-t border-ui-border-base/50 flex items-center justify-between text-xs text-ui-fg-muted font-mono">
											<span>/{site.slug}</span>
											<span className="text-[11px] text-blue-500 font-sans group-hover:underline">
												Manage Pages &rarr;
											</span>
										</div>
									</button>
								))}
							</div>
						)}
					</div>
				)}

				{/* TAB 2: @cms/ui COMPONENTS */}
				{activeTab === "ui" && (
					<div className="space-y-6">
						<div className="grid grid-cols-1 md:grid-cols-2 gap-6">
							{/* Buttons & Actions */}
							<Container className="p-6 bg-ui-bg-base border border-ui-border-base rounded-xl space-y-4">
								<Heading level="h2" className="text-base font-semibold">
									Buttons & Toasts
								</Heading>
								<Text className="text-sm text-ui-fg-subtle">
									Testing primary, secondary, transparent, and danger button
									variants with interactive feedback.
								</Text>
								<div className="flex flex-wrap gap-3 pt-2">
									<Button
										variant="primary"
										onClick={() => toast.success("Primary clicked!")}
									>
										Primary Action
									</Button>
									<Button
										variant="secondary"
										onClick={() => toast.info("Secondary clicked!")}
									>
										Secondary Action
									</Button>
									<Button
										variant="transparent"
										onClick={() => toast.info("Transparent clicked!")}
									>
										Transparent
									</Button>
									<Button
										variant="danger"
										onClick={() => toast.error("Danger clicked!")}
									>
										Danger Action
									</Button>
								</div>
							</Container>

							{/* Badges & Status Indicators */}
							<Container className="p-6 bg-ui-bg-base border border-ui-border-base rounded-xl space-y-4">
								<Heading level="h2" className="text-base font-semibold">
									Badges & Status Badges
								</Heading>
								<Text className="text-sm text-ui-fg-subtle">
									Color palette tokens and status badge indicators from @cms/ui.
								</Text>
								<div className="flex flex-wrap gap-2 pt-2">
									<Badge color="green">Green</Badge>
									<Badge color="blue">Blue</Badge>
									<Badge color="purple">Purple</Badge>
									<Badge color="orange">Orange</Badge>
									<Badge color="red">Red</Badge>
									<Badge color="grey">Neutral</Badge>
								</div>
								<div className="flex flex-wrap gap-3 pt-2">
									<StatusBadge color="green">Operational</StatusBadge>
									<StatusBadge color="blue">Syncing</StatusBadge>
									<StatusBadge color="orange">Warning</StatusBadge>
									<StatusBadge color="red">Offline</StatusBadge>
								</div>
							</Container>
						</div>

						{/* Form Controls & Tips */}
						<div className="grid grid-cols-1 md:grid-cols-2 gap-6">
							<Container className="p-6 bg-ui-bg-base border border-ui-border-base rounded-xl space-y-4">
								<Heading level="h2" className="text-base font-semibold">
									Input Controls & Switches
								</Heading>
								<div className="space-y-4">
									<div>
										<label
											htmlFor="test-input"
											className="block text-xs font-medium text-ui-fg-subtle mb-1.5"
										>
											Input Component
										</label>
										<Input
											id="test-input"
											value={inputText}
											onChange={(e) => setInputText(e.target.value)}
											placeholder="Type something..."
										/>
									</div>
									<div className="flex items-center justify-between pt-2">
										<div className="space-y-0.5">
											<div className="text-sm font-medium text-ui-fg-base">
												Switch Toggle State
											</div>
											<div className="text-xs text-ui-fg-subtle">
												Currently {switchEnabled ? "Enabled" : "Disabled"}
											</div>
										</div>
										<Switch
											checked={switchEnabled}
											onCheckedChange={setSwitchEnabled}
										/>
									</div>
									<div className="flex items-center gap-2 pt-2 text-xs text-ui-fg-subtle">
										<span>Keyboard Shortcut:</span>
										<Kbd>⌘</Kbd>
										<Kbd>K</Kbd>
									</div>
								</div>
							</Container>

							<Container className="p-6 bg-ui-bg-base border border-ui-border-base rounded-xl space-y-4">
								<Heading level="h2" className="text-base font-semibold">
									Inline Tips & Callouts
								</Heading>
								<InlineTip label="Package Synchronization" variant="neutral">
									Changes made in local packages are immediately reflected in
									the Studio application via Vite HMR.
								</InlineTip>
							</Container>
						</div>
					</div>
				)}

				{/* TAB 3: @cms/icons GALLERY */}
				{activeTab === "icons" && (
					<div className="space-y-6">
						<div className="p-5 bg-ui-bg-base border border-ui-border-base rounded-xl">
							<Heading level="h2" className="text-base font-semibold mb-1">
								@cms/icons Icon Set
							</Heading>
							<Text className="text-sm text-ui-fg-subtle mb-6">
								Verified SVG React icons loaded from the @cms/icons package,
								supporting custom sizes and CSS color inheritance.
							</Text>

							<div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6 gap-3">
								{[
									{ Icon: Cms, name: "Cms" },
									{ Icon: Sparkles, name: "Sparkles" },
									{ Icon: Check, name: "Check" },
									{ Icon: ShieldCheck, name: "ShieldCheck" },
									{ Icon: CommandLine, name: "CommandLine" },
									{ Icon: BookOpen, name: "BookOpen" },
									{ Icon: Bolt, name: "Bolt" },
									{ Icon: MagnifyingGlass, name: "MagnifyingGlass" },
									{ Icon: Layers3, name: "Layers3" },
									{ Icon: Globe, name: "Globe" },
									{ Icon: Sun, name: "Sun" },
									{ Icon: Moon, name: "Moon" },
								].map(({ Icon, name }) => (
									<button
										type="button"
										key={name}
										className="p-4 bg-ui-bg-subtle/50 hover:bg-ui-bg-subtle border border-ui-border-base rounded-xl flex flex-col items-center justify-center gap-2.5 transition-all text-center group cursor-pointer"
										onClick={() => toast.info(`Copied icon: <${name} />`)}
									>
										<Icon className="w-6 h-6 text-ui-fg-subtle group-hover:text-ui-fg-base group-hover:scale-110 transition-transform" />
										<span className="text-xs font-mono text-ui-fg-subtle group-hover:text-ui-fg-base">
											{name}
										</span>
									</button>
								))}
							</div>
						</div>
					</div>
				)}

				{/* TAB 4: @cms/i18n LOCALIZATION */}
				{activeTab === "i18n" && (
					<div className="space-y-6">
						<Container className="p-6 bg-ui-bg-base border border-ui-border-base rounded-xl space-y-6">
							<div>
								<Heading level="h2" className="text-base font-semibold mb-1">
									@cms/i18n Reactive Store & Language Switcher
								</Heading>
								<Text className="text-sm text-ui-fg-subtle">
									Dynamic locale management powered by useLocale() hook and
									INTERFACE_LOCALES.
								</Text>
							</div>

							<div className="p-4 bg-ui-bg-subtle border border-ui-border-base rounded-xl flex items-center justify-between">
								<div className="space-y-1">
									<div className="text-xs text-ui-fg-subtle uppercase tracking-wider font-semibold">
										Active Locale
									</div>
									<div className="text-lg font-mono font-bold text-ui-fg-base">
										{locale}
									</div>
								</div>
								<Badge color="blue">
									{INTERFACE_LOCALES.find((l) => l.code === locale)?.label ||
										locale}
								</Badge>
							</div>

							<div>
								<div className="text-xs font-medium text-ui-fg-subtle mb-3">
									Select Interface Language:
								</div>
								<div className="flex flex-wrap gap-3">
									{INTERFACE_LOCALES.map((loc) => (
										<Button
											key={loc.code}
											variant={locale === loc.code ? "primary" : "secondary"}
											onClick={() => {
												setLocale(loc.code as Locale);
												toast.success(`Language switched to: ${loc.label}`);
											}}
										>
											<Globe className="w-4 h-4 mr-2" />
											{loc.label} ({loc.native})
										</Button>
									))}
								</div>
							</div>
						</Container>
					</div>
				)}

				{/* TAB 5: @cms/docs-ui & @cms/docs-utils */}
				{activeTab === "docs" && (
					<div className="space-y-6">
						{/* @cms/docs-utils Metadata */}
						<Container className="p-6 bg-ui-bg-base border border-ui-border-base rounded-xl space-y-4">
							<div className="flex items-center justify-between">
								<div>
									<Heading level="h2" className="text-base font-semibold">
										@cms/docs-utils Configuration
									</Heading>
									<Text className="text-sm text-ui-fg-subtle">
										Reading global documentation configuration metadata.
									</Text>
								</div>
								<DocsBadge variant="purple" badgeType="default">
									docs-utils
								</DocsBadge>
							</div>

							<div className="grid grid-cols-1 sm:grid-cols-3 gap-4 pt-2">
								<div className="p-3 bg-ui-bg-subtle border border-ui-border-base rounded-lg">
									<div className="text-xs text-ui-fg-subtle">Version</div>
									<div className="font-mono text-sm font-semibold">
										{docsMeta.version.number}
									</div>
								</div>
								<div className="p-3 bg-ui-bg-subtle border border-ui-border-base rounded-lg">
									<div className="text-xs text-ui-fg-subtle">Release Date</div>
									<div className="font-mono text-sm font-semibold">
										{new Date(
											docsMeta.version.releaseDate,
										).toLocaleDateString()}
									</div>
								</div>
								<div className="p-3 bg-ui-bg-subtle border border-ui-border-base rounded-lg truncate">
									<div className="text-xs text-ui-fg-subtle">Release URL</div>
									<a
										href={docsMeta.version.releaseUrl}
										target="_blank"
										rel="noreferrer"
										className="font-mono text-xs text-blue-500 hover:underline truncate block"
									>
										{docsMeta.version.releaseUrl}
									</a>
								</div>
							</div>
						</Container>

						{/* @cms/docs-ui Component Showcase */}
						<Container className="p-6 bg-ui-bg-base border border-ui-border-base rounded-xl space-y-6">
							<div>
								<Heading level="h2" className="text-base font-semibold mb-1">
									@cms/docs-ui Notes & Admonitions
								</Heading>
								<Text className="text-sm text-ui-fg-subtle">
									Pre-styled documentation callouts and badges.
								</Text>
							</div>

							<div className="space-y-4">
								<Note type="default" title="Default Note">
									This callout is rendered by the @cms/docs-ui Note component.
								</Note>

								<Note type="success" title="Success Notice">
									All workspace packages successfully passed compilation and
									bundling.
								</Note>

								<Note type="warning" title="Admonition Test">
									Ensure packages are built with tsdown before running
									production bundle.
								</Note>

								<Note type="error" title="Error Notice">
									Error callout styling verified with high contrast border.
								</Note>
							</div>

							<div className="pt-4 border-t border-ui-border-base">
								<div className="text-xs font-semibold uppercase tracking-wider text-ui-fg-subtle mb-3">
									@cms/docs-ui Badges & Keyboard Tags
								</div>
								<div className="flex flex-wrap items-center gap-3">
									<DocsBadge variant="green" badgeType="default">
										green badge
									</DocsBadge>
									<DocsBadge variant="purple" badgeType="default">
										purple badge
									</DocsBadge>
									<DocsBadge variant="blue" badgeType="default">
										blue badge
									</DocsBadge>
									<DocsBadge variant="orange" badgeType="default">
										orange badge
									</DocsBadge>
									<DocsBadge variant="red" badgeType="default">
										red badge
									</DocsBadge>
									<div className="flex items-center gap-1.5 ml-4">
										<DocsKbd>Ctrl</DocsKbd>
										<span className="text-xs text-ui-fg-subtle">+</span>
										<DocsKbd>Shift</DocsKbd>
										<span className="text-xs text-ui-fg-subtle">+</span>
										<DocsKbd>P</DocsKbd>
									</div>
								</div>
							</div>
						</Container>

						{/* @cms/types Navigation Validation */}
						<Container className="p-6 bg-ui-bg-base border border-ui-border-base rounded-xl space-y-4">
							<Heading level="h2" className="text-base font-semibold">
								@cms/types Data Model Verification
							</Heading>
							<Text className="text-sm text-ui-fg-subtle">
								Typed navigation schema verified with NavigationItem interface.
							</Text>
							<div className="overflow-x-auto">
								<table className="w-full text-left text-sm border-collapse">
									<thead>
										<tr className="border-b border-ui-border-base text-xs text-ui-fg-subtle uppercase">
											<th className="py-2.5 px-3">Title</th>
											<th className="py-2.5 px-3">Type</th>
											<th className="py-2.5 px-3">Sidebar ID</th>
										</tr>
									</thead>
									<tbody>
										{testNavigation.map((item) => (
											<tr
												key={item.title}
												className="border-b border-ui-border-base/50"
											>
												<td className="py-2.5 px-3 font-medium text-ui-fg-base">
													{item.title}
												</td>
												<td className="py-2.5 px-3">
													<Badge color="grey">{item.type}</Badge>
												</td>
												<td className="py-2.5 px-3 font-mono text-xs text-ui-fg-subtle">
													{item.sidebar_id}
												</td>
											</tr>
										))}
									</tbody>
								</table>
							</div>
						</Container>
					</div>
				)}
			</div>
		</div>
	);
}
