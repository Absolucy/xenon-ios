import Foundation

public enum DaemonIPC {
	public static func communicate_with_daemon(message: String, handler: (Data) -> Void) throws {
		let socket = try Socket.create(family: .unix, type: .stream, proto: .unix)
		try socket.setWriteTimeout(value: 1500)
		try socket.setReadTimeout(value: 1500)
		try socket.connect(to: "/tmp/me.aspenuwu.xenon.sock")
		try socket.write(from: message + "\0")
		var reply = Data()
		_ = try socket.read(into: &reply)
		socket.close()
		handler(reply)
	}

	public static func communicate_with_daemon(message: String, handler: (String) -> Void) throws {
		try self.communicate_with_daemon(message: message) { (data: Data) in
			handler(String(decoding: data, as: UTF8.self))
		}
	}

	public static func communicate_with_daemon(message: String) throws {
		try self.communicate_with_daemon(message: message) { (_: Data) in }
	}
}
