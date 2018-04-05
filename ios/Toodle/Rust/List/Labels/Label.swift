/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import Foundation
import UIKit

class Label: RustObject {

    var name: String {
        return String(cString: label_get_name(self.raw))
    }

    var color: UIColor {
        get {
            return UIColor(hex: String(cString: label_get_color(self.raw))) ?? UIColor.gray
        }
        set {
            if let hex = newValue.toHex() {
                label_set_color(raw, hex)
            }
        }
    }
}
