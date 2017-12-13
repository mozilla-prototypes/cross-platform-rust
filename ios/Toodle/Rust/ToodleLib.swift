///* This Source Code Form is subject to the terms of the Mozilla Public
// * License, v. 2.0. If a copy of the MPL was not distributed with this
// * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import Foundation
import UIKit

class ToodleLib: RustObject {

    class var sharedInstance: ToodleLib {
        struct Static {
            static let instance: ToodleLib = ToodleLib()
        }
        return Static.instance
    }

    var raw: OpaquePointer

    required init(raw: OpaquePointer) {
        self.raw = raw
    }

    func intoRaw() -> OpaquePointer {
        return self.raw
    }

    convenience init() {
        let paths = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask)
        let documentsURL = paths[0]
        let storeURI = documentsURL.appendingPathComponent("todolist.db", isDirectory: false).absoluteString
        self.init(raw: new_toodle(storeURI))
    }

        deinit {
            toodle_destroy(raw)
        }

    fileprivate func toPointerArray(list: [RustObject]) -> OpaquePointer {
        var pointerArray = list.map({ $0.intoRaw() })
        return OpaquePointer(AutoreleasingUnsafeMutablePointer<[OpaquePointer]>(&pointerArray))
    }

    func allItems() -> [Item] {
        let items = list_manager_get_all_items(self.raw)
        var allItems: [Item] = []
        for index in 0..<item_list_count(items) {
            let item = Item(raw: item_list_entry_at(items, Int(index))!)
            allItems.append(item)
        }
        return allItems
    }

    //    func allLabels() -> [Label] {
    //        let labels = list_manager_get_all_labels(self.raw)
    //        var allLabels: [Label] = []
    //        for index in 0..<label_list_count(labels) {
    //            let label = Label(raw: label_list_entry_at(labels, index))
    //            allLabels.append(label)
    //        }
    //        return allLabels
    //    }

    func createLabel(withName name: String, color: UIColor) -> Label {
        return Label(raw: list_manager_create_label(self.raw, name, color.toHex()!))
    }

    func createItem(withName name: String, dueDate: Date?, completionDate: Date?, labels: [Label]) -> Item? {
        var dd: UnsafeMutablePointer<Int64>? = nil
        if let due = dueDate {
            var d = due.asInt64Timestamp()
            dd = UnsafeMutablePointer<Int64>(&d)
        }
        var pointerArray = self.toPointerArray(list: labels as [RustObject])
        return Item(raw: list_manager_create_item(self.raw,
                                                  name,
                                                  dd,
                                                  UnsafeMutablePointer<OpaquePointer>(&pointerArray))!)
    }

    func update(item: Item, name: String, dueDate: Date?, completionDate: Date?, labels: [Label]) {
        var dd: AutoreleasingUnsafeMutablePointer<Int64>? = nil
        if let due = dueDate{
            var d = due.asInt64Timestamp()
            dd = AutoreleasingUnsafeMutablePointer<Int64>(&d)
        }
        var cd: AutoreleasingUnsafeMutablePointer<Int64>? = nil
        if let completion = completionDate {
            var c = completion.asInt64Timestamp()
            cd = AutoreleasingUnsafeMutablePointer<Int64>(&c)
        }
        var pointerArray = self.toPointerArray(list: labels as [RustObject])
        list_manager_update_item(raw,
                                 item.raw,
                                 name,
                                 dd,
                                 cd,
                                 UnsafeMutablePointer<OpaquePointer>(&pointerArray))
    }
}

class Singleton {
}
