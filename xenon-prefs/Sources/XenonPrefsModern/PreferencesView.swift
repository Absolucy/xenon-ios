
import SwiftUI
import XenonPrefs

enum ActiveSheet: Identifiable {
	case qr, mount, licenses

	var id: Int {
		hashValue
	}
}

struct PreferencesView: View {
	@State private var mounts: [String: MountPoint]
	@State private var pubkey: String? = nil

	@State private var daemon_alert = false
	@State private var success_alert = false
	@State private var active_sheet: ActiveSheet?

	init() {
		_mounts = State(wrappedValue: MountPoint.load_mounts())
		do {
			try DaemonIPC.communicate_with_daemon(message: "pubkey") { (reply: String) in
				_pubkey = State(wrappedValue: reply)
			}
		} catch {}
	}

	var EncryptionSection: some View {
		Section(header: Text("🔑 Encryption")) {
			pubkey.map { pubkey in
				HStack {
					Text("Public Key").bold()
					Divider()
					Spacer()
					Text(pubkey).font(.caption)
				}
			}
			HStack {
				Image(systemName: "exclamationmark.triangle")
				Button("Regenerate Keys") { self.regenerate_keys() }
			}
		}
	}

	@ViewBuilder func MountPointText(_ mount: MountPoint) -> some View {
		switch mount {
		case let .path(path):
			Text(path).font(.caption)
		case let .bundle(bundle):
			Text(bundle.replacingOccurrences(of: "group.", with: "")).font(.caption)
		case let .icloudbundle(bundle):
			Text(bundle).font(.caption)
		case let .preset(preset):
			switch preset {
			case .photos:
				Text("Local Photos").font(.caption)
			case .localfiles:
				Text("Local Files").font(.caption)
			case .home:
				Text("/var/mobile").font(.caption)
			case .documents:
				Text("Documents").font(.caption)
			}
		}
	}

	@ViewBuilder func MountPointsView() -> some View {
		List {
			ForEach(mounts.keys.sorted(by: { $0 < $1 }), id: \.self) { name in
				self.mounts[name].map { mount in
					HStack {
						Text(name).bold()
						Spacer()
						self.MountPointText(mount)
					}
				}
			}.onDelete(perform: delete_mount)
			HStack {
				Image(systemName: "plus")
				Button("Add New Mount") {
					self.active_sheet = .mount
				}
			}
		}
	}

	var MountPointSection: some View {
		Section(header: Text("💾 Mount Points")) {
			MountPointsView()
		}
	}

	var ResourcesSection: some View {
		Section(header: Text("📌 Resources")) {
			HStack {
				Image(systemName: "ant")
				Button("Report an issue") {
					UIApplication.shared.open(NSURL(
						string: "https://github.com/aspenluxxxy/xenon/issues/new?assignees=&labels=bug&template=bug_report.md&title="
					)! as URL)
				}
			}
			HStack {
				Image(systemName: "doc.on.clipboard")
				Button("Copy log to clipboard") {
					guard let log_text = try? String(
						contentsOf: URL(fileURLWithPath: "/var/mobile/Library/me.aspenuwu.xenon/daemon.log"), encoding: .utf8
					) else { return }
					UIPasteboard.general.string = log_text
					success_alert = true
				}
			}
		}
	}

	var CreditsSection: some View {
		Section(header: Text("💖 Credits"), footer: Text("© Aspen 2021")) {
			HStack {
				Image(contentsOfFile: "/Library/PreferenceBundles/XenonPrefs.bundle/aspen.png")
				Button("Tweak by Aspen") {
					UIApplication.shared.open(
						URL(string: "https://twitter.com/aspenluxxxy")!,
						options: .init(),
						completionHandler: .none
					)
				}
			}.padding([.top, .bottom, .trailing])
			HStack {
				Image(contentsOfFile: "/Library/PreferenceBundles/XenonPrefs.bundle/alpha.png")
				Button("Logo by Alpha") {
					UIApplication.shared.open(
						URL(string: "https://twitter.com/Kutarin_")!,
						options: .init(),
						completionHandler: .none
					)
				}
			}
			.padding([.top, .bottom, .trailing])
			Button("Licenses") {
				active_sheet = .licenses
			}
		}
	}

	var body: some View {
		ZStack {
			if success_alert {
				SuccessPopupView(success_view: $success_alert.animation(.easeOut)).zIndex(9999.0)
			}
			VStack {
				Header(
					"Xenon",
					icon: (Image(contentsOfFile: "/Library/PreferenceBundles/XenonPrefs.bundle/icon@3x.png") ??
						Image(systemName: "folder")).resizable().frame(width: 64, height: 64),
					subtitle: "The easiest way to access your device's files from your own computer!"
				)
				.background(
					LinearGradient(gradient: Gradient(colors: [Color.pink, Color.purple]), startPoint: .top,
					               endPoint: .bottom)
						.edgesIgnoringSafeArea(.all)
						.frame(width: UIScreen.main.bounds.width, height: UIScreen.main.bounds.height * 0.5)
						.offset(y: UIScreen.main.bounds.height * -0.17)
				)
				Form {
					HStack {
						Image(systemName: "desktopcomputer")
						Button("Pair Computer") {
							self.active_sheet = .qr
						}
					}
					EncryptionSection
					MountPointSection
					ResourcesSection
					CreditsSection
				}
			}
		}
		.alert(isPresented: $daemon_alert) {
			Alert(title: Text("Failed to communicate with the Xenon daemon"),
			      message: Text("Are you sure that the Xenon daemon is currently running?"),
			      dismissButton: .default(Text("OK")))
		}
		.sheet(item: $active_sheet) { item in
			switch item {
			case .qr:
				CodeScannerView(codeTypes: [.qr], simulatedData: "XR42~nothing") { result in
					switch result {
					case let .success(code):
						active_sheet = .none
						do {
							try DaemonIPC.communicate_with_daemon(message: code)
							self.success_alert = true
						} catch {
							self.daemon_alert = true
						}
					case let .failure(error):
						NSLog("Xenon QR scanning errored: \(error.localizedDescription)")
					}
				}
			case .mount:
				CreateMountView(mounts: $mounts, daemon_alert: $daemon_alert, active_sheet: $active_sheet)
			case .licenses:
				LicensesView()
			}
		}
		.navigationBarTitle("Xenon")
		.environment(\.horizontalSizeClass, .regular)
	}

	func regenerate_keys() {
		do {
			try DaemonIPC.communicate_with_daemon(message: "regenerate-keys") { (reply: String) in
				self.pubkey = reply
				self.success_alert = true
			}
		} catch {
			self.daemon_alert = true
		}
	}

	// Oh god I hate it
	func delete_mount(at offsets: IndexSet) {
		var keys = self.mounts.keys.sorted(by: { $0 < $1 })
		keys.remove(atOffsets: offsets)
		for key in self.mounts.keys {
			if !keys.contains(key) {
				self.mounts[key] = nil
			}
		}
		MountPoint.save_mounts(self.mounts)
	}
}

private extension Image {
	init?(contentsOfFile path: String) {
		guard let image = UIImage(contentsOfFile: path) else { return nil }
		self.init(uiImage: image)
	}
}

#if DEBUG
	struct PreferencesView_Previews: PreviewProvider {
		static var previews: some View {
			PreferencesView()
				.previewLayout(.device)
				.preferredColorScheme(.dark)
				.previewDevice("iPhone 11")
				.padding()
		}
	}
#endif
