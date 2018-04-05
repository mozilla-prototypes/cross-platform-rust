/* -*- Mode: Java; c-basic-offset: 4; tab-width: 20; indent-tabs-mode: nil; -*-
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package com.mozilla.toodle.rust;

import android.util.Log;
import com.sun.jna.Pointer;

import java.util.Date;
import java.util.UUID;

public class TypedValue extends RustObject {

    public TypedValue(Pointer pointer) {
        this.rawPointer = pointer;
    }

    public Long asLong() {
        long value = JNA.INSTANCE.typed_value_as_long(this.rawPointer);
        this.rawPointer = null;
        return value;
    }

    public Long asEntid() {
        long value = JNA.INSTANCE.typed_value_as_entid(this.rawPointer);
        this.rawPointer = null;
        return value;
    }

    public String asKeyword() {
        String value = JNA.INSTANCE.typed_value_as_kw(this.rawPointer);
        this.rawPointer = null;
        return value;
    }

    public Boolean asBoolean() {
        long value = JNA.INSTANCE.typed_value_as_boolean(this.rawPointer);
        this.rawPointer = null;
        return value == 0 ? false : true;
    }

    public Double asDouble() {
        double value = JNA.INSTANCE.typed_value_as_double(this.rawPointer);
        this.rawPointer = null;
        return value;
    }

    public Date asDate() {
        long value = JNA.INSTANCE.typed_value_as_timestamp(this.rawPointer);
        this.rawPointer = null;
        return new Date(value);
    }

    public String asString() {
        String value = JNA.INSTANCE.typed_value_as_string(this.rawPointer);
        this.rawPointer = null;
        return value;
    }

    public UUID asUUID() {
        String value = JNA.INSTANCE.typed_value_as_uuid(this.rawPointer);
        this.rawPointer = null;
        return UUID.fromString(value);
    }

    @Override
    public void close() {
        Log.i("TypedValue", "close");

        if (this.rawPointer != null) {
            JNA.INSTANCE.typed_value_destroy(this.rawPointer);
        }
    }
}
