# Rule: Frontend React & TypeScript Coding Standards

## Context

The CMS frontend applications (`apps/studio`, `apps/site`) are built with React 18, TypeScript (strict mode), React Router v7, TanStack Query v5, Zustand, and React Hook Form + Zod.
Linting and formatting are strictly governed by Biome (`biome check`).

---

## 📋 Form State Rules (Mandatory)

1. **React Hook Form is the Single Source of Truth for Forms**:
   - **All user-editable form payloads must use React Hook Form** (`useForm`, `Controller`, `FormProvider`).
   - Prohibit one `useState` per field for form editing.
   - Use `useState` only for UI-only transient state (e.g. selected table row, active tab index, search filter text, visual drawer toggle).
2. **Zod Validation Contracts**:
   - Every backend-backed form must be validated using Zod (`zodResolver(schema)`).
   - Validation schemas must match the backend Rust request structs (required fields, string lengths, enums, regexes).
   - Do NOT add ad-hoc field validation inside components when a Zod schema can represent the rule.
3. **No Form Field Sync via `useEffect`**:
   - Do NOT synchronize form fields through `useEffect`.
   - Initialize with `defaultValues`, call `reset()` when opening a new record or switching entities, and derive submitted payloads inside `handleSubmit`.
4. **Modal & Drawer Visibility**:
   - Pair modal/drawer visibility toggles with form `reset()` when opening a payload-backed form.
5. **No `as` Type Assertions**:
   - Avoid `as` type assertions to force form values into domain types.
   - Use explicit TypeScript union types and typed option arrays.

---

## 🌐 Data Access & State Management

1. **TanStack Query for Server State**:
   - All server data reading and mutation must go through TanStack Query (`useQuery`, `useMutation`).
   - Never make raw `fetch` calls directly inside React components.
   - Encapsulate API queries inside module-specific hooks/services (e.g. `useProjectQuery`, `useCreatePageMutation`).
2. **Zustand for Client State Only**:
   - Store only durable client state in Zustand: authenticated user, active workspace/project ID, editor layout preferences.
   - Do NOT store server lists or records in Zustand when TanStack Query can own caching, refetching, and invalidation.
3. **Strict `useEffect` Policy**:
   - **Avoid `useEffect` unless absolutely necessary.**
   - Never use `useEffect` for data fetching (use TanStack Query).
   - Never use `useEffect` for derived state (use `useMemo` or compute inline).
   - Allowed only for external system integration (keyboard shortcuts, window resize, WebSocket events).

---

## 🎨 UI Component & Markup Discipline

1. **Semantic & Reusable Components**:
   - Use dedicated UI components and design system tokens; avoid scattered arbitrary CSS hex codes.
   - Ensure keyboard accessibility and focus management on all interactive controls.
2. **Linting & Formatting**:
   - Run `biome check` and `biome format`. All commits must be clean with 0 Biome diagnostics.
