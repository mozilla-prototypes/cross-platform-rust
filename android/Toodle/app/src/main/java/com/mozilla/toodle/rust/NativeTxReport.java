package com.mozilla.toodle.rust;

import android.util.Log;

import com.sun.jna.Pointer;
import com.sun.jna.Structure;

import java.io.Closeable;
import java.util.Arrays;
import java.util.List;

/**
 * Created by emilytoop on 01/03/2018.
 */

public class NativeTxReport extends Structure implements Closeable {
    public static class ByReference extends NativeTxReport implements Structure.ByReference {
    }

    public static class ByValue extends NativeTxReport implements Structure.ByValue {
    }

    public int txid;
    public Pointer changes;
    public int numberOfItems;
//    // Used by the Swift counterpart, JNA does this for us automagically.
    public int changes_len;

    public List<Long> getChanges() {
        final long[] array = (long[]) changes.getLongArray(0, numberOfItems);
        Long[] longArray = new Long[numberOfItems];
        int idx = 0;
        for(long change: array) {
            longArray[idx++] = change;
        }
        return Arrays.asList(longArray);
    }

    @Override
    protected List<String> getFieldOrder() {
        return Arrays.asList("txid", "changes", "changes_len", "numberOfItems");
    }

    @Override
    public void close() {
        Log.i("NativeTxReport", "close");
//        JNA.INSTANCE.item_c_destroy(this.getPointer());
    }
}
