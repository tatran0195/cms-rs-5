interface ProjectPreviewThumbnailProps {
	type: "dtc" | "b2b" | "docs";
}

export function ProjectPreviewThumbnail({
	type,
}: ProjectPreviewThumbnailProps) {
	if (type === "dtc") {
		return (
			<div className="w-full h-full min-h-[148px] bg-[#FAF8F5] p-3 rounded-lg flex flex-col justify-between border border-ui-border-base select-none shadow-xs">
				<div className="flex items-center justify-between text-[11px] text-stone-500 border-b border-stone-200/70 pb-2">
					<span className="font-serif tracking-wide font-medium text-stone-800">
						Autumn Essentials
					</span>
					<div className="flex gap-1">
						<span className="h-1.5 w-1.5 rounded-full bg-stone-300" />
						<span className="h-1.5 w-1.5 rounded-full bg-stone-300" />
					</div>
				</div>

				<div className="grid grid-cols-3 gap-2.5 my-auto py-2">
					<div className="aspect-4/3 bg-[#E8E2D9] rounded flex flex-col items-center justify-center p-1.5 relative overflow-hidden">
						<div className="h-6 w-5 rounded bg-stone-400/40" />
						<span className="text-[7px] text-stone-500 mt-1">Linen Shirt</span>
					</div>
					<div className="aspect-4/3 bg-[#D8E0DA] rounded flex flex-col items-center justify-center p-1.5 relative overflow-hidden">
						<div className="h-6 w-5 rounded-full bg-emerald-800/20" />
						<span className="text-[7px] text-stone-500 mt-1">Ceramic Vase</span>
					</div>
					<div className="aspect-4/3 bg-[#E4DDD3] rounded flex flex-col items-center justify-center p-1.5 relative overflow-hidden">
						<div className="h-5 w-5 bg-amber-700/25 rotate-45 rounded-xs" />
						<span className="text-[7px] text-stone-500 mt-1">Brass Tray</span>
					</div>
				</div>

				<div className="flex justify-between items-center text-[9px] text-stone-400 pt-1 border-t border-stone-200/50">
					<span>Medusa Storefront</span>
					<span className="font-mono text-[8px]">next.medusajs.com</span>
				</div>
			</div>
		);
	}

	return (
		<div className="w-full h-full min-h-[148px] bg-[#F8FAFC] p-3 rounded-lg flex flex-col justify-between border border-ui-border-base select-none shadow-xs">
			<div className="flex items-center justify-between text-[11px] text-slate-600 border-b border-slate-200/80 pb-2">
				<span className="font-semibold text-slate-800">Wholesale orders</span>
				<div className="flex gap-2 text-[9px] text-slate-400">
					<span className="text-emerald-600 font-medium">&bull; Connected</span>
				</div>
			</div>

			<div className="grid grid-cols-3 gap-2 my-auto py-2">
				<div className="bg-white border border-slate-200/90 rounded p-1.5 shadow-2xs">
					<span className="text-[8px] text-slate-400 font-medium">Orders</span>
					<div className="text-[12px] font-bold text-slate-700">12</div>
				</div>
				<div className="bg-white border border-slate-200/90 rounded p-1.5 shadow-2xs">
					<span className="text-[8px] text-slate-400 font-medium">Quotes</span>
					<div className="text-[12px] font-bold text-slate-700">3</div>
				</div>
				<div className="bg-white border border-slate-200/90 rounded p-1.5 shadow-2xs">
					<span className="text-[8px] text-slate-400 font-medium">Volume</span>
					<div className="text-[12px] font-bold text-emerald-600">$41,200</div>
				</div>
			</div>

			<div className="flex justify-between items-center text-[9px] text-slate-400 pt-1 border-t border-slate-200/50">
				<span>B2B Portal</span>
				<span className="font-mono text-[8px]">demo.medusajs.com</span>
			</div>
		</div>
	);
}
