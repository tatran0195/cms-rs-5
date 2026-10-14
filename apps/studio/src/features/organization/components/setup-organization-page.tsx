import { ArrowRightOnRectangle, QuestionMark } from "@cms/icons";
import { Button, Heading, Input, Text, toast } from "@cms/ui";
import { zodResolver } from "@hookform/resolvers/zod";
import { useNavigate } from "@tanstack/react-router";
import { Controller, useForm } from "react-hook-form";
import { useAuth } from "../../auth";
import { useWorkspaceStore } from "../../workspace";
import {
	type CreateOrganizationInput,
	createOrganizationSchema,
} from "../schemas/organization-schema";

export function SetupOrganizationPage() {
	const navigate = useNavigate();
	const { user, logout } = useAuth();
	const createOrganization = useWorkspaceStore(
		(state) => state.createOrganization,
	);

	const defaultName = user?.fullName
		? `${user.fullName}'s projects`
		: "Alex Rivera's projects";

	const {
		control,
		handleSubmit,
		formState: { errors, isSubmitting },
	} = useForm<CreateOrganizationInput>({
		resolver: zodResolver(createOrganizationSchema),
		defaultValues: {
			name: defaultName,
		},
	});

	const onSubmit = (data: CreateOrganizationInput) => {
		const newWs = createOrganization(data.name);
		toast.success(`Organization "${newWs.name}" created!`);
		navigate({ to: "/" });
	};

	const handleSignOut = async () => {
		await logout();
		navigate({ to: "/login" });
	};

	const userEmail = user?.email || "alex@acme.example";

	return (
		<div className="min-h-screen bg-ui-bg-subtle flex flex-col justify-between selection:bg-ui-bg-highlight">
			{/* Top Bar: Sign Out */}
			<header className="w-full flex justify-end px-8 py-5">
				<button
					type="button"
					onClick={handleSignOut}
					className="flex items-center gap-1.5 text-xs text-ui-fg-muted hover:text-ui-fg-base transition-colors font-mono cursor-pointer"
				>
					<ArrowRightOnRectangle className="h-3.5 w-3.5" />
					<span>Sign out ({userEmail})</span>
				</button>
			</header>

			{/* Main Center Content */}
			<main className="flex-1 flex flex-col items-center justify-center px-4 -mt-16">
				<div className="w-full max-w-xl flex flex-col items-center">
					{/* Icon Header */}
					<div className="h-10 w-10 rounded-full border border-ui-border-base bg-ui-bg-base flex items-center justify-center text-ui-fg-subtle mb-4 shadow-xs">
						<QuestionMark className="h-4 w-4" />
					</div>

					{/* Title & Subtitle */}
					<Heading level="h1" className="text-lg font-semibold text-ui-fg-base text-center">
						Setup your organization
					</Heading>
					<Text size="small" className="text-ui-fg-subtle text-center mt-1">
						Specify the name of your organization
					</Text>

					{/* Dotted border container matching Screenshot 2 */}
					<div className="w-full mt-10 pt-8 pb-8 border-t border-b border-dashed border-ui-border-base">
						<form onSubmit={handleSubmit(onSubmit)} className="space-y-8">
							<div className="grid grid-cols-1 md:grid-cols-2 gap-8 items-start">
								<div>
									<Text className="text-xs font-medium text-ui-fg-base">
										Setup your organization
									</Text>
									<Text size="small" className="text-ui-fg-muted text-xs mt-0.5">
										Specify the name of your organization
									</Text>
								</div>

								<div className="space-y-1.5">
									<label
										htmlFor="org-name"
										className="block text-xs font-medium text-ui-fg-subtle"
									>
										Organization name
									</label>
									<Controller
										name="name"
										control={control}
										render={({ field }) => (
											<Input
												{...field}
												id="org-name"
												aria-invalid={Boolean(errors.name)}
												placeholder="Alex Rivera's projects"
												className="bg-ui-bg-base text-xs"
											/>
										)}
									/>
									{errors.name && (
										<p className="text-xs text-ui-fg-error">
											{errors.name.message}
										</p>
									)}
								</div>
							</div>

							<div className="pt-2">
								<Button
									type="submit"
									variant="primary"
									isLoading={isSubmitting}
									className="w-full h-10 text-xs font-medium bg-ui-fg-base text-ui-bg-base hover:bg-ui-fg-subtle"
								>
									Create
								</Button>
							</div>
						</form>
					</div>
				</div>
			</main>

			{/* Footer Step Indicator */}
			<footer className="w-full text-center py-6 text-[11px] text-ui-fg-muted font-mono">
				Step 01 / 02
			</footer>
		</div>
	);
}
