# ADR 0001: Internal Company Platform Scope & No SaaS Billing/Marketing

- **Status**: Accepted
- **Date**: 2026-10-12
- **Deciders**: Architecture Team

## Context

This CMS repository is built and maintained as an internal company documentation platform. It provides project management, Notion-style markdown editing, asset management, and published documentation site viewing for company engineering and product teams.

Previous iterations or generic templates included SaaS artifacts such as subscription plans, billing tiers, Stripe payment gateways, and public marketing landing pages.

## Decision

1. **Internal Deployment Only**:
   - The platform is strictly for internal company deployment. It is NOT a public multi-tenant SaaS product.
2. **Absolute Prohibition of Billing & Subscription Features**:
   - Do NOT implement, scaffold, or re-introduce any Plan, Billing, or Subscription features.
   - No `PlanSection`, `plan` tabs, or billing management in project settings, workspace settings, or dashboard overviews.
   - No pricing tiers (Free, Pro, Team, Enterprise).
   - No payment gateways (Stripe, Polar, LemonSqueezy, Paddle).
   - No billing quota enforcement or upgrade prompts. Operational usage tracking (page count, storage) is telemetry only and must never gate features.
3. **No Marketing, Landing, or Blog Pages**:
   - The root route `/` intentionally redirects directly to `/app`.
   - No marketing landing pages at `/`.
   - No marketing routes (`/pricing`, `/cloud`, `/compare/*`, `/alternatives/*`, `/blog/*`, `/about`, `/contact`, `/developers`, `/guides`).
   - Authentication is internal team login/signup without public SaaS terms-of-service or privacy policy checkboxes.

## Consequences

- Prevents architectural bloat and accidental re-introduction of SaaS code.
- Simplifies routing, authorization, and data models.
- Guarantees that all AI agents maintain the focus on internal documentation workflows.
