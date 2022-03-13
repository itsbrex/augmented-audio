//
//  SwiftUIView.swift
//
//
//  Created by Pedro Tacla Yamada on 11/3/2022.
//

import OSCKit
import SwiftUI

struct TracksPanelContentView: View {
    @EnvironmentObject var store: Store

    var body: some View {
        HStack {
            let tracksPanelContentView = HStack(alignment: .top, spacing: 30) {
                HStack(alignment: .center, spacing: 30) {
                    KnobView(label: "Normal")
                    KnobView(label: "Center", value: 0.1).style(.center)
                    KnobView(label: "Other")
                    KnobView().style(.center)
                    KnobView()
                    KnobView().style(.center)
                    KnobView()

                    KnobView(
                        onChanged: { value in
                            store.setParameter(name: "volume", value: Float(value))
                        }
                    )
                }
            }
            .padding(PADDING * 2)
            .frame(maxWidth: .infinity, maxHeight: .infinity)

            tracksPanelContentView
        }.frame(maxHeight: .infinity)
            .foregroundColor(SequencerColors.white)
            .background(SequencerColors.black)
    }
}

struct TracksPanelContentView_Previews: PreviewProvider {
    static var previews: some View {
        TracksPanelContentView().environmentObject(Store(engine: nil))
    }
}
