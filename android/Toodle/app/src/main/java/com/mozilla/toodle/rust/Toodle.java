/* -*- Mode: Java; c-basic-offset: 4; tab-width: 20; indent-tabs-mode: nil; -*-
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package com.mozilla.toodle.rust;

import android.content.Context;

import com.mozilla.toodle.Item;

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
                item.dueDate()
        );
    }

    public void registerChangedItemsCallback(NativeItemsChangedCallback callback) {
        JNA.INSTANCE.toodle_on_items_changed(callback);
    }

    public void getAllItems(NativeItemsCallback callback) {
        JNA.INSTANCE.toodle_get_all_items(rawPointer, callback);
    }

    @Override
    public void close() {
        JNA.INSTANCE.toodle_destroy(rawPointer);
    }
}