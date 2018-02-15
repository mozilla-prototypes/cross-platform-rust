/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import Foundation

class Item {
    var raw: UnsafePointer<CItem>

    required init(raw: UnsafePointer<CItem>) {
        self.raw = raw
    }

    func intoRaw() -> UnsafePointer<CItem> {
        return self.raw
    }

    deinit {
        item_c_destroy(raw)
    }

    var uuid: String? {
        if let uuid = raw.pointee.uuid {
            return String(cString: uuid)
        }
        return nil
    }

    var name: String {
        get {
            return String(cString: raw.pointee.name)
        }
        set {
            item_set_name(UnsafeMutablePointer<CItem>(mutating: raw), newValue)
        }
    }

    var completionDate: Date? {
        get {
            guard let date = raw.pointee.completionDate else {
                return nil
            }
            return Date(timeIntervalSince1970: Double(date.pointee))
        }
        set {
            if let d = newValue {
                let timestamp = d.timeIntervalSince1970
                var date = Int64(timestamp)
                item_set_completion_date(UnsafeMutablePointer<CItem>(mutating: raw), AutoreleasingUnsafeMutablePointer<Int64>(&date))
            }
        }
    }

    func completionDateAsString() -> String? {
        guard let completionDate = self.completionDate else {
            return nil
        }
        let dateFormatter = DateFormatter()
        dateFormatter.dateFormat = "yyyy-MM-dd'T'HH:mm:ss.SSSZ"
        return dateFormatter.string(from: completionDate)
    }
}
