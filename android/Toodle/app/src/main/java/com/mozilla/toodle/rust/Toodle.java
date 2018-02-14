/* -*- Mode: Java; c-basic-offset: 4; tab-width: 20; indent-tabs-mode: nil; -*-
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package com.mozilla.toodle.rust;

import android.content.Context;
import android.util.Log;

import com.mozilla.toodle.Item;
import com.sun.jna.NativeLong;
import com.sun.jna.ptr.NativeLongByReference;

public class Toodle extends RustObject {
    static {
        System.loadLibrary("toodle");
    }

    private static final String DB_NAME = "toodle.db";

    public Toodle(Context context) {
        this.rawPointer = JNA.INSTANCE.new_toodle(
                context.getDatabasePath(DB_NAME).getAbsolutePath()
        );
    }

    public void createItem(Item item) {
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

    public void registerChangedItemsCallback(NativeItemsChangedCallback callback) {
        JNA.INSTANCE.toodle_on_items_changed(callback);
    }

    public void getAllItems(NativeItemsCallback callback) {
        JNA.INSTANCE.toodle_all_items(rawPointer, callback);
    }

    public NativeResult sync(String username) {
        return JNA.INSTANCE.toodle_sync(rawPointer, username);
    }

    @Override
    public void close() {
        Log.i("Toodle", "close");
        JNA.INSTANCE.toodle_destroy(rawPointer);
    }
}