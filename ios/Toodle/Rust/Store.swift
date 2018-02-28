///* This Source Code Form is subject to the terms of the Mozilla Public
// * License, v. 2.0. If a copy of the MPL was not distributed with this
// * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import Foundation
import UIKit

class Store: RustObject {

    var raw: OpaquePointer

    required init(raw: OpaquePointer) {
        self.raw = raw;
    }

    func intoRaw() -> OpaquePointer {
        return self.raw
    }

    func entidForAttribute(attribute: String) -> Int64 {
        return Int64(store_entid_for_attribute(self.raw, attribute))
    }

    deinit {
        print("Destroying store")
        store_destroy(self.raw)
        print("Store destroyed")
    }
}

