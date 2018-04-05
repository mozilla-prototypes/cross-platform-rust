/* -*- Mode: Java; c-basic-offset: 4; tab-width: 20; indent-tabs-mode: nil; -*-
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package com.mozilla.toodle.rust;

import com.sun.jna.Pointer;
import com.sun.jna.Structure;

import java.io.Closeable;
import java.io.IOException;
import java.util.Arrays;
import java.util.List;

public class NativeResult extends Structure implements Closeable {
    public static class ByReference extends NativeItem implements Structure.ByReference {
    }

    public static class ByValue extends NativeItem implements Structure.ByValue {
    }

    public Pointer ok;
    public String err;

    public boolean isSuccess() {
        return this.ok != null;
    }

    public boolean isFailure() {
        return this.err != null;
    }

    @Override
    protected List<String> getFieldOrder() {
        return Arrays.asList("ok", "err");
    }

    @Override
    public void close() throws IOException {
        // TODO do we need to make sure the error string is memory managed properly?

        if(this.getPointer() != null) {
            JNA.INSTANCE.destroy(this.getPointer());
        }
    }
}