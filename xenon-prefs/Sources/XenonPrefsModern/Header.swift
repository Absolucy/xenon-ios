//
//  Header.swift
//  NomaePreferences
//
//  Created by Eamon Tracey.
//

import SwiftUI

/// A header view to display a package name,
/// an optional package icon, and an optional subtitle
public struct Header<Icon: View>: View {
	let packageName: String
	let icon: Icon?
	let subtitle: String?

	public init(_ packageName: String, icon: Icon?, subtitle: String? = nil) {
		self.packageName = packageName
		self.icon = icon
		self.subtitle = subtitle
	}

	public var body: some View {
		HStack {
			icon
				.padding(.trailing)
			VStack(alignment: .leading) {
				Text(packageName)
					.font(.largeTitle)
				subtitle.map { subtitle in
					Text(subtitle)
						.font(.callout)
						.foregroundColor(.white)
				}
			}
		}
		.padding()
	}
}
