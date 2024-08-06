import 'package:flutter/material.dart';
import 'package:gap/gap.dart';
import 'package:get/get.dart';
import 'package:midi_to_mml/command_signals.dart';
import 'package:midi_to_mml/controller.dart';
import 'package:midi_to_mml/messages/types.pb.dart';
import 'package:remixicon/remixicon.dart';

class TrackContent extends GetView<AppController> {
	const TrackContent({ super.key });

	void splitTrack() {
		final track = controller.currentTrack();
		if (track == null) return;
		SplitTrack(track.index);
	}

	@override
	Widget build(context) {
		final headerChildren = [
			Obx(() => Text(
				"Track ${controller.currentTrack()?.index}",
				style: Theme.of(context).textTheme.headlineSmall,
			)),
			const SizedBox(width: 0, height: 8),

			Row(mainAxisAlignment: MainAxisAlignment.center, children: [
				TextButton.icon(
					onPressed: splitTrack,
					icon: const Icon(Remix.git_branch_line),
					label: const Text("Split"),
				),
				TextButton.icon(
					onPressed: () => (),
					icon: const Icon(Remix.git_pull_request_line),
					label: const Text("Merge"),
				),
				Obx(() {
					final track = controller.currentTrack();

					IconData icon = Remix.volume_up_line;
					String label = "Mute";

					if (track == null || track.isMuted) {
						icon = Remix.volume_mute_line;
						label = "Muted";
					}

					return TextButton.icon(
						onPressed: () {
							if (track == null) return;
							track.isMuted = !track.isMuted;
							controller.currentTrack.refresh();
						},
						icon: Icon(icon),
						label: Text(label),
					);
				}),
			]),
		];

		final screenWidth = MediaQuery.sizeOf(context).width;
		final isOverBreakpoint = screenWidth > 420;

		return Expanded(
			child: Column(children: [
				Builder(builder: (context) {
					if (isOverBreakpoint) {
						return Padding(
							padding: const EdgeInsets.all(16),
							child: Flex(
								direction: Axis.horizontal,
								mainAxisAlignment: MainAxisAlignment.spaceBetween,
								children: headerChildren,
							),
						);
					}

					return Flex(
						direction: Axis.vertical,
						mainAxisAlignment: MainAxisAlignment.spaceBetween,
						children: headerChildren,
					);
				}),
				const Gap(8),

				Expanded(child: ListView(children: [
					Padding(
						padding: const EdgeInsets.all(16),
						child: Obx(() => Text(controller.currentTrack()?.mml ?? '')),
					),
				])),
			]),
		);
	}
}

class TrackTabButton extends GetView<AppController> {
	final SignalMmlTrack track;

	const TrackTabButton({
		super.key,
		required this.track,
	});

	@override
	Widget build(context) {

		return Column(children: [
			Obx(() => TextButton.icon(
				onPressed: () => controller.currentTrack(track),
				icon: Builder(builder: (context) {
					const icon = ImageIcon(AssetImage("assets/icon-instruments/piano.png"));

					if (track.isMuted) {
						return Badge(
							label: Icon(
								Remix.volume_mute_line,
								color: Theme.of(context).colorScheme.onPrimary,
								size: 12,
							),
							backgroundColor: Theme.of(context).colorScheme.error,
							offset: const Offset(4, -4),
							child: icon,
						);
					}

					return icon;
				}),
				label: Text("Track ${track.index}"),
				style: ButtonStyle(
					shape: const WidgetStatePropertyAll(
						RoundedRectangleBorder(
							borderRadius: BorderRadius.zero,
						),
					),
					backgroundColor: WidgetStatePropertyAll(
						(track.index == controller.currentTrack()?.index) ?
						Get.theme.colorScheme.primaryContainer :
						Colors.transparent
					),
				),
			)),
			const Gap(4),
			Text(
				"${track.mmlNoteLength} notes",
				style: Theme.of(context).textTheme.labelSmall,
			),
			Text(
				track.name,
				style: Theme.of(context).textTheme.labelSmall,
			),
		]);
	}
}
