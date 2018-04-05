/* -*- Mode: Java; c-basic-offset: 4; tab-width: 20; indent-tabs-mode: nil; -*-
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package com.mozilla.toodle.rust;

import android.content.Context;
import android.os.Handler;
import android.util.Log;

import com.mozilla.toodle.Item;
import com.mozilla.toodle.ItemsCallback;
import com.sun.jna.Memory;
import com.sun.jna.NativeLong;
import com.sun.jna.Pointer;
import com.sun.jna.ptr.NativeLongByReference;

import java.io.Closeable;
import java.util.ArrayList;
import java.util.LinkedList;
import java.util.List;

public class Toodle extends Store {
    static {
        System.loadLibrary("toodle");
    }
    private static final String LOG_TAG = "Toodle";

    private static final String DB_NAME = "toodle.db";
    private static Toodle sharedInstance;

    private Toodle(Context context) {
        super();
        rawPointer = JNA.INSTANCE.new_toodle(context.getDatabasePath(DB_NAME).getAbsolutePath());
    }

    public static Toodle getSharedInstance(Context context) {
        if (sharedInstance == null) {
            sharedInstance = new Toodle(context);
        }
        return sharedInstance;
    }

    public void createItem(Item item) {
        Log.i(LOG_TAG, "sync pointer: " + this.rawPointer);
        JNA.INSTANCE.toodle_create_item(
                this.rawPointer,
                item.name(),
                new NativeLongByReference(new NativeLong(item.dueDate()))
        );
    }

    public void updateItem(Item item) {
        final NativeLongByReference completionDateRef;
        if (item.completionDate() != null) {
            completionDateRef = new NativeLongByReference(new NativeLong(item.completionDate()));
        } else {
            completionDateRef = null;
        }

        JNA.INSTANCE.toodle_update_item_by_uuid(
                this.rawPointer,
                item.uuid().toString(),
                item.name(),
                new NativeLongByReference(new NativeLong(item.dueDate())),
                completionDateRef
        );
    }

    public void getAllItems(final ItemsCallback callback) {
        String allItemsSQL = "[:find ?eid ?uuid ?name " +
                             ":where "+
                             "[?eid :todo/uuid ?uuid] "+
                             "[?eid :todo/name ?name]]";
        Query query = this.query(allItemsSQL);
        query.execute(new QueryResultRowsHandler() {
            @Override
            public void handleRows(ResultRows rows) {
                ArrayList<Item> itemsList = new ArrayList<Item>();
                for(ResultRow row: rows) {
                    Item item = new Item(row.asEntid(0), row.asUUID(1), row.asString(2));
                    itemsList.add(item);
                }

                callback.items(itemsList);
            }
        });
    }
}

