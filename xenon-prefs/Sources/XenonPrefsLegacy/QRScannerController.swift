import AVFoundation
import UIKit

class QRScannerController: UIViewController, AVCaptureMetadataOutputObjectsDelegate {
	var captureSession: AVCaptureSession!
	var previewLayer: AVCaptureVideoPreviewLayer!
	var handler: ((String) -> Void)!

	override func viewDidLoad() {
		super.viewDidLoad()

		view.backgroundColor = UIColor.black
		self.captureSession = AVCaptureSession()

		guard let videoCaptureDevice = AVCaptureDevice.default(for: .video) else { return }
		let videoInput: AVCaptureDeviceInput

		do {
			videoInput = try AVCaptureDeviceInput(device: videoCaptureDevice)
		} catch {
			return
		}

		if self.captureSession.canAddInput(videoInput) {
			self.captureSession.addInput(videoInput)
		} else {
			self.failed()
			return
		}

		let metadataOutput = AVCaptureMetadataOutput()

		if self.captureSession.canAddOutput(metadataOutput) {
			self.captureSession.addOutput(metadataOutput)

			metadataOutput.setMetadataObjectsDelegate(self, queue: DispatchQueue.main)
			metadataOutput.metadataObjectTypes = [.qr]
		} else {
			self.failed()
			return
		}

		self.previewLayer = AVCaptureVideoPreviewLayer(session: self.captureSession)
		self.previewLayer.frame = view.layer.bounds
		self.previewLayer.videoGravity = .resizeAspectFill
		view.layer.addSublayer(self.previewLayer)

		self.captureSession.startRunning()
	}

	func failed() {
		let ac = UIAlertController(
			title: "Scanning not supported",
			message: "Your device does not support scanning a code from an item. Please use a device with a camera.",
			preferredStyle: .alert
		)
		ac.addAction(UIAlertAction(title: "OK", style: .default))
		present(ac, animated: true)
	}

	override func viewWillAppear(_ animated: Bool) {
		super.viewWillAppear(animated)

		if self.captureSession?.isRunning == false {
			self.captureSession.startRunning()
		}
	}

	override func viewWillDisappear(_ animated: Bool) {
		super.viewWillDisappear(animated)

		if self.captureSession?.isRunning == true {
			self.captureSession.stopRunning()
		}
	}

	func metadataOutput(
		_: AVCaptureMetadataOutput, didOutput metadataObjects: [AVMetadataObject],
		from _: AVCaptureConnection
	) {
		self.captureSession.stopRunning()

		if let metadataObject = metadataObjects.first {
			guard let readableObject = metadataObject as? AVMetadataMachineReadableCodeObject else { return }
			guard let stringValue = readableObject.stringValue else { return }
			AudioServicesPlaySystemSound(SystemSoundID(kSystemSoundID_Vibrate))
			if stringValue.starts(with: "XE42~") {
				self.handler(stringValue)
			} else {
				let ac = UIAlertController(
					title: "Invalid QR Code",
					message:
					"QR code was not a valid Xenon pairing code!",
					preferredStyle: .alert
				)
				ac.addAction(UIAlertAction(title: "OK", style: .default))
				present(ac, animated: true)
			}
			dismiss(animated: true)
		}
	}

	override var prefersStatusBarHidden: Bool {
		true
	}

	override var supportedInterfaceOrientations: UIInterfaceOrientationMask {
		.portrait
	}
}
