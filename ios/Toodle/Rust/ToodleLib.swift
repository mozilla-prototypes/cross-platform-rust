///* This Source Code Form is subject to the terms of the Mozilla Public
// * License, v. 2.0. If a copy of the MPL was not distributed with this
// * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import Foundation
import UIKit

class ToodleLib {

    fileprivate func toPointerArray(list: [RustObject]) -> OpaquePointer {
        var pointerArray = list.map({ $0.intoRaw() })
        return OpaquePointer(AutoreleasingUnsafeMutablePointer<[OpaquePointer]>(&pointerArray))
    }

    static var sharedInstance = Store.sharedInstance

    static func allItems(completion: @escaping ([Item]) -> ()) {
        let allItemsSQL = """
                            [:find ?eid ?uuid ?name
                            :where
                            [?eid :todo/uuid ?uuid]
                            [?eid :todo/name ?name]]
                          """
        do {
            try Store.sharedInstance.query(sql: allItemsSQL).execute { (result) in
                switch result {
                    case .success(let rows):
                        guard let rows = rows else {
                            return
                        }
                        let items = rows.compactMap({ (row) -> Item? in
                            guard let uuid = row.asUUID(index: 1) else {
                                return nil
                            }
                            return Item(id: row.asEntid(index: 0), uuid: uuid, name: row.asString(index: 2))
                        })
                        completion(items)
                    case .error(let err): print("Error!: \(err)")
                }
            }
        } catch {
            print("Error Pointer deallocated")
        }
    }

    static func createLabel(withName name: String, color: UIColor) -> Label {
        return Label(raw: toodle_create_label(Store.sharedInstance.intoRaw(), name, color.toHex()!))
    }

    static func createItem(withName name: String, dueDate: Date?, completionDate: Date?, labels: [Label]) {
        var dd: UnsafeMutablePointer<Int64>? = nil
        if let due = dueDate {
            var d = due.asInt64Timestamp()
            dd = UnsafeMutablePointer<Int64>(&d)
        }

        toodle_create_item(Store.sharedInstance.intoRaw(), name, dd)
    }

    static func update(item: Item, name: String, dueDate: Date?, completionDate: Date?, labels: [Label]) {
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
        if let uuid = item.uuid {
            toodle_update_item_by_uuid(Store.sharedInstance.intoRaw(), uuid.uuidString, name, dd, cd)
        }
    }
}
