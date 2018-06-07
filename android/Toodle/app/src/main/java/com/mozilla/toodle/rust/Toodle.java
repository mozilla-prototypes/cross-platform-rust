/* -*- Mode: Java; c-basic-offset: 4; tab-width: 20; indent-tabs-mode: nil; -*-
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package com.mozilla.toodle.rust;

import android.content.Context;
import android.util.Log;

import com.mozilla.toodle.Item;
import com.sun.jna.Memory;
import com.sun.jna.NativeLong;
import com.sun.jna.Pointer;
import com.sun.jna.ptr.NativeLongByReference;

public class Toodle extends RustObject {
    static {
        System.loadLibrary("toodle_ffi");
    }
    private static final String LOG_TAG = "Toodle";

    private static final String DB_NAME = "toodle.db";
    private static Toodle sharedInstance;

    private Toodle(Context context) {
        this.rawPointer = JNA.INSTANCE.new_toodle(
                context.getDatabasePath(DB_NAME).getAbsolutePath()
        );
    }

    public static Toodle getSharedInstance(Context context) {
        if (sharedInstance == null) {
            sharedInstance = new Toodle(context);
        }
        return sharedInstance;
    }

    public void createItem(Item item) {
        Log.i(LOG_TAG, "sync pointer: " + rawPointer);
        JNA.INSTANCE.toodle_create_item(
                rawPointer,
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
                rawPointer,
                item.uuid(),
                item.name(),
                new NativeLongByReference(new NativeLong(item.dueDate())),
                completionDateRef
        );
    }

    public void getAllItems(NativeItemsCallback callback) {
        JNA.INSTANCE.toodle_all_items(rawPointer, callback);
    }

    public NativeResult sync() {
        Log.i(LOG_TAG, "sync pointer: " + rawPointer);
        return JNA.INSTANCE.toodle_sync(rawPointer, "00000000-0000-0000-0000-000000000998", "http://mentat.dev.lcip.org/mentatsync/0.1");
    }

    public void registerObserver(String key, String[] attributes, NativeTxObserverCallback callback) {
        // turn string array into int array
        long[] attrEntids = new long[attributes.length];
        for(int i = 0; i < attributes.length; i++) {
            attrEntids[i] = JNA.INSTANCE.store_entid_for_attribute(rawPointer, attributes[i]);
        }
        Log.i(LOG_TAG, "Registering observer {" + key + "} for attributes:");
        for (int i = 0; i < attrEntids.length; i++) {
            Log.i(LOG_TAG, "entid: " + attrEntids[i]);
        }
        final Pointer entidsNativeArray = new Memory(8 * attrEntids.length);
        entidsNativeArray.write(0, attrEntids, 0, attrEntids.length);
        JNA.INSTANCE.store_register_observer(rawPointer, key, entidsNativeArray, attrEntids.length, callback);
    }

    public void unregisterObserver(String key) {
        JNA.INSTANCE.store_unregister_observer(rawPointer, key);
    }

    @Override
    public void close() {
        Log.i("Toodle", "close");
        JNA.INSTANCE.toodle_destroy(rawPointer);
    }
}