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
        item_destroy(raw)
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

    var dueDate: Date? {
        get {
            guard let date = raw.pointee.dueDate else {
                return nil
            }
            return Date(timeIntervalSince1970: Double(date.pointee))
        }
        set {
            if let d = newValue {
                let timestamp = d.timeIntervalSince1970
                var date = Int64(timestamp)
                item_set_due_date(UnsafeMutablePointer<CItem>(mutating: raw), AutoreleasingUnsafeMutablePointer<Int64>(&date))
            }
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

    fileprivate var _labels: [Label]?

    var labels: [Label] {
        get {
            if _labels == nil {
                _labels = []
                // TODO: When we get labels in, put this back!
//                let ls = item_get_labels(self.raw)
//                _labels = []
//                for index in 0..<item_labels_count(ls) {
//                    let label = Label(raw: item_label_at(ls, index)!)
//                    _labels?.append(label)
//                }
            }

            return _labels!
        }
        set {
            _labels = nil
        }
    }

    func dueDateAsString() -> String? {
        guard let dueDate = self.dueDate else {
            return nil
        }
        let dateFormatter = DateFormatter()
        dateFormatter.dateFormat = "yyyy-MM-dd'T'HH:mm:ss.SSSZ"
        return dateFormatter.string(from: dueDate)
    }
}
