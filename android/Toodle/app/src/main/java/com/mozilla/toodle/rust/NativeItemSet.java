/* -*- Mode: Java; c-basic-offset: 4; tab-width: 20; indent-tabs-mode: nil; -*-
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package com.mozilla.toodle.rust;

import android.util.Log;

import com.sun.jna.Structure;

import java.io.Closeable;
import java.util.Arrays;
import java.util.List;


public class NativeItemSet extends Structure implements Closeable {
    public static class ByReference extends NativeItemSet implements Structure.ByReference {
    }

    public static class ByValue extends NativeItemSet implements Structure.ByValue {
    }

    public NativeItem.ByReference items;
    public int numberOfItems;
    // Used by the Swift counterpart, JNA does this for us automagically.
    public int len;

    public List<NativeItem> getItems() {
        final NativeItem[] array = (NativeItem[]) items.toArray(numberOfItems);
        return Arrays.asList(array);
    }

    @Override
    protected List<String> getFieldOrder() {
        return Arrays.asList("items", "numberOfItems", "len");
    }

    @Override
    public void close() {
        Log.i("NativeItemSet", "close");
        final NativeItem[] nativeItems = (NativeItem[]) items.toArray(numberOfItems);
        for (NativeItem nativeItem : nativeItems) {
            nativeItem.close();
        }
    }
}
