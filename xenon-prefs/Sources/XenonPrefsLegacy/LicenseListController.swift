import Preferences
import UIKit
import XenonPrefsC

class LicenseListController: PSListController {
	override var specifiers: NSMutableArray? {
		get {
			if let specifiers = value(forKey: "_specifiers") as? NSMutableArray {
				self.add_specifiers(specifiers)
				return specifiers
			} else {
				let specifiers = loadSpecifiers(fromPlistName: "Licenses", target: self)
				setValue(specifiers, forKey: "_specifiers")
				if let specs = try? specifiers! {
					self.add_specifiers(specs)
				}
				return specifiers
			}
		}
		set {
			super.specifiers = newValue
		}
	}

	func add_specifiers(_ specs: NSMutableArray) {
		for dependency in Licenses.get().sorted { $0.name < $1.name } {
			if let license = dependency.license as? Licenses {
				if license.licenses.count > 0 {
					let specifier = PSSpecifier.preferenceSpecifierNamed(
						dependency.name, target: self, set: nil,
						get: nil, detail: nil,
						cell: PSCellType.groupCell, edit: nil
					)!
					specifier.setProperty(dependency.name, forKey: "label")
					specifier.setProperty(true, forKey: "enabled")
					specifier.setProperty(
						dependency.authors.replacingOccurrences(of: "|", with: ", "), forKey: "footerText"
					)
					specs.add(specifier as PSSpecifier?)
					for license_name in license.licenses.sorted { $0 < $1 } {
						let specifier = PSSpecifier.preferenceSpecifierNamed(
							license_name, target: self, set: nil,
							get: nil, detail: nil,
							cell: PSCellType.buttonCell, edit: nil
						)!
						specifier.setProperty(license_name, forKey: "label")
						specifier.setProperty(true, forKey: "enabled")
						specifier.buttonAction = #selector(self.open_license(_:))
						specs.add(specifier as PSSpecifier?)
					}
				}
			}
		}
	}

	@objc func open_license(_ specifier: PSSpecifier?) {
		if let specifier = specifier,
		   let license_name = specifier.name
		{
			do {
				let license_text = try String(
					contentsOf: URL(fileURLWithPath: "/Library/PreferenceBundles/XenonPrefs.bundle/Licenses/")
						.appendingPathComponent(license_name + ".txt"), encoding: .utf8
				)

				let alert = UIAlertController(
					title: license_name + " license", message: license_text, preferredStyle: .alert
				)

				let dismissAction = UIAlertAction(title: "Close", style: .destructive, handler: nil)
				alert.addAction(dismissAction)

				self.present(alert, animated: true, completion: nil)
			} catch {
				UIApplication.shared.open(
					URL(string: "https://opensource.org/licenses/" + license_name)!, options: .init(),
					completionHandler: .none
				)
			}
		}
	}
}
