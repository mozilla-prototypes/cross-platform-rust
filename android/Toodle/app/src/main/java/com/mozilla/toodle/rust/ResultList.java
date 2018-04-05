package com.mozilla.toodle.rust;

import android.support.annotation.NonNull;
import android.util.Log;

import com.sun.jna.Pointer;

import java.util.Iterator;

public class ResultList extends ResultRow implements Iterable<TypedValue> {

    public ResultList(Pointer pointer) {
        super(pointer);
    }

    @Override
    public void close() {
        Log.i("ResultList", "close");

        if(this.rawPointer != null) {
            JNA.INSTANCE.destroy(this.rawPointer);
        }
    }

    @Override
    public ResultListIterator iterator() {
        Pointer iterPointer = JNA.INSTANCE.values_iter(this.rawPointer);
        this.rawPointer = null;
        if(iterPointer == null) {
            return null;
        }
        return new ResultListIterator(iterPointer);
    }
}
