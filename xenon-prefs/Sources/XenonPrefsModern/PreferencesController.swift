import SwiftUI
import UIKit

extension Bundle: ObservableObject {}

@objcMembers open class PreferenceLoaderController: UIViewController {
	func setRootController(_: Any?) {}
	func setParentController(_: Any?) {}
	func setSpecifier(_: Any?) {}
}

open class PreferencesController: PreferenceLoaderController {
	override public func loadView() {
		let host = UIHostingController(rootView: PreferencesView())
		let tmp = host.view
		host.view = nil
		view = tmp
	}
}
