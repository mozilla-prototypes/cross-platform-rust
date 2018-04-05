/* -*- Mode: Java; c-basic-offset: 4; tab-width: 20; indent-tabs-mode: nil; -*-
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package com.mozilla.toodle.rust;

import android.util.Log;

import com.sun.jna.Pointer;

import java.util.Iterator;


public class ResultRows extends RustObject implements Iterable<ResultRow> {

    public ResultRows(Pointer pointer) {
        this.rawPointer = pointer;
    }

    public ResultRow rowAtIndex(int index) {
        Pointer pointer = JNA.INSTANCE.row_at_index(this.rawPointer, index);
        if (pointer == null) {
            return null;
        }

        return new ResultRow(pointer);
    }

    @Override
    public ResultRowsIterator iterator() {
        Pointer iterPointer = JNA.INSTANCE.rows_iter(this.rawPointer);
        this.rawPointer = null;
        if (iterPointer == null) {
            return null;
        }
        return new ResultRowsIterator(iterPointer);
    }

    @Override
    public void close() {
        Log.i("ResultRows", "close");

        if (this.rawPointer != null) {
            JNA.INSTANCE.typed_value_result_set_destroy(this.rawPointer);
        }
    }
}
