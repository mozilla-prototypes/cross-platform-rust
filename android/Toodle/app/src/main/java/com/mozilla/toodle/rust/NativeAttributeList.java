package com.mozilla.toodle.rust;

import android.util.Log;

import com.sun.jna.Structure;
import com.sun.jna.ptr.IntByReference;

import java.io.Closeable;
import java.util.Arrays;
import java.util.List;

/**
 * Created by emilytoop on 01/03/2018.
 */

public class NativeAttributeList extends Structure implements Closeable {
    public static class ByReference extends NativeAttributeList implements Structure.ByReference {
    }

    public static class ByValue extends NativeAttributeList implements Structure.ByValue {
    }

    public IntByReference attributes;
    public int numberOfItems;
    // Used by the Swift counterpart, JNA does this for us automagically.
    public int len;

//    public List<Integer> getAttributes() {
//        final Integer[] array = (Integer[]) attributes.toArray(numberOfItems);
//        return Arrays.asList(array);
//    }

    @Override
    protected List<String> getFieldOrder() {
        return Arrays.asList("attributes", "numberOfItems", "len");
    }

    @Override
    public void close() {
        Log.i("NativeAttributeList", "close");
    }
}
