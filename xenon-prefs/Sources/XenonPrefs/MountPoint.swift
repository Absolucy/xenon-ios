import Foundation

public enum MountPoint {
	case path(String)
	case bundle(String)
	case icloudbundle(String)
	case preset(MountPreset)
}

public extension MountPoint {
	private enum CodingKeys: String, CodingKey {
		case path, bundle, icloudbundle, preset
	}

	enum MountPointCodingError: Error {
		case decoding(String)
	}

	static func load_mounts() -> [String: MountPoint] {
		if let data = try? Data(
			contentsOf: URL(fileURLWithPath: "/var/mobile/Library/me.aspenuwu.xenon/mounts.json"),
			options: .mappedIfSafe
		) {
			return
				(try? JSONDecoder().decode([String: MountPoint].self, from: data)) ?? [String: MountPoint]()
		} else {
			return [String: MountPoint]()
		}
	}

	static func save_mounts(_ mounts: [String: MountPoint]) {
		do {
			FileManager.default.createFile(
				atPath: "/var/mobile/Library/me.aspenuwu.xenon/mounts.json",
				contents: try JSONEncoder().encode(mounts)
			)
		} catch {}
	}
}

extension MountPoint: Decodable {
	public init(from decoder: Decoder) throws {
		let values = try decoder.container(keyedBy: CodingKeys.self)
		// Decode path
		if let value = try? values.decode(
			String.self,
			forKey: .path
		) {
			self = .path(value)
			return
		}
		// Decode bundle
		if let value = try? values.decode(
			String.self,
			forKey: .bundle
		) {
			self = .bundle(value)
			return
		}
		// Decode icloudbundle
		if let value = try? values.decode(
			String.self,
			forKey: .icloudbundle
		) {
			self = .icloudbundle(value)
			return
		}

		// Decode preset
		if let value = try? values.decode(
			MountPreset.self,
			forKey: .preset
		) {
			self = .preset(value)
			return
		}

		throw
			MountPointCodingError
			.decoding("Whoops! \(dump(values))")
	}
}

extension MountPoint: Encodable {
	public func encode(to encoder: Encoder) throws {
		var container = encoder.container(keyedBy: CodingKeys.self)
		switch self {
		case let .path(path):
			try container.encode(path, forKey: .path)
		case let .bundle(bundle):
			try container.encode(bundle, forKey: .bundle)
		case let .icloudbundle(icloudbundle):
			try container.encode(icloudbundle, forKey: .icloudbundle)
		case let .preset(preset):
			try container.encode(preset, forKey: .preset)
		}
	}
}

public enum MountPreset: String, Codable {
	case photos, localfiles, home, documents
}
