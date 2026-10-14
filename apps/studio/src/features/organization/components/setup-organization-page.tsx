import { ArrowRightOnRectangle, QuestionMark } from "@cms/icons";
import {
	Button,
	Divider,
	Heading,
	IconBadge,
	Input,
	Label,
	Text,
	toast,
} from "@cms/ui";
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
					{/* Icon Header using IconBadge */}
					<IconBadge
						size="large"
						color="grey"
						className="rounded-full h-10 w-10 mb-4 shadow-xs"
					>
						<QuestionMark className="h-4 w-4" />
					</IconBadge>

					{/* Title & Subtitle using Heading and Text */}
					<Heading
						level="h1"
						className="text-lg font-semibold text-ui-fg-base text-center"
					>
						Setup your organization
					</Heading>
					<Text size="small" className="text-ui-fg-subtle text-center mt-1">
						Specify the name of your organization
					</Text>

					{/* Dashed divider container using Divider from @cms/ui */}
					<div className="w-full mt-8">
						<Divider variant="dashed" />
						<form onSubmit={handleSubmit(onSubmit)} className="py-8 space-y-8">
							<div className="grid grid-cols-1 md:grid-cols-2 gap-8 items-start">
								<div>
									<Text size="small" weight="plus" className="text-ui-fg-base">
										Setup your organization
									</Text>
									<Text size="small" className="text-ui-fg-muted mt-0.5">
										Specify the name of your organization
									</Text>
								</div>

								<div className="space-y-1.5">
									<Label
										size="xsmall"
										weight="plus"
										htmlFor="org-name"
										className="text-ui-fg-subtle"
									>
										Organization name
									</Label>
									<Controller
										name="name"
										control={control}
										render={({ field }) => (
											<Input
												{...field}
												id="org-name"
												size="base"
												aria-invalid={Boolean(errors.name)}
												placeholder="Alex Rivera's projects"
												className="bg-ui-bg-base"
											/>
										)}
									/>
									{errors.name && (
										<Text size="xsmall" className="text-ui-fg-error">
											{errors.name.message}
										</Text>
									)}
								</div>
							</div>

							<div>
								<Button
									type="submit"
									variant="primary"
									size="large"
									isLoading={isSubmitting}
									className="w-full"
								>
									Create
								</Button>
							</div>
						</form>
						<Divider variant="dashed" />
					</div>
				</div>
			</main>

			{/* Footer Step Indicator */}
			<footer className="w-full text-center py-6">
				<Text size="xsmall" family="mono" className="text-ui-fg-muted">
					Step 01 / 02
				</Text>
			</footer>
		</div>
	);
}
