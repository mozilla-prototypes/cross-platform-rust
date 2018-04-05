/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import Foundation

class Item {
    var raw: OpaquePointer?
    var id: Int64?
    var uuid: UUID?
    var name: String
    var dueDate: Date? {
        get {
            return ToodleLib.sharedInstance.value(forAttribute: ":todo/due_date", onEntity: self.id!)?.asDate()
        }
        set {
            if let date = newValue {
                store_set_timestamp_for_attribute_on_entid(ToodleLib.sharedInstance.intoRaw(), self.id!, ":todo/due_date", date.asInt64Timestamp())
            }
        }
    }
    var completionDate: Date? {
        get {
            return ToodleLib.sharedInstance.value(forAttribute: ":todo/completion_date", onEntity: self.id!)?.asDate()
        }
        set {
            if let date = newValue {
                store_set_timestamp_for_attribute_on_entid(ToodleLib.sharedInstance.intoRaw(), self.id!, ":todo/completion_date", date.asInt64Timestamp())
            }
        }
    }

    fileprivate var _labels: [Label]?

    init(name: String) {
        self.name = name
    }

    init(id: Int64, uuid: UUID, name: String) {
        self.name = name
        self.uuid = uuid
        self.id = id
    }

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
        dateFormatter.dateStyle = .long
        dateFormatter.timeStyle = .short
        return dateFormatter.string(from: dueDate)
    }

    func completionDateAsString() -> String? {
        guard let completionDate = self.completionDate else {
            return nil
        }
        let dateFormatter = DateFormatter()
        dateFormatter.dateStyle = .long
        dateFormatter.timeStyle = .short
        return dateFormatter.string(from: completionDate)
    }
}
