import AudioToolbox
import SwiftUI

// Basically works like this:
// At first only icon is shown and with animation of 'appear' it pops up (scales from 0 to 1)
// Then after some delay the 'morph' flag gets set too.
// This means, the text appears,
// its opacity animates to 0
// and moves to the right :) all together
struct SuccessPopupView: View {
	@Binding var success_view: Bool

	var body: some View {
		ZStack {
			VStack(alignment: .center) {
				Spacer()
				Image(systemName: "checkmark.circle.fill")
					.foregroundColor(Color.white)
					.font(.title)
					.padding()
					.background(
						RoundedRectangle(cornerRadius: 10)
							.foregroundColor(Color.green.opacity(0.75))
					)
				Spacer()
			}
		}
		.onAppear {
			AudioServicesPlaySystemSound(kSystemSoundID_Vibrate)
			DispatchQueue.main.asyncAfter(deadline: .now() + 1.25) {
				success_view = false
			}
		}
	}
}
