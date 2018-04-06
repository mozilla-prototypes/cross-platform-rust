/* -*- Mode: Java; c-basic-offset: 4; tab-width: 20; indent-tabs-mode: nil; -*-
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package com.mozilla.toodle.rust;

import android.util.Log;

import com.sun.jna.Pointer;

import java.util.Iterator;

public class RelResultIterator extends RustObject implements Iterator {

    Pointer nextPointer;

    RelResultIterator(Pointer iterator) {
        this.rawPointer = iterator;
    }

    private Pointer getNextPointer() {
        return JNA.INSTANCE.rows_iter_next(this.rawPointer);
    }

    @Override
    public boolean hasNext() {
        this.nextPointer = getNextPointer();
        return this.nextPointer != null;
    }

    @Override
    public TupleResult next() {
        Pointer next = this.nextPointer == null ? getNextPointer() : this.nextPointer;
        if (next == null) {
            return null;
        }

        return new TupleResult(next);
    }


    @Override
    public void close() {
        Log.i("TupleResult", "close");
        if (this.rawPointer != null) {
            JNA.INSTANCE.typed_value_result_set_iter_destroy(this.rawPointer);
        }
    }
}
