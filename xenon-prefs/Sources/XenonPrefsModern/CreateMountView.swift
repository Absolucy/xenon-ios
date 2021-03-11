import Foundation
import SwiftUI
import XenonPrefs

enum MountType {
	case path, icloud, app, preset
}

enum MountCreationState {
	case choose, choosing(MountType), naming(MountPoint)
}

struct CreateMountView: View {
	@Binding var mounts: [String: MountPoint]
	@Binding var daemon_alert: Bool
	@Binding var active_sheet: ActiveSheet?

	@State var state = MountCreationState.choose
	@State var path = ""
	@State var name = ""
	@State var bundles: [String] = []

	@ViewBuilder func ChooseBundle(handler: @escaping ((String) -> Void)) -> some View {
		List {
			ForEach(self.bundles.sorted(), id: \String.self) { bundle in
				Button(bundle) {
					handler(bundle)
				}.padding(.leading).padding(.trailing)
			}
		}
	}

	fileprivate func dir_exists(_ path: String) -> Bool {
		var isDirectory = ObjCBool(true)
		let exists = FileManager.default.fileExists(atPath: path, isDirectory: &isDirectory)
		return exists && isDirectory.boolValue
	}

	@ViewBuilder func ChoosePath() -> some View {
		VStack {
			Form {
				TextField("Valid File Path", text: $path).padding()
			}
			Spacer()
			Form {
				Button("Continue") {
					self.state = .naming(.path(self.path))
				}.disabled(path.count == 0 || !dir_exists(path))
			}
		}
	}

	@ViewBuilder func ChoosePreset() -> some View {
		Form {
			VStack {
				Button("On-Device Photos") {
					self.state = .naming(.preset(.photos))
				}
				HStack(alignment: .center) {
					Spacer()
					Text(
						"The photos stored locally on the device. This does NOT include photos stored ONLY in iCloud, but if they were taken on-device and then uploaded to iCloud, they will probably appear here."
					)
					.font(.caption).multilineTextAlignment(.center)
					Spacer()
				}
			}.padding()
			VStack {
				Button("'On my iPhone' Files") {
					self.state = .naming(.preset(.localfiles))
				}
				HStack(alignment: .center) {
					Spacer()
					Text("Files that appear in the \"On my iPhone\" section of the stock Files app.").font(.caption)
						.multilineTextAlignment(.center)
					Spacer()
				}
			}.padding()
			VStack {
				Button("Home Directory") {
					self.state = .naming(.preset(.home))
				}
				HStack(alignment: .center) {
					Spacer()
					Text("The iOS home directory (/var/mobile), equivalent to /home/mobile on Linux and /Users/mobile on macOS.")
						.font(.caption).multilineTextAlignment(.center)
					Spacer()
				}
			}.padding()
		}
	}

	@ViewBuilder func ChooseName() -> some View {
		VStack {
			Form {
				TextField("Mount Point Name", text: $name).padding()
			}
			Spacer()
			Form {
				Button("Add Mount") {
					if case let .naming(mount) = state {
						mounts[name] = mount
						MountPoint.save_mounts(mounts)
					}
					do {
						try DaemonIPC.communicate_with_daemon(message: "reload-mounts")
					} catch {
						daemon_alert = true
					}
					state = .choose
					active_sheet = .none
				}
				.disabled(name.count == 0 || name
					.range(of: "^[A-Za-z0-9_.-]+$", options: .regularExpression, range: nil, locale: nil) == nil ||
					name.caseInsensitiveCompare("xenon") == .orderedSame)
			}
		}
	}

	@ViewBuilder func ChooseType() -> some View {
		Form {
			VStack {
				Button("App Bundle") {
					self.state = .choosing(.app)
					try? DaemonIPC.communicate_with_daemon(message: "bundles") { (reply: Data) in
						let bundles = ((try? JSONDecoder().decode([String].self, from: reply)) ?? [String]()).sorted()
						for bundle in bundles {
							self.bundles.append(bundle)
						}
					}
				}
				HStack(alignment: .center) {
					Spacer()
					Text("Mounts an app's local storage folder.").font(.caption).multilineTextAlignment(.center)
					Spacer()
				}
			}.padding()
			VStack {
				Button("iCloud Bundle") {
					self.state = .choosing(.icloud)
					try? DaemonIPC.communicate_with_daemon(message: "icloud-bundles") { (reply: Data) in
						let bundles = ((try? JSONDecoder().decode([String].self, from: reply)) ?? [String]()).sorted()
						for bundle in bundles {
							self.bundles.append(bundle)
						}
					}
				}
				HStack(alignment: .center) {
					Spacer()
					Text("Mounts an app's iCloud storage folder.").font(.caption).multilineTextAlignment(.center)
					Spacer()
				}
			}.padding()
			VStack {
				Button("Path") { self.state = .choosing(.path) }
				HStack(alignment: .center) {
					Spacer()
					Text("Mounts a direct folder path from the filesystem.").font(.caption).multilineTextAlignment(.center)
					Spacer()
				}
			}.padding()
			VStack {
				Button("Preset") { self.state = .choosing(.preset) }
				HStack(alignment: .center) {
					Spacer()
					Text("Mounts a folder pre-defined by Xenon for convienence.").font(.caption).multilineTextAlignment(.center)
					Spacer()
				}
			}.padding()
		}
	}

	@ViewBuilder func Choose() -> some View {
		switch self.state {
		case .choose:
			self.ChooseType().transition(.slide).animation(.default)
		case let .choosing(kind):
			switch kind {
			case .path:
				self.ChoosePath().transition(.slide).animation(.default)
			case .icloud:
				self.ChooseBundle { bundle in
					self.state = .naming(.icloudbundle(bundle))
				}.transition(.slide).animation(.default)
			case .app:
				self.ChooseBundle { bundle in
					self.state = .naming(.bundle(bundle))
				}.transition(.slide).animation(.default)
			case .preset:
				self.ChoosePreset().transition(.slide).animation(.default)
			}
		case .naming:
			self.ChooseName().transition(.slide).animation(.default)
		}
	}

	var body: some View {
		VStack {
			HStack {
				Choose()
			}
		}
	}
}
