/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import UIKit

class ToDoListItemsTableViewController: UITableViewController {

    lazy var syncToRefresh: UIRefreshControl = {
        let refreshControl = UIRefreshControl()
        refreshControl.addTarget(self, action: #selector(sync), for: UIControlEvents.valueChanged)
        refreshControl.tintColor = UIColor.red
        return refreshControl
    }()

    var items = [Item]()

    override func viewDidLoad() {
        super.viewDidLoad()

        self.tableView.addSubview(self.syncToRefresh)

        ToodleLib.allItems(completion: { (items) in
            DispatchQueue.main.async {
                self.items = items
                self.tableView.reloadData()
            }
        })
        let attrs = [":todo/uuid", ":todo/name", ":todo/due_date", ":todo/completion_date"]
        ToodleLib.sharedInstance.register(key: "ToDoListItemsTableViewController", observer: self, attributes: attrs)

        self.title = "All Items"
        self.navigationItem.rightBarButtonItem = UIBarButtonItem(barButtonSystemItem: UIBarButtonSystemItem.add, target: self, action: #selector(newItem))
    }

    deinit {
        ToodleLib.sharedInstance.unregister(key: "ToDoListItemsTableViewController")
    }

    override func didReceiveMemoryWarning() {
        super.didReceiveMemoryWarning()
    }

    // MARK: - Table view data source

    override func numberOfSections(in tableView: UITableView) -> Int {
        return 1
    }

    override func tableView(_ tableView: UITableView, numberOfRowsInSection section: Int) -> Int {
        return self.items.count
    }


    override func tableView(_ tableView: UITableView, cellForRowAt indexPath: IndexPath) -> UITableViewCell {
        let cell = tableView.dequeueReusableCell(withIdentifier: "ItemCell") ?? UITableViewCell(style: .subtitle, reuseIdentifier: "ItemCell")
        let item = self.items[indexPath.row]
        cell.textLabel?.text = item.name
        if let completionDateString = item.completionDateAsString() {
            cell.detailTextLabel?.text = "Completed: \(completionDateString)"
        } else if let dueDateString = item.dueDateAsString() {
            cell.detailTextLabel?.text = "Due: \(dueDateString)"
        } else {
            cell.detailTextLabel?.text = ""
        }

        return cell
    }

    override func tableView(_ tableView: UITableView, didSelectRowAt indexPath: IndexPath) {
        let item = self.items[indexPath.row]
        let itemVC = ItemViewController(item: item)
        self.navigationController?.pushViewController(itemVC, animated: true)
    }

    @objc fileprivate func newItem() {
        let itemVC = ItemViewController()
        let navController = UINavigationController(rootViewController: itemVC)
        self.present(navController, animated: true, completion: nil)
    }

    @objc func sync() {
        let success = ToodleLib.sharedInstance.sync_now()
        if success {
            print("Sync succeeded")
            self.syncToRefresh.endRefreshing()
        } else {
            print("Sync failed")
            self.syncToRefresh.endRefreshing()
        }
    }

}

extension ToDoListItemsTableViewController: Observing {
    func transactionDidOccur(key: String, reports: [TxReport]) {
        print("transaction did occur \(key)")
        ToodleLib.allItems(completion: { (items) in
            DispatchQueue.main.async {
                self.items = items
                self.tableView.reloadData()
            }
        })
    }
}
