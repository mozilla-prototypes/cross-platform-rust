package com.mozilla.toodle.rust;

import android.support.annotation.Nullable;
import android.util.Log;

import com.sun.jna.Structure;
import com.sun.jna.ptr.NativeLongByReference;

import java.io.Closeable;
import java.util.Arrays;
import java.util.List;

/**
 * Created by emilytoop on 01/03/2018.
 */

public class NativeTxReportList extends Structure implements Closeable {
    public static class ByReference extends NativeTxReportList implements Structure.ByReference {
    }

    public static class ByValue extends NativeTxReportList implements Structure.ByValue {
    }

    public NativeTxReport.ByReference reports;
    public int numberOfItems;
    // Used by the Swift counterpart, JNA does this for us automagically.
    public int len;

    public List<NativeTxReport> getReports() {
        final NativeTxReport[] array = (NativeTxReport[]) reports.toArray(numberOfItems);
        return Arrays.asList(array);
    }

    @Override
    protected List<String> getFieldOrder() {
        return Arrays.asList("reports", "numberOfItems", "len");
    }

    @Override
    public void close() {
        Log.i("NativeTxReportList", "close");
        final NativeTxReport[] nativeReports = (NativeTxReport[]) reports.toArray(numberOfItems);
        for (NativeTxReport nativeReport : nativeReports) {
            nativeReport.close();
        }
    }
}
