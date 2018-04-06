/* -*- Mode: Java; c-basic-offset: 4; tab-width: 20; indent-tabs-mode: nil; -*-
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package com.mozilla.toodle.rust;

import android.util.Log;

import com.sun.jna.Structure;
import com.sun.jna.ptr.IntByReference;

import java.io.Closeable;
import java.util.Arrays;
import java.util.List;

public class NativeAttributeList extends Structure implements Closeable {
    public static class ByReference extends NativeAttributeList implements Structure.ByReference {
    }

    public static class ByValue extends NativeAttributeList implements Structure.ByValue {
    }

    public IntByReference attributes;
    public int numberOfItems;
    // Used by the Swift counterpart, JNA does this for us automagically.
    public int len;

    @Override
    protected List<String> getFieldOrder() {
        return Arrays.asList("attributes", "numberOfItems", "len");
    }

    @Override
    public void close() {
        Log.i("NativeAttributeList", "close");

        if (this.getPointer() != null) {
            JNA.INSTANCE.destroy(this.getPointer());
        }
    }
}
