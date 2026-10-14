# Design: Home Page, Create Organization Page, and Last Visited Workspace Redirect

## 1. Overview & Context

This design implements the **Home / Projects overview page** and the **Create Organization page** for the Studio frontend application (`apps/studio`), styled after the Medusa Cloud platform architecture and reference mockups. It also implements the **post-login redirect flow**, ensuring authenticated users are automatically navigated to the projects page of their last-visited workspace (or to the organization setup page if no workspace exists yet).

Per user directive, this phase focuses on **high-fidelity UI representation, realistic interactive mock state, and layout fidelity**, enabling full interaction before wiring to the backend Axum API.

---

## 2. Core Constraints & Standards Alignment

- **Internal Scope & No Billing (ADR 0001)**:
  - No plan tiers, subscription modals, payment gateways, or pricing tables.
  - Sidebar and navigation items exclude billing/payments.
- **Form State Standards (ADR 0005 & AGENTS.md)**:
  - All user-editable forms use **React Hook Form** with **Zod** (`zodResolver`).
  - No uncontrolled or ad-hoc raw state for input submissions.
- **Component & Design System**:
  - Leverages `@cms/ui` (Medusa UI components: `Button`, `Input`, `Heading`, `Text`, `Badge`, `StatusBadge`, `DropdownMenu`, `Kbd`, `Avatar`, `Container`, `InlineTip`, `toast`).
  - Leverages `@cms/icons` (Medusa icon library: `ChevronUpDown`, `CirclePlus`, `MagnifyingGlass`, `ArrowRightOnRectangle`, `Folder`, `CommandLine`, `CogSixTooth`, `Key`, `User`, `Globe`, etc.).
- **Routing**:
  - TanStack Router (`@tanstack/react-router`) with route definitions under `apps/studio/src/routes/`.

---

## 3. Architecture & Data Flow

```
[ User Logs In (/login) ]
           │
           ▼
[ Check Workspace Store ]
    │                │
    │ (Has workspace)│ (No workspace)
    ▼                ▼
[ Redirect to / ]  [ Redirect to /setup-organization ]
    │                      │
    ▼                      ▼
[ Home / Projects Page ]  [ Setup Organization Page ]
  - Reads last visited      - User creates organization
  - Renders Projects UI     - Updates last_visited_id
                            - Navigates to / (Home)
```

### 3.1 Workspace State Management (`features/workspace/store/workspace-store.ts`)
- Manages:
  - `workspaces`: Array of user workspaces (`id`, `name`, `slug`, `role`, `createdAt`).
  - `activeWorkspaceId`: Currently selected workspace ID.
  - `lastVisitedWorkspaceId`: Persisted ID of the last active workspace.
  - `projects`: Map/list of projects belonging to each workspace.
- Storage:
  - Backed by `localStorage` (`cms_workspaces_v1` & `cms_last_visited_workspace_id`) to retain state across sessions and page reloads.
- Default Seed Data:
  - **Acme Inc** (`slug: acme-inc`):
    - Project 1: **Acme DTC**
      - Backend: `Live` (`demo.medusajs.com`)
      - Storefront: `Live` (`next.medusajs.com`)
      - Latest Activity: `Ready - to develop ⑂ cf750e1` by `Alex Rivera` (5m ago) - *"Handle partial refunds on split payments"*
    - Project 2: **Acme B2B**
      - Backend: `Live` (`demo.medusajs.com`)
      - Storefront: `Live` (`next.medusajs.com`)
      - Latest Activity: `Ready - to main ⑂ 532c4e3` by `Alex Rivera` (5m ago) - *"Reduce cold start by trimming the bundle"*
  - **Personal Workspace** (`slug: personal`):
    - Allows demonstrating instantaneous organization switching in the UI.

### 3.2 Authentication & Redirect Integration
- In `LoginPage` (`apps/studio/src/features/auth/components/login-page.tsx`):
  - Upon successful credentials verification or MFA completion:
    - Resolves `getActiveWorkspace()` from `workspace-store`.
    - If a workspace exists, navigates to `/`.
    - If no workspace exists, navigates to `/setup-organization`.
- In `IndexRoute` (`apps/studio/src/routes/index.tsx`):
  - Checks authentication status. If unauthenticated, redirects to `/login`.
  - Checks if any workspace exists. If none, redirects to `/setup-organization`.
  - If workspace exists, renders `ProjectsDashboard`.

---

## 4. UI Layout & Component Specifications

### 4.1 Create Organization Page (`/setup-organization`)
- **Route**: `apps/studio/src/routes/setup-organization.tsx`
- **Component**: `apps/studio/src/features/organization/components/setup-organization-page.tsx`
- **Layout & Visuals (1:1 with Reference Screenshot 2)**:
  1. **Top Header**:
     - Minimal top bar with user email and sign-out trigger: `Sign out (${user.email})` with an exit icon (`ArrowRightOnRectangle`).
  2. **Wizard Center Area**:
     - Centered circular icon with an organization/question glyph.
     - Header: *"Setup your organization"*
     - Subtitle: *"Specify the name of your organization"*
     - Subtle divider border.
  3. **Two-Column Form Row**:
     - **Left Column**:
       - Label: *"Setup your organization"*
       - Description: *"Specify the name of your organization"* (subtle foreground text).
     - **Right Column**:
       - Field label: *"Organization name"*
       - Field input: Validated text input with default suggestion (e.g., `"${user.fullName || 'Alex Rivera'}'s projects"`).
  4. **Actions & Footer**:
     - Divider border.
     - Dark action button: **"Create"** (full width of the form card).
     - Bottom footer: *"Step 01 / 02"*.
- **Form State**:
  - Managed with **React Hook Form** + **Zod**:
    ```ts
    export const createOrganizationSchema = z.object({
      name: z.string().trim().min(2, "Organization name must be at least 2 characters").max(50),
    });
    ```
  - On submit: calls `createOrganization(values.name)`, marks it as `last_visited_workspace_id`, shows toast notification, and navigates to `/`.

### 4.2 Home / Projects Dashboard (`/`)
- **Route**: `apps/studio/src/routes/index.tsx`
- **Component**: `apps/studio/src/features/projects/components/projects-dashboard.tsx`
- **Layout & Visuals (1:1 with Reference Screenshot 1)**:
  1. **Left Sidebar (`Sidebar.tsx`)**:
     - **Workspace Switcher Dropdown**:
       - Displays current workspace name (e.g., *"Acme Inc"*) with `ChevronUpDown` icon.
       - Dropdown menu allows switching between existing organizations or selecting *"Create Organization"* (`+` icon).
     - **Quick Search Input**:
       - Placeholder: *"Search..."* with `<Kbd>Ctrl K</Kbd>` badge.
     - **Navigation Menu Items**:
       - **Projects** (active, highlighted with solid selection state)
       - **Usage**
       - **Team**
       - **Settings** (with chevron expansion indicator)
       - **Access keys** (with chevron expansion indicator)
       - **Account**
       - **Connections**
  2. **Top Navigation Bar (`TopNav.tsx`)**:
     - Clean header bar.
     - Right section: Search shortcut, Help/Agent badge button, user Avatar (`"A"`).
  3. **Main Workspace Content (`ProjectsDashboard.tsx`)**:
     - Title bar:
       - Heading: *"All projects"*
       - Right controls:
         - Search input: *"Search projects"*
         - Primary button: **"Create Project"**
     - **Project Cards List (`ProjectCard.tsx`)**:
       - Container with clean border and rounded corners.
       - **Left Preview Area**:
         - Rich documentation/storefront preview canvas showcasing a styled mock layout.
       - **Right Details Area**:
         - Project Title (e.g. *"Acme DTC"*, *"Acme B2B"*).
         - Status Indicators:
           - `Backend` • <span style="color:#10B981">● Live</span> `demo.medusajs.com`
           - `Storefront` • <span style="color:#10B981">● Live</span> `next.medusajs.com`
         - Latest Activity subsection:
           - Status indicator: `● Ready - to develop ⑂ cf750e1`
           - Commit row: Author Avatar (`A`), author name (*"Alex Rivera"*), timestamp (*"5m ago"*), commit message (*"Handle partial refunds on split payments"*).
         - Card Footer Actions:
           - Primary button: **"Project details"**
           - Right action buttons: Grid/view icon, Settings gear icon, GitHub repository icon.

---

## 5. File Structure & Changes

```
apps/studio/src/
├── features/
│   ├── workspace/
│   │   ├── index.ts
│   │   ├── types.ts
│   │   └── store/
│   │       └── workspace-store.ts
│   ├── organization/
│   │   ├── index.ts
│   │   ├── schemas/
│   │   │   └── organization-schema.ts
│   │   └── components/
│   │       └── setup-organization-page.tsx
│   ├── projects/
│   │   ├── index.ts
│   │   ├── types.ts
│   │   └── components/
│   │       ├── projects-dashboard.tsx
│   │       ├── project-card.tsx
│   │       ├── project-preview-thumbnail.tsx
│   │       ├── sidebar.tsx
│   │       └── topbar.tsx
│   └── auth/
│       └── components/
│           └── login-page.tsx (update redirect to use last visited workspace)
└── routes/
    ├── index.tsx (update to render ProjectsDashboard or redirect)
    ├── setup-organization.tsx (new route for organization setup wizard)
    └── routeTree.gen.ts (regenerated via `tsr generate`)
```

---

## 6. Verification & Self-Review

- [x] **Placeholder Scan**: No TODOs or unspecified UI components; all components mapped to `@cms/ui` and `@cms/icons`.
- [x] **Internal Consistency**: Matches repository rules forbidding billing and requiring React Hook Form + Zod.
- [x] **Scope Check**: Focused exclusively on high-fidelity representation of the Home/Projects and Create Organization views, plus the last-visited workspace redirection logic.
- [x] **Ambiguity Check**: Redirect behavior is strictly defined for all states (unauthenticated, authenticated without workspace, authenticated with workspace).
