/* -*- Mode: Java; c-basic-offset: 4; tab-width: 20; indent-tabs-mode: nil; -*-
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package com.mozilla.toodle.rust;

import android.support.annotation.Nullable;
import android.util.Log;

import com.sun.jna.Structure;
import com.sun.jna.ptr.NativeLongByReference;

import java.io.Closeable;
import java.util.Arrays;
import java.util.List;

public class NativeItem extends Structure implements Closeable {
    public static class ByReference extends NativeItem implements Structure.ByReference {
    }

    public static class ByValue extends NativeItem implements Structure.ByValue {
    }

    public String uuid;
    public String itemName;
    @Nullable public NativeLongByReference dueDate;
    @Nullable public NativeLongByReference completionDate;

    @Override
    protected List<String> getFieldOrder() {
        return Arrays.asList("uuid", "itemName", "dueDate", "completionDate");
    }

    @Override
    public void close() {
        Log.i("NativeItem", "close");
        if(this.getPointer() != null) {
            JNA.INSTANCE.destroy(this.getPointer());
        }
    }
}
