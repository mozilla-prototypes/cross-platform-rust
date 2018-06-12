///* This Source Code Form is subject to the terms of the Mozilla Public
// * License, v. 2.0. If a copy of the MPL was not distributed with this
// * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import Foundation
import UIKit

import Mentat

class ToodleLib: Mentat {

    class var sharedInstance: ToodleLib {
        struct Static {
            static let instance: ToodleLib = ToodleLib()
        }
        return Static.instance
    }
    
    convenience init() {
        let paths = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask)
        let documentsURL = paths[0]
        let storeURI = documentsURL.appendingPathComponent("todolist.db", isDirectory: false).absoluteString
        
        self.init(raw: new_toodle(storeURI))
    }

    fileprivate func toPointerArray(list: [RustObject]) -> OpaquePointer {
        var pointerArray = list.map({ $0.getRaw() })
        return OpaquePointer(AutoreleasingUnsafeMutablePointer<[OpaquePointer]>(&pointerArray))
    }

    func allItems() -> [Item] {
        let items = toodle_get_all_items(self.getRaw())
        var allItems: [Item] = []
        for index in 0..<item_list_count(items) {
            let item = Item(raw: item_list_entry_at(items, Int(index))!)
            allItems.append(item)
        }
        return allItems
    }

    func createLabel(withName name: String, color: UIColor) -> Label {
        return Label(raw: toodle_create_label(self.getRaw(), name, color.toHex()!))
    }

    func createItem(withName name: String, dueDate: Date?, completionDate: Date?, labels: [Label]) -> Item? {
        var dd: UnsafeMutablePointer<Int64>? = nil
        if let due = dueDate {
            var d = due.asInt64Timestamp()
            dd = UnsafeMutablePointer<Int64>(&d)
        }

        if let item_raw = toodle_create_item(self.getRaw(), name, dd) {
            return Item(raw: item_raw)
        }

        return nil
    }

    func item(withUuid uuid: String) -> Item? {
        guard let new_item = toodle_item_for_uuid(self.getRaw(), uuid) else {
            return nil
        }
        return Item(raw: new_item)
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
        if let uuid = item.uuid {
            toodle_update_item_by_uuid(self.getRaw(), uuid, name, dd, cd)
        } else {
            toodle_update_item(self.getRaw(),
                               item.raw,
                               name,
                               dd,
                               cd,
                               UnsafeMutablePointer<OpaquePointer>(&pointerArray))
        }
    }

    func sync_now() -> Bool {
        let err = toodle_sync(self.getRaw(), "00000000-0000-0000-0000-000000000117", "http://mentat.dev.lcip.org/mentatsync/0.1")
<<<<<<< HEAD
<<<<<<< HEAD
        if let error = err.pointee.err {
=======
        if let error = err.err {
>>>>>>> b0b1d3c... Update Toodle iOS to work with current Mentat, including using Carthage to import Mental SDK
=======
        if let error = err.err {
>>>>>>> 706acdf64fb1ef2a5a7f73f58d1ea1b20f5e08a1
            let str = String(cString: error)
            print("Sync error \(str)")
            return false
        }

        return true
    }
}

class Singleton {
}
