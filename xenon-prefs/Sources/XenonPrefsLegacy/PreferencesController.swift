import Foundation
import Preferences
import XenonPrefs
import XenonPrefsC

class PreferencesController: PSEditableListController {
	var welcome_controller: OBWelcomeController?

	var mounts: [String: MountPoint] {
		get {
			MountPoint.load_mounts()
		}
		set {
			do {
				MountPoint.save_mounts(newValue)
				do {
					try DaemonIPC.communicate_with_daemon(message: "reload-mounts")
				} catch {
					self.daemon_alert()
				}
			} catch {}
		}
	}

	override var specifiers: NSMutableArray? {
		get {
			if let specifiers = value(forKey: "_specifiers") as? NSMutableArray {
				add_specifiers(specifiers)
				return specifiers
			} else {
				let specifiers = loadSpecifiers(fromPlistName: "Root", target: self) as NSMutableArray
				setValue(specifiers, forKey: "_specifiers")
				add_specifiers(specifiers)
				return specifiers
			}
		}
		set {
			super.specifiers = newValue
		}
	}

	override func performDeletionAction(for specifier: PSSpecifier?) -> Bool {
		let orig = super.performDeletionAction(for: specifier)
		if let specifier = specifier,
		   let name = specifier.name
		{
			self.mounts.removeValue(forKey: name)
		}
		return orig
	}

	override func tableView(_: UITableView, editingStyleForRowAt _: IndexPath)
		-> UITableViewCell.EditingStyle
	{
		.delete
	}

	override func editable() -> Bool {
		true
	}

	override func viewDidLoad() {
		super.viewDidLoad()

		let header_view = UIView(frame: CGRect(x: 0, y: 0, width: 200, height: 200))
		let header_img_view = UIImageView(frame: CGRect(x: 0, y: 0, width: 200, height: 200))
		header_img_view.contentMode = .scaleAspectFill
		header_img_view.image = UIImage(contentsOfFile: "/Library/PreferenceBundles/XenonPrefs.bundle/banner.png")
		header_img_view.translatesAutoresizingMaskIntoConstraints = false

		header_view.addSubview(header_img_view)
		NSLayoutConstraint.activate([
			header_img_view.topAnchor.constraint(equalTo: header_view.topAnchor),
			header_img_view.leadingAnchor.constraint(equalTo: header_view.leadingAnchor),
			header_img_view.trailingAnchor.constraint(equalTo: header_view.trailingAnchor),
			header_img_view.bottomAnchor.constraint(equalTo: header_view.bottomAnchor),
		])

		table.tableHeaderView = header_view

		if #available(iOS 13.0, *) {
			open_welcome()
		}
	}

	@available(iOS 13.0, *)
	func open_welcome() {
		self.welcome_controller = OBWelcomeController(
			title: "Xenon",
			detailText: "The easiest way to access your phone's files!",
			icon: UIImage(named: "icon@3x", in: Bundle(for: PreferencesController.self), compatibleWith: nil)
		)
		self.welcome_controller!.addBulletedListItem(
			withTitle: nil,
			description: "Install the Xenon client on your computer",
			image: UIImage(systemName: "plus.app")
		)
		self.welcome_controller!.addBulletedListItem(
			withTitle: nil,
			description: "Select 'Pair Device' in the Xenon client, and scan the QR Code using 'Pair Computer'",
			image: UIImage(systemName: "qrcode.viewfinder")
		)
		self.welcome_controller!.addBulletedListItem(
			withTitle: nil,
			description: "Add mounts from this preferences menu",
			image: UIImage(systemName: "folder.badge.plus")
		)
		self.welcome_controller!.addBulletedListItem(
			withTitle: nil,
			description: "Access your mounts remotely, from your PC!",
			image: UIImage(systemName: "wifi")
		)

		let continue_button = OBBoldTrayButton(type: 1)!
		continue_button.addTarget(self, action: #selector(PreferencesController.close_welcome), for: .touchUpInside)
		continue_button.setTitle("Continue", for: 0)
		continue_button.setTitleColor(UIColor.white, for: .normal)
		continue_button.clipsToBounds = true
		continue_button.layer.cornerRadius = 14
		continue_button.layer.cornerCurve = .continuous
		self.welcome_controller!.buttonTray().addButton(continue_button)

		self.welcome_controller!.buttonTray().effectView.effect = UIBlurEffect(style: .systemChromeMaterial)
		let effect_welcome_view = UIVisualEffectView(frame: welcome_controller!.viewIfLoaded.bounds)
		effect_welcome_view.effect = UIBlurEffect(style: .systemChromeMaterial)
		self.welcome_controller!.viewIfLoaded.insertSubview(effect_welcome_view, at: 0)
		self.welcome_controller!.viewIfLoaded.backgroundColor = UIColor.clear

		self.welcome_controller!.modalPresentationStyle = .pageSheet
		self.welcome_controller!.isModalInPresentation = true
		self.welcome_controller!.view.tintColor = .systemPurple

		present(self.welcome_controller!, animated: true, completion: nil)
	}

	@available(iOS 13.0, *)
	@objc func close_welcome() {
		self.welcome_controller!.dismiss(animated: true, completion: nil)
	}

	func add_specifiers(_ specs: NSMutableArray) {
		for (name, _) in self.mounts.sorted(by: { $0.0 < $1.0 }) {
			let specifier = PSSpecifier.preferenceSpecifierNamed(
				name, target: self, set: nil,
				get: nil, detail: nil,
				cell: PSCellType.staticTextCell, edit: nil
			)!
			specifier.setProperty(name, forKey: "label")
			specifier.setProperty(true, forKey: "enabled")
			specs.insert(specifier, at: 6)
		}
	}

	func get_name(handler: @escaping ((String) -> Void)) {
		let alert_controller = UIAlertController(
			title: "Name", message: "Enter the name of the new mount point", preferredStyle: .alert
		)

		let confirm_action = UIAlertAction(title: "Confirm", style: .default) { _ in
			if let name = alert_controller.textFields?[0].text {
				if name.range(of: "^[A-Za-z0-9_.-]+$", options: .regularExpression, range: nil, locale: nil) == nil ||
					name.caseInsensitiveCompare("xenon") == .orderedSame
				{
					let ac = UIAlertController(
						title: "Invalid Name",
						message:
						"Your mount name contains invalid characters!\nPlease ensure that your mount name only contains alphanumeric characters, dots, dashes, and underscores.",
						preferredStyle: .alert
					)
					ac.addAction(UIAlertAction(title: "OK", style: .default))
					self.present(ac, animated: true)
				} else {
					handler(name)
				}
			}
		}

		let cancel_action = UIAlertAction(title: "Cancel", style: .cancel) { _ in }

		alert_controller.addTextField { textField in
			textField.placeholder = "Name"
		}

		alert_controller.addAction(confirm_action)
		alert_controller.addAction(cancel_action)

		present(alert_controller, animated: true, completion: nil)
	}

	func add_folder() {
		let alert_controller = UIAlertController(
			title: "Folder path", message: "Enter the path to the folder you'd like to mount", preferredStyle: .alert
		)

		let confirm_action = UIAlertAction(title: "Confirm", style: .default) { _ in
			if let path = alert_controller.textFields?[0].text {
				var is_dir: ObjCBool = false
				let exists = FileManager.default.fileExists(atPath: path, isDirectory: &is_dir)
				if exists, is_dir.boolValue {
					self.get_name { name in
						self.mounts[name] = .path(path)
						self.reloadSpecifiers()
					}
				} else {
					let ac = UIAlertController(
						title: "Folder doesn't exist",
						message:
						"The folder you specified does not exist.\nPlease make sure you entered the ABSOLUTE path correctly!",
						preferredStyle: .alert
					)
					ac.addAction(UIAlertAction(title: "OK", style: .default))
					self.present(ac, animated: true)
				}
			}
		}

		let cancel_action = UIAlertAction(title: "Cancel", style: .cancel) { _ in }

		alert_controller.addTextField { textField in
			textField.placeholder = "Absolute Path"
		}

		alert_controller.addAction(confirm_action)
		alert_controller.addAction(cancel_action)

		present(alert_controller, animated: true, completion: nil)
	}

	func add_bundle() {
		let choose_bundle = UIAlertController(title: "Choose Bundle", message: "", preferredStyle: .actionSheet)
		do {
			try DaemonIPC.communicate_with_daemon(message: "bundles") { (reply: Data) in
				let bundles = ((try? JSONDecoder().decode([String].self, from: reply)) ?? [String]()).sorted()
				for bundle in bundles {
					let bundle_action = UIAlertAction(title: bundle, style: .default) { _ in
						self.get_name { name in
							self.mounts[name] = .bundle(bundle)
							self.reloadSpecifiers()
						}
					}
					choose_bundle.addAction(bundle_action)
				}

				let cancel_action = UIAlertAction(title: "Cancel", style: .cancel)
				choose_bundle.addAction(cancel_action)

				self.present(choose_bundle, animated: true, completion: nil)
			}
		} catch {
			self.daemon_alert()
		}
	}

	func add_icloud_bundle() {
		let choose_bundle = UIAlertController(title: "Choose iCloud Bundle", message: "", preferredStyle: .actionSheet)
		do {
			try DaemonIPC.communicate_with_daemon(message: "icloud-bundles") { (reply: Data) in
				let bundles = ((try? JSONDecoder().decode([String].self, from: reply)) ?? [String]()).sorted()
				for bundle in bundles {
					let bundle_action = UIAlertAction(title: bundle, style: .default) { _ in
						self.get_name { name in
							self.mounts[name] = .bundle(bundle)
							self.reloadSpecifiers()
						}
					}
					choose_bundle.addAction(bundle_action)
				}

				let cancel_action = UIAlertAction(title: "Cancel", style: .cancel)
				choose_bundle.addAction(cancel_action)

				self.present(choose_bundle, animated: true, completion: nil)
			}
		} catch {
			self.daemon_alert()
		}
	}

	func add_preset() {
		let choose_preset = UIAlertController(title: "Choose Preset", message: "", preferredStyle: .actionSheet)

		let photos_action = UIAlertAction(title: "On-Device Photos", style: .default) { _ in
			self.get_name { name in
				self.mounts[name] = .preset(.photos)
				self.reloadSpecifiers()
			}
		}
		choose_preset.addAction(photos_action)

		let omi_action = UIAlertAction(title: "'On my iPhone' Files", style: .default) { _ in
			self.get_name { name in
				self.mounts[name] = .preset(.localfiles)
				self.reloadSpecifiers()
			}
		}
		choose_preset.addAction(omi_action)

		let home_action = UIAlertAction(title: "Home (/var/mobile)", style: .default) { _ in
			self.get_name { name in
				self.mounts[name] = .preset(.home)
				self.reloadSpecifiers()
			}
		}
		choose_preset.addAction(home_action)

		let cancel_action = UIAlertAction(title: "Cancel", style: .cancel)
		choose_preset.addAction(cancel_action)

		present(choose_preset, animated: true, completion: nil)
	}

	@objc func add_mount() {
		let choose_type = UIAlertController(title: "Choose Mount Type", message: "", preferredStyle: .actionSheet)

		let path_action = UIAlertAction(title: "Folder Path", style: .default) {
			_ in
			self.add_folder()
		}
		choose_type.addAction(path_action)

		let preset_action = UIAlertAction(title: "Preset", style: .default) {
			_ in
			self.add_preset()
		}
		choose_type.addAction(preset_action)

		let icloud_bundle_action = UIAlertAction(title: "iCloud Bundle", style: .default) {
			_ in
			self.add_icloud_bundle()
		}
		choose_type.addAction(icloud_bundle_action)

		let bundle_action = UIAlertAction(title: "App Bundle", style: .default) {
			_ in
			self.add_bundle()
		}
		choose_type.addAction(bundle_action)

		let cancel_action = UIAlertAction(title: "Cancel", style: .cancel)
		choose_type.addAction(cancel_action)

		present(choose_type, animated: true, completion: nil)
	}

	func daemon_alert() {
		let ac = UIAlertController(
			title: "Failed to communicate with the Xenon daemon",
			message:
			"Are you sure that the Xenon daemon is currently running?",
			preferredStyle: .alert
		)
		ac.addAction(UIAlertAction(title: "OK", style: .default))
		present(ac, animated: true)
	}

	@objc func begin_pair() {
		let sv = QRScannerController()
		sv.handler = { code in
			do {
				try DaemonIPC.communicate_with_daemon(message: code)
			} catch {
				self.daemon_alert()
			}
		}
		present(sv, animated: true, completion: nil)
	}

	@objc func view_pubkey() {
		do {
			try DaemonIPC.communicate_with_daemon(message: "pubkey") { (reply: String) in
				let alert = UIAlertController(
					title: "Public Key", message: "Your public key has been copied to the clipboard.", preferredStyle: .alert
				)
				UIPasteboard.general.string = reply

				let dismiss_action = UIAlertAction(title: "Close", style: .destructive, handler: nil)
				alert.addAction(dismiss_action)

				self.present(alert, animated: true, completion: nil)
			}
		} catch {
			self.daemon_alert()
		}
	}

	@objc func copy_config() {
		do {
			try DaemonIPC.communicate_with_daemon(message: "generate-config") { (reply: String) in
				let alert = UIAlertController(
					title: "Client Configuration",
					message: "Your client configuration has been copied to the clipboard.\nPlease put this in your Xenon client's 'config.toml' file, accessible from the system tray.",
					preferredStyle: .alert
				)
				UIPasteboard.general.string = "[connection]\n" + reply

				let dismiss_action = UIAlertAction(title: "Close", style: .destructive, handler: nil)
				alert.addAction(dismiss_action)

				self.present(alert, animated: true, completion: nil)
			}
		} catch {
			self.daemon_alert()
		}
	}

	@objc func regenerate_keys() {
		let alert_controller = UIAlertController(
			title: "Are you sure?",
			message: "Regenerating keys will require you to re-pair all devices!\nThere is no reason to do this unless your keys have been compromised!",
			preferredStyle: .alert
		)

		let yes_action = UIAlertAction(title: "Regenerate Keys", style: .default) {
			_ in
			do {
				try DaemonIPC.communicate_with_daemon(message: "regenerate-keys")
			} catch {
				self.daemon_alert()
			}
		}
		alert_controller.addAction(yes_action)

		let cancel_action = UIAlertAction(title: "Cancel", style: .cancel)
		alert_controller.addAction(cancel_action)

		present(alert_controller, animated: true, completion: nil)
	}

	@objc func open_twitter() {
		let appURL = NSURL(string: "twitter://user?screen_name=aspenluxxxy")!
		let webURL = NSURL(string: "https://twitter.com/aspenluxxxy")!

		let application = UIApplication.shared

		if application.canOpenURL(appURL as URL) {
			application.open(appURL as URL)
		} else {
			application.open(webURL as URL)
		}
	}

	@objc func open_alpha_twitter() {
		let appURL = NSURL(string: "twitter://user?screen_name=Kutarin_")!
		let webURL = NSURL(string: "https://twitter.com/Kutarin_")!

		let application = UIApplication.shared

		if application.canOpenURL(appURL as URL) {
			application.open(appURL as URL)
		} else {
			application.open(webURL as URL)
		}
	}

	@objc func open_github_docs() {
		let githubURL = NSURL(string: "https://github.com/aspenluxxxy/xenon/wiki")!
		UIApplication.shared.open(githubURL as URL)
	}

	@objc func open_github_issue() {
		let githubURL =
			NSURL(
				string: "https://github.com/aspenluxxxy/xenon/issues/new?assignees=&labels=bug&template=bug_report.md&title="
			)!
		UIApplication.shared.open(githubURL as URL)
	}

	@objc func copy_log() {
		if let log_text = try? String(
			contentsOf: URL(fileURLWithPath: "/var/mobile/Library/me.aspenuwu.xenon/daemon.log"), encoding: .utf8
		) {
			let alert = UIAlertController(
				title: "Log Copied",
				message: "The latest log file has been copied to the clipboard.",
				preferredStyle: .alert
			)
			UIPasteboard.general.string = log_text

			let dismiss_action = UIAlertAction(title: "Close", style: .destructive, handler: nil)
			alert.addAction(dismiss_action)

			present(alert, animated: true, completion: nil)
		}
	}
}
