# GEMINI.md — CMS Modern Documentation Platform

Please refer directly to the comprehensive repository rules, constraints, tech stack standards, and architecture guidelines defined in:

- [AGENTS.md](file:///d:/Workspace/Software/Cloned-Repos/cms-rs-4/AGENTS.md)
- [ADR 0001: Internal Company Scope](file:///d:/Workspace/Software/Cloned-Repos/cms-rs-4/docs/adr/0001-internal-company-scope.md)
- [ADR 0002: Three-Layer Crate Architecture (Superseded)](file:///d:/Workspace/Software/Cloned-Repos/cms-rs-4/docs/adr/0002-three-layer-crate-architecture.md)
- [ADR 0003: SQLx Without Heavy ORMs](file:///d:/Workspace/Software/Cloned-Repos/cms-rs-4/docs/adr/0003-sqlx-compile-time-no-orm.md)
- [ADR 0004: Hybrid Search Architecture](file:///d:/Workspace/Software/Cloned-Repos/cms-rs-4/docs/adr/0004-hybrid-search-lindera-tantivy-fastembed.md)
- [ADR 0005: Frontend State and Forms](file:///d:/Workspace/Software/Cloned-Repos/cms-rs-4/docs/adr/0005-react-router-v7-state-and-forms.md)
- [ADR 0006: Zero-Docker Windows Deployment](file:///d:/Workspace/Software/Cloned-Repos/cms-rs-4/docs/adr/0006-zero-docker-single-machine-windows-deployment.md)
- [ADR 0007: Modular Monolith Transition to Vertical Slices](file:///d:/Workspace/Software/Cloned-Repos/cms-rs-4/docs/adr/0007-vertical-slices-modular-monolith.md)

## 🚫 Primary Constraints Summary

1. **Internal Deployment Only**: Dedicated internal company documentation platform. NOT a public multi-tenant SaaS.
2. **No Plan, Billing, or Pricing**: No subscription models, no pricing tiers, no payment gateways (Stripe, etc.), no quota paywalls.
3. **No Marketing Pages**: Root `/` redirects to `/app`. No promotional blogs, marketing routes, or landing sites.
4. **Strict Rust Standards**: Edition 2024, `forbid(unsafe_code)`, strict clippy, no `unwrap` or `panic`.
5. **Strict Frontend Standards**: React Router v7, TanStack Query for server state, Zustand for client state, React Hook Form + Zod for forms, Biome linting.
