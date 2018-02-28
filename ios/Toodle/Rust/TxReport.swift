/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import Foundation

class TxReport {
    var raw: UnsafePointer<ExternTxReport>

    required init(raw: UnsafePointer<ExternTxReport>) {
        self.raw = raw
    }

    func intoRaw() -> UnsafePointer<ExternTxReport> {
        return self.raw
    }

//    deinit {
//        item_c_destroy(raw)
//    }
}
