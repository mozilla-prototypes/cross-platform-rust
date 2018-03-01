package com.mozilla.toodle.rust;

import android.support.annotation.Nullable;
import android.util.Log;

import com.sun.jna.Structure;
import com.sun.jna.ptr.IntByReference;
import com.sun.jna.ptr.NativeLongByReference;

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
    public int[] changes;
    // Used by the Swift counterpart, JNA does this for us automagically.
    public int changes_len;

    @Override
    protected List<String> getFieldOrder() {
        return Arrays.asList("txid", "changes", "changes_len");
    }

    @Override
    public void close() {
        Log.i("NativeTxReport", "close");
//        JNA.INSTANCE.item_c_destroy(this.getPointer());
    }
}
