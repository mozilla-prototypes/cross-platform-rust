/* -*- Mode: Java; c-basic-offset: 4; tab-width: 20; indent-tabs-mode: nil; -*-
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package com.mozilla.toodle.rust;

import android.util.Log;

import com.sun.jna.Pointer;

public class CollResult extends TupleResult implements Iterable<TypedValue> {

    public CollResult(Pointer pointer) {
        super(pointer);
    }

    @Override
    public void close() {
        Log.i("CollResult", "close");

        if (this.rawPointer != null) {
            JNA.INSTANCE.destroy(this.rawPointer);
        }
    }

    @Override
    public ColResultIterator iterator() {
        Pointer iterPointer = JNA.INSTANCE.values_iter(this.rawPointer);
        this.rawPointer = null;
        if (iterPointer == null) {
            return null;
        }
        return new ColResultIterator(iterPointer);
    }
}
