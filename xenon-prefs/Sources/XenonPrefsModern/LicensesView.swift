//
//  SwiftUIView.swift
//
//
//  Created by Aspen on 3/10/21.
//

import SwiftUI
import XenonPrefs

struct LicensesView: View {
	@State private var license: String?

	@ViewBuilder func DependencyName(_ name: String) -> some View {
		HStack {
			Spacer()
			Text(name).multilineTextAlignment(.center)
			Spacer()
		}
	}

	@ViewBuilder func DependencyLicenses(_ licenses: Set<String>) -> some View {
		let licenses = licenses.sorted { $0 < $1 }
		HStack {
			ForEach(licenses, id: \.self) { license in
				Button(license) {
					do {
						self.license = try String(
							contentsOf: URL(fileURLWithPath: "/Library/PreferenceBundles/XenonPrefs.bundle/Licenses/")
								.appendingPathComponent(license + ".txt"), encoding: .utf8
						)
					} catch {
						UIApplication.shared.open(
							URL(string: "https://opensource.org/licenses/" + license)!, options: .init(),
							completionHandler: .none
						)
					}
				}.buttonStyle(BorderlessButtonStyle())
				if license != licenses.last {
					Divider()
				}
			}
		}
	}

	@ViewBuilder func DependencyAuthors(_ authors: String) -> some View {
		let authors = authors.components(separatedBy: "|").sorted { $0 < $1 }
		HStack {
			ForEach(authors, id: \String.self) { (author: String) in
				Text(author).font(.caption).multilineTextAlignment(.center)
				if author != authors.last {
					Divider()
				}
			}
		}
	}

	var body: some View {
		Form {
			ForEach(Licenses.get().sorted { $0.name < $1.name }, id: \.name) { (dependency: DependencyLicensingInfo) in
				dependency.license.map { licensing in
					VStack {
						DependencyName(dependency.name)
						DependencyLicenses(licensing.licenses).padding(5.0)
						DependencyAuthors(dependency.authors)
					}.padding()
				}
			}
		}.sheet(item: $license) { license in
			Text(license).padding()
		}
	}
}

extension String: Identifiable {
	public var id: String { self }
}

struct LicensesView_Previews: PreviewProvider {
	static var previews: some View {
		LicensesView()
			.preferredColorScheme(.dark)
	}
}
