import Foundation

public struct DependencyLicensingInfo: Decodable {
	public var name: String
	public var version: String
	public var authors: String
	public var description: String?
	public var repository: String?
	public var license: Licenses?
}

public struct Licenses {
	public var licenses: Set<String>
}

extension Licenses: Decodable {
	public init(from decoder: Decoder) {
		self.licenses = Set()

		do {
			let container = try decoder.singleValueContainer()
			let value = try container.decode(String.self)
			let split = value.components(separatedBy: " OR ")
			for license in split {
				let license_name = license.trimmingCharacters(in: .whitespacesAndNewlines)
				#if targetEnvironment(simulator)
					self.licenses.insert(license_name)
				#else
					let path = URL(fileURLWithPath: "/Library/PreferenceBundles/XenonPrefs.bundle/Licenses/")
						.appendingPathComponent(license_name + ".txt")
					if FileManager.default.fileExists(atPath: path.path) {
						self.licenses.insert(license_name)
					}
				#endif
			}
		} catch {}
	}

	public static func get() -> [DependencyLicensingInfo] {
		#if targetEnvironment(simulator)
			let json = """
			[{"name":"adler","version":"0.2.3","authors":"Jonas Schievink <jonasschievink@gmail.com>","repository":"https://github.com/jonas-schievink/adler.git","license":"0BSD OR Apache-2.0 OR MIT","license_file":null,"description":"A simple clean-room implementation of the Adler-32 checksum"},{"name":"adler32","version":"1.2.0","authors":"Remi Rampin <remirampin@gmail.com>","repository":"https://github.com/remram44/adler32-rs","license":"Zlib","license_file":null,"description":"Minimal Adler32 implementation for Rust."},{"name":"aead","version":"0.3.2","authors":"RustCrypto Developers","repository":"https://github.com/RustCrypto/traits","license":"Apache-2.0 OR MIT","license_file":null,"description":"Traits for Authenticated Encryption with Associated Data (AEAD) algorithms"}]
			""".data(using: .utf8)!
			return try! JSONDecoder().decode([DependencyLicensingInfo].self, from: json)
		#else
			let license_info_path = URL(fileURLWithPath: "/Library/PreferenceBundles/XenonPrefs.bundle/LicenseInfo.json")
			if let data = try? Data(
				contentsOf: license_info_path,
				options: .mappedIfSafe
			) {
				return
					(try? JSONDecoder().decode([DependencyLicensingInfo].self, from: data)) ?? [DependencyLicensingInfo]()
			} else {
				return [DependencyLicensingInfo]()
			}
		#endif
	}
}
