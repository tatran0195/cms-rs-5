# ADR 0005: Frontend Architecture, State Boundaries, and Form Standards

- **Status**: Accepted
- **Date**: 2026-10-12
- **Deciders**: Architecture Team

## Context

Frontend applications (`apps/studio` and `apps/site`) require clean separation between:
- Server cache (remote entity records, search indices, project trees).
- Durable client state (authenticated session, selected workspace/project, editor UI state).
- Form lifecycle and field-level validation (editing project settings, markdown documents, metadata).

Without strict architectural boundaries, React applications suffer from state synchronization bugs, unnecessary `useEffect` spaghetti, and inconsistent validation between frontend and backend.

## Decision

1. **Framework & Routing**:
   - React 18 + TypeScript with React Router v7.
   - Vite for fast bundling.
   - Biome for strict linting and formatting.
2. **Server State via TanStack Query v5**:
   - All backend data fetching and mutation is managed through TanStack Query (`useQuery`, `useMutation`).
   - No raw `fetch` calls in React UI components.
   - Queries and mutations are wrapped in modular service/query hooks (e.g. `useProjectQuery()`, `useUpdatePageMutation()`).
   - Do NOT store server response lists or records in Zustand or component `useState`.
3. **Client State via Zustand**:
   - Zustand is reserved strictly for durable client-owned state:
     - Authenticated user identity and active session.
     - Selected workspace ID, project ID, active document ID.
     - UI layout states (sidebar collapsed, active editor split panes, preview toggles).
4. **Form Management via React Hook Form + Zod**:
   - **All user-editable form payloads must use React Hook Form** (`useForm`, `Controller`, `FormProvider`).
   - Prohibit one `useState` per form input field.
   - Schema validation must be backed by Zod (`zodResolver(schema)`).
   - Validation schemas match the backend Rust request structs and database constraints.
   - Derived values and submitted payloads are calculated inside `handleSubmit`, never synchronized through `useEffect`.
5. **Strict `useEffect` Policy**:
   - Prohibit `useEffect` for data fetching (use TanStack Query).
   - Prohibit `useEffect` for derived state (use `useMemo` or compute inline).
   - Prohibit `useEffect` for syncing form fields (use `defaultValues` or `form.reset()`).
   - `useEffect` is strictly permitted only for external integrations (DOM listeners, keyboard shortcuts, WebSocket lifecycle).

## Consequences

- Eliminates race conditions and stale state bugs.
- Guarantees predictable form submission and instant validation feedback.
- Ensures consistent code style across all studio and site views.
