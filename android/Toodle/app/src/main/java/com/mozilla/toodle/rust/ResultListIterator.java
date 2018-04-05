package com.mozilla.toodle.rust;

import android.util.Log;

import com.sun.jna.Pointer;

import java.io.IOException;
import java.util.Iterator;

public class ResultListIterator extends RustObject implements Iterator {

    Pointer nextPointer;

    ResultListIterator(Pointer iterator) {
        this.rawPointer = iterator;
    }

    private Pointer getNextPointer() {
        return JNA.INSTANCE.values_iter_next(this.rawPointer);
    }

    @Override
    public boolean hasNext() {
        this.nextPointer = getNextPointer();
        return this.nextPointer != null;
    }

    @Override
    public TypedValue next() {
        Pointer next = this.nextPointer == null ? getNextPointer() : this.nextPointer;
        if(next == null) {
            return null;
        }

        return new TypedValue(next);
    }

    @Override
    public void close() {
        Log.i("ResultRow", "close");
        if(this.rawPointer != null) {
            JNA.INSTANCE.typed_value_list_iter_destroy(this.rawPointer);
        }
    }
}
