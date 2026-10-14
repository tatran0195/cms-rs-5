import { z } from "zod";

export const createOrganizationSchema = z.object({
	name: z
		.string()
		.trim()
		.min(2, "Organization name must be at least 2 characters")
		.max(64, "Organization name cannot exceed 64 characters"),
});

export type CreateOrganizationInput = z.infer<typeof createOrganizationSchema>;
