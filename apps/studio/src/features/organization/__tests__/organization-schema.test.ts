import { describe, expect, it } from "bun:test";
import { createOrganizationSchema } from "../schemas/organization-schema";

describe("createOrganizationSchema", () => {
	it("validates non-empty organization names", () => {
		const valid = createOrganizationSchema.safeParse({ name: "Acme Corp" });
		expect(valid.success).toBe(true);
		if (valid.success) {
			expect(valid.data.name).toBe("Acme Corp");
		}
	});

	it("rejects empty or single character names", () => {
		const empty = createOrganizationSchema.safeParse({ name: "" });
		expect(empty.success).toBe(false);

		const short = createOrganizationSchema.safeParse({ name: "A" });
		expect(short.success).toBe(false);
	});
});
