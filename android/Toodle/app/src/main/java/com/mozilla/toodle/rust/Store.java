package com.mozilla.toodle.rust;


import android.util.Log;

import com.sun.jna.Memory;
import com.sun.jna.Pointer;

public class Store extends RustObject {

    public Store(String dbPath) {
        this.rawPointer = JNA.INSTANCE.store_open(dbPath);
    }

    public Store() { }

    public Query query(String query) {
        return new Query(JNA.INSTANCE.store_query(this.rawPointer, query));
    }

    public NativeResult sync() {
        return JNA.INSTANCE.store_sync(rawPointer, "00000000-0000-0000-0000-000000000117", "http://mentat.dev.lcip.org/mentatsync/0.1");
    }

    public void registerObserver(String key, String[] attributes, NativeTxObserverCallback callback) {
        // turn string array into int array
        long[] attrEntids = new long[attributes.length];
        for(int i = 0; i < attributes.length; i++) {
            attrEntids[i] = JNA.INSTANCE.store_entid_for_attribute(this.rawPointer, attributes[i]);
        }
        Log.i("Store", "Registering observer {" + key + "} for attributes:");
        for (int i = 0; i < attrEntids.length; i++) {
            Log.i("Store", "entid: " + attrEntids[i]);
        }
        final Pointer entidsNativeArray = new Memory(8 * attrEntids.length);
        entidsNativeArray.write(0, attrEntids, 0, attrEntids.length);
        JNA.INSTANCE.store_register_observer(rawPointer, key, entidsNativeArray, attrEntids.length, callback);
    }

    public void unregisterObserver(String key) {
        JNA.INSTANCE.store_unregister_observer(rawPointer, key);
    }

    public TypedValue valueForAttributeOnEntity(String attribute, long entid) {
        NativeResult result = JNA.INSTANCE.store_value_for_attribute(this.rawPointer, entid, attribute);

        if(result.isSuccess()) {
            return new TypedValue(result.ok);
        }

        if (result.isFailure()) {
            Log.i("Store", result.err);
        }

        return null;
    }

    @Override
    public void close() {
        Log.i("Store", "close");
        if(this.rawPointer != null) {
            JNA.INSTANCE.store_destroy(this.rawPointer);
        }
    }
}
