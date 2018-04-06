/* -*- Mode: Java; c-basic-offset: 4; tab-width: 20; indent-tabs-mode: nil; -*-
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package com.mozilla.toodle.rust;

import android.util.Log;

import com.sun.jna.Pointer;

import java.util.Date;
import java.util.UUID;

public class Query extends RustObject {

    public Query(Pointer pointer) {
        this.rawPointer = pointer;
    }

    void bindInt(String varName, int value) {
        this.validate();
        JNA.INSTANCE.query_builder_bind_int(this.rawPointer, varName, value);
    }

    void bindLong(String varName, long value) {
        this.validate();
        JNA.INSTANCE.query_builder_bind_long(this.rawPointer, varName, value);
    }

    void bindEntidReference(String varName, long value) {
        this.validate();
        JNA.INSTANCE.query_builder_bind_ref(this.rawPointer, varName, value);
    }

    void bindKeywordReference(String varName, String value) {
        this.validate();
        JNA.INSTANCE.query_builder_bind_ref_kw(this.rawPointer, varName, value);
    }

    void bindKeyword(String varName, String value) {
        this.validate();
        JNA.INSTANCE.query_builder_bind_kw(this.rawPointer, varName, value);
    }

    void bindBoolean(String varName, boolean value) {
        this.validate();
        JNA.INSTANCE.query_builder_bind_boolean(this.rawPointer, varName, value ? 1 : 0);
    }

    void bindDouble(String varName, double value) {
        this.validate();
        JNA.INSTANCE.query_builder_bind_double(this.rawPointer, varName, value);
    }

    void bindDate(String varName, Date value) {
        this.validate();
        JNA.INSTANCE.query_builder_bind_timestamp(this.rawPointer, varName, value.getTime());
    }

    void bindString(String varName, String value) {
        this.validate();
        JNA.INSTANCE.query_builder_bind_string(this.rawPointer, varName, value);
    }

    void bindUUID(String varName, UUID value) {
        this.validate();
        JNA.INSTANCE.query_builder_bind_uuid(this.rawPointer, varName, value.toString());
    }

    void executeMap(final TupleResultHandler handler) {
        this.validate();
        NativeResult result = JNA.INSTANCE.query_builder_execute(rawPointer);
        rawPointer = null;

        if (result.isFailure()) {
            Log.e("Query", result.err);
            return;
        }

        RelResult rows = new RelResult(result.ok);
        for (TupleResult row: rows) {
            handler.handleRow(row);
        }
    }

    void execute(final RelResultHandler handler) {
        this.validate();
        NativeResult result = JNA.INSTANCE.query_builder_execute(rawPointer);
        rawPointer = null;

        if (result.isFailure()) {
            Log.e("Query", result.err);
            return;
        }
        handler.handleRows(new RelResult(result.ok));
    }

    void executeScalar(final ScalarResultHandler handler) {
        this.validate();
        NativeResult result = JNA.INSTANCE.query_builder_execute_scalar(rawPointer);
        rawPointer = null;

        if (result.isFailure()) {
            Log.e("Query", result.err);
            return;
        }
        handler.handleValue(new TypedValue(result.ok));
    }

    void executeColl(final CollResultHandler handler) {
        this.validate();
        NativeResult result = JNA.INSTANCE.query_builder_execute_coll(rawPointer);
        rawPointer = null;

        if (result.isFailure()) {
            Log.e("Query", result.err);
            return;
        }
        handler.handleList(new CollResult(result.ok));
    }

    void executeCollMap(final ScalarResultHandler handler) {
        this.validate();

        NativeResult result = JNA.INSTANCE.query_builder_execute_coll(rawPointer);
        rawPointer = null;

        if (result.isFailure()) {
            Log.e("Query", result.err);
            return;
        }

        CollResult list = new CollResult(result.ok);
        for(TypedValue value: list) {
            handler.handleValue(value);
        }
    }

    void executeTuple(final TupleResultHandler handler) {
        this.validate();
        NativeResult result = JNA.INSTANCE.query_builder_execute_tuple(rawPointer);
        rawPointer = null;

        if (result.isFailure()) {
            Log.e("Query", result.err);
            return;
        }
        handler.handleRow(new TupleResult(result.ok));
    }

    @Override
    public void close() {
        Log.i("Query", "close");

        if (this.rawPointer == null) {
            return;
        }
        JNA.INSTANCE.query_builder_destroy(this.rawPointer);
    }
}
