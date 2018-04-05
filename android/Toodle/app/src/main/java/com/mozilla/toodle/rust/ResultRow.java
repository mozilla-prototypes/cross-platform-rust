package com.mozilla.toodle.rust;

import android.util.Log;
import com.sun.jna.Pointer;

import java.util.Date;
import java.util.UUID;

public class ResultRow extends RustObject {

    public ResultRow(Pointer pointer) {
        this.rawPointer = pointer;
    }

    public TypedValue get(Integer index) {
        Pointer pointer = JNA.INSTANCE.value_at_index(this.rawPointer, index);
        if(pointer == null) {
            return null;
        }
        return new TypedValue(pointer);
    }

    public Long asLong(Integer index) {
        return JNA.INSTANCE.value_at_index_as_long(this.rawPointer, index);
    }

    public Long asEntid(Integer index) {
        return JNA.INSTANCE.value_at_index_as_entid(this.rawPointer, index);
    }

    public String asKeyword(Integer index) {
        return JNA.INSTANCE.value_at_index_as_kw(this.rawPointer, index);
    }

    public Boolean asBool(Integer index) {
        return JNA.INSTANCE.value_at_index_as_boolean(this.rawPointer, index) == 0 ? false : true;
    }

    public Double asDouble(Integer index) {
        return JNA.INSTANCE.value_at_index_as_double(this.rawPointer, index);
    }

    public Date asDate(Integer index) {
        return new Date(JNA.INSTANCE.value_at_index_as_timestamp(this.rawPointer, index));
    }

    public String asString(Integer index) {
        return JNA.INSTANCE.value_at_index_as_string(this.rawPointer, index);
    }

    public UUID asUUID(Integer index) {
        return UUID.fromString(JNA.INSTANCE.value_at_index_as_uuid(this.rawPointer, index));
    }

    @Override
    public void close() {
        Log.i("ResultRow", "close");
        if(this.rawPointer != null) {
            JNA.INSTANCE.typed_value_list_destroy(this.rawPointer);
        }
    }
}
