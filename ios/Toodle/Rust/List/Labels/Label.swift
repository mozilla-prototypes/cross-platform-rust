/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import Foundation
import UIKit

import Mentat

class Label: RustObject {

    var name: String {
        return String(cString: label_get_name(self.getRaw()))
    }

    var color: UIColor {
        get {
            return UIColor(hex: String(cString: label_get_color(self.getRaw()))) ?? UIColor.gray
        }
        set {
            if let hex = newValue.toHex() {
                label_set_color(self.getRaw(), hex)
            }
        }
    }
    
    override func cleanup(pointer: OpaquePointer) {
        label_destroy(self.getRaw())
    }
}
