package com.mozilla.toodle.rust;

import android.util.Log;

import com.sun.jna.Pointer;

import java.util.Iterator;

public class ResultRowsIterator extends RustObject implements Iterator {

    Pointer nextPointer;

    ResultRowsIterator(Pointer iterator) {
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
    public ResultRow next() {
        Pointer next = this.nextPointer == null ? getNextPointer() : this.nextPointer;
        if(next == null) {
            return null;
        }

        return new ResultRow(next);
    }


    @Override
    public void close() {
        Log.i("ResultRow", "close");
        if(this.rawPointer != null) {
            JNA.INSTANCE.typed_value_result_set_iter_destroy(this.rawPointer);
        }
    }
}
