//
//  CodeScannerView.swift
//
//  Created by Paul Hudson on 10/12/2019.
//  Copyright © 2019 Paul Hudson. All rights reserved.
//

import AVFoundation
import SwiftUI

/// A SwiftUI view that is able to scan barcodes, QR codes, and more, and send back what was found.
/// To use, set `codeTypes` to be an array of things to scan for, e.g. `[.qr]`, and set `completion` to
/// a closure that will be called when scanning has finished. This will be sent the string that was detected or a `ScanError`.
/// For testing inside the simulator, set the `simulatedData` property to some test data you want to send back.
public struct CodeScannerView: UIViewControllerRepresentable {
	public enum ScanError: Error {
		case badInput, badOutput
	}

	public enum ScanMode {
		case once, oncePerCode, continuous
	}

	public class ScannerCoordinator: NSObject, AVCaptureMetadataOutputObjectsDelegate {
		var parent: CodeScannerView
		var codesFound: Set<String>
		var isFinishScanning = false
		var lastTime = Date(timeIntervalSince1970: 0)

		init(parent: CodeScannerView) {
			self.parent = parent
			self.codesFound = Set<String>()
		}

		public func metadataOutput(
			_: AVCaptureMetadataOutput,
			didOutput metadataObjects: [AVMetadataObject],
			from _: AVCaptureConnection
		) {
			if let metadataObject = metadataObjects.first {
				guard let readableObject = metadataObject as? AVMetadataMachineReadableCodeObject else { return }
				guard let stringValue = readableObject.stringValue else { return }
				guard self.isFinishScanning == false else { return }

				switch self.parent.scanMode {
				case .once:
					self.found(code: stringValue)
					// make sure we only trigger scan once per use
					self.isFinishScanning = true
				case .oncePerCode:
					if !self.codesFound.contains(stringValue) {
						self.codesFound.insert(stringValue)
						self.found(code: stringValue)
					}
				case .continuous:
					if self.isPastScanInterval() {
						self.found(code: stringValue)
					}
				}
			}
		}

		func isPastScanInterval() -> Bool {
			Date().timeIntervalSince(self.lastTime) >= self.parent.scanInterval
		}

		func found(code: String) {
			self.lastTime = Date()
			AudioServicesPlaySystemSound(SystemSoundID(kSystemSoundID_Vibrate))
			self.parent.completion(.success(code))
		}

		func didFail(reason: ScanError) {
			self.parent.completion(.failure(reason))
		}
	}

	#if targetEnvironment(simulator)
		public class ScannerViewController: UIViewController, UIImagePickerControllerDelegate,
			UINavigationControllerDelegate
		{
			var delegate: ScannerCoordinator?
			override public func loadView() {
				view = UIView()
				view.isUserInteractionEnabled = true
				let label = UILabel()
				label.translatesAutoresizingMaskIntoConstraints = false
				label.numberOfLines = 0

				label
					.text =
					"You're running in the simulator, which means the camera isn't available. Tap anywhere to send back some simulated data."
				label.textAlignment = .center
				let button = UIButton()
				button.translatesAutoresizingMaskIntoConstraints = false
				button.setTitle("Or tap here to select a custom image", for: .normal)
				button.setTitleColor(UIColor.systemBlue, for: .normal)
				button.setTitleColor(UIColor.gray, for: .highlighted)
				button.addTarget(self, action: #selector(self.openGallery), for: .touchUpInside)

				let stackView = UIStackView()
				stackView.translatesAutoresizingMaskIntoConstraints = false
				stackView.axis = .vertical
				stackView.spacing = 50
				stackView.addArrangedSubview(label)
				stackView.addArrangedSubview(button)

				view.addSubview(stackView)

				NSLayoutConstraint.activate([
					button.heightAnchor.constraint(equalToConstant: 50),
					stackView.leadingAnchor.constraint(equalTo: view.layoutMarginsGuide.leadingAnchor),
					stackView.trailingAnchor.constraint(equalTo: view.layoutMarginsGuide.trailingAnchor),
					stackView.centerYAnchor.constraint(equalTo: view.centerYAnchor),
				])
			}

			override public func touchesBegan(_: Set<UITouch>, with _: UIEvent?) {
				guard let simulatedData = delegate?.parent.simulatedData else {
					print("Simulated Data Not Provided!")
					return
				}

				self.delegate?.found(code: simulatedData)
			}

			@objc func openGallery(_: UIButton) {
				let imagePicker = UIImagePickerController()
				imagePicker.delegate = self
				self.present(imagePicker, animated: true, completion: nil)
			}

			public func imagePickerController(
				_: UIImagePickerController,
				didFinishPickingMediaWithInfo info: [UIImagePickerController.InfoKey: Any]
			) {
				if let qrcodeImg = info[.originalImage] as? UIImage {
					let detector = CIDetector(
						ofType: CIDetectorTypeQRCode,
						context: nil,
						options: [CIDetectorAccuracy: CIDetectorAccuracyHigh]
					)!
					let ciImage = CIImage(image: qrcodeImg)!
					var qrCodeLink = ""

					let features = detector.features(in: ciImage)
					for feature in features as! [CIQRCodeFeature] {
						qrCodeLink += feature.messageString!
					}

					if qrCodeLink == "" {
						self.delegate?.didFail(reason: .badOutput)
					} else {
						self.delegate?.found(code: qrCodeLink)
					}
				} else {
					print("Something went wrong")
				}
				self.dismiss(animated: true, completion: nil)
			}
		}
	#else
		public class ScannerViewController: UIViewController {
			var captureSession: AVCaptureSession!
			var previewLayer: AVCaptureVideoPreviewLayer!
			var delegate: ScannerCoordinator?

			override public func viewDidLoad() {
				super.viewDidLoad()

				NotificationCenter.default.addObserver(self,
				                                       selector: #selector(self.updateOrientation),
				                                       name: Notification.Name("UIDeviceOrientationDidChangeNotification"),
				                                       object: nil)

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
					self.delegate?.didFail(reason: .badInput)
					return
				}

				let metadataOutput = AVCaptureMetadataOutput()

				if self.captureSession.canAddOutput(metadataOutput) {
					self.captureSession.addOutput(metadataOutput)

					metadataOutput.setMetadataObjectsDelegate(self.delegate, queue: DispatchQueue.main)
					metadataOutput.metadataObjectTypes = self.delegate?.parent.codeTypes
				} else {
					self.delegate?.didFail(reason: .badOutput)
					return
				}
			}

			override public func viewWillLayoutSubviews() {
				self.previewLayer?.frame = view.layer.bounds
			}

			@objc func updateOrientation() {
				guard let orientation = UIApplication.shared.windows.first?.windowScene?.interfaceOrientation
				else { return }
				guard let connection = captureSession.connections.last,
				      connection.isVideoOrientationSupported else { return }
				connection.videoOrientation = AVCaptureVideoOrientation(rawValue: orientation.rawValue) ?? .portrait
			}

			override public func viewDidAppear(_ animated: Bool) {
				super.viewDidAppear(animated)
				self.updateOrientation()
			}

			override public func viewWillAppear(_ animated: Bool) {
				super.viewWillAppear(animated)

				if self.previewLayer == nil {
					self.previewLayer = AVCaptureVideoPreviewLayer(session: self.captureSession)
				}
				self.previewLayer.frame = view.layer.bounds
				self.previewLayer.videoGravity = .resizeAspectFill
				view.layer.addSublayer(self.previewLayer)

				if self.captureSession?.isRunning == false {
					self.captureSession.startRunning()
				}
			}

			override public func viewDidDisappear(_ animated: Bool) {
				super.viewDidDisappear(animated)

				if self.captureSession?.isRunning == true {
					self.captureSession.stopRunning()
				}

				NotificationCenter.default.removeObserver(self)
			}

			override public var prefersStatusBarHidden: Bool {
				true
			}

			override public var supportedInterfaceOrientations: UIInterfaceOrientationMask {
				.all
			}
		}
	#endif

	public let codeTypes: [AVMetadataObject.ObjectType]
	public let scanMode: ScanMode
	public let scanInterval: Double
	public var simulatedData = ""
	public var completion: (Result<String, ScanError>) -> Void

	public init(
		codeTypes: [AVMetadataObject.ObjectType],
		scanMode: ScanMode = .once,
		scanInterval: Double = 2.0,
		simulatedData: String = "",
		completion: @escaping (Result<String, ScanError>) -> Void
	) {
		self.codeTypes = codeTypes
		self.scanMode = scanMode
		self.scanInterval = scanInterval
		self.simulatedData = simulatedData
		self.completion = completion
	}

	public func makeCoordinator() -> ScannerCoordinator {
		ScannerCoordinator(parent: self)
	}

	public func makeUIViewController(context: Context) -> ScannerViewController {
		let viewController = ScannerViewController()
		viewController.delegate = context.coordinator
		return viewController
	}

	public func updateUIViewController(_: ScannerViewController, context _: Context) {}
}
