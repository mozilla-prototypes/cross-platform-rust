/* -*- Mode: Java; c-basic-offset: 4; tab-width: 20; indent-tabs-mode: nil; -*-
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package com.mozilla.toodle.rust;

import android.content.Context;
import android.util.Log;

import com.mozilla.toodle.Item;
import com.mozilla.toodle.ItemsCallback;
import com.sun.jna.NativeLong;
import com.sun.jna.ptr.NativeLongByReference;

import java.util.ArrayList;

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

    public static Toodle getSharedInstance() {
        return sharedInstance;
    }

    public void createItem(Item item) {
        Log.i(LOG_TAG, "sync pointer: " + this.rawPointer);
        JNA.INSTANCE.toodle_create_item(
                this.rawPointer,
                item.name(),
                new NativeLongByReference(new NativeLong(item.dueDate().getTime()))
        );
    }

    public void updateItem(Item item) {
        final NativeLongByReference completionDateRef;
        if (item.completionDate() != null) {
            completionDateRef = new NativeLongByReference(new NativeLong(item.completionDate().getTime()));
        } else {
            completionDateRef = null;
        }

        JNA.INSTANCE.toodle_update_item_by_uuid(
                this.rawPointer,
                item.uuid().toString(),
                item.name(),
                new NativeLongByReference(new NativeLong(item.dueDate().getTime())),
                completionDateRef
        );
    }

    public void getAllItems(final ItemsCallback callback) {
        final String allItemsQuery = "[:find ?eid ?uuid ?name " +
                                      ":where [?eid :todo/uuid ?uuid] "+
                                             "[?eid :todo/name ?name]]";
        final Query query = query(allItemsQuery);
        new Thread(new Runnable() {
            @Override
            public void run() {
                query.execute(new RelResultHandler() {
                    @Override
                    public void handleRows(RelResult rows) {
                        ArrayList<Item> itemsList = new ArrayList<Item>();
                        for(TupleResult row: rows) {
                            Item item = new Item(row.asEntid(0), row.asUUID(1), row.asString(2));
                            itemsList.add(item);
                        }

                        callback.items(itemsList);
                    }
                });
            }
        }).start();

    }
}

