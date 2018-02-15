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
        let items = toodle_get_all_items(self.raw)
        var allItems: [Item] = []
        for index in 0..<item_list_count(items) {
            let item = Item(raw: item_list_entry_at(items, Int(index))!)
            allItems.append(item)
        }
        return allItems
    }

    func createItem(withName name: String) -> Item? {
        return Item(raw: toodle_create_item(self.raw,
                                            name)!)
    }

    func item(withUuid uuid: String) -> Item? {
        guard let new_item = toodle_item_for_uuid(self.raw, uuid) else {
            return nil
        }
        return Item(raw: new_item)
    }

    func update(item: Item, name: String, completionDate: Date?) {
        var cd: AutoreleasingUnsafeMutablePointer<Int64>? = nil
        if let completion = completionDate {
            var c = completion.asInt64Timestamp()
            cd = AutoreleasingUnsafeMutablePointer<Int64>(&c)
        }
        if let uuid = item.uuid {
            toodle_update_item_by_uuid(self.raw, uuid, name, cd)
        } else {
            toodle_update_item(self.raw,
                               item.raw,
                               name,
                               cd)
        }
    }
}

class Singleton {
}
