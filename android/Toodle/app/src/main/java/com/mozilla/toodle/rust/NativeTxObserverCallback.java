package com.mozilla.toodle.rust;

/**
 * Created by emilytoop on 01/03/2018.
 */

import com.sun.jna.Pointer;

public interface NativeTxObserverCallback {
    void transactionObserverCalled(String key, NativeTxReportList.ByReference reports);
}
