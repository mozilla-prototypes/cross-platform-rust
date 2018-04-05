package com.mozilla.toodle.rust;

import android.os.Handler;
import android.os.Looper;
import android.util.Log;

import com.sun.jna.Pointer;

import java.util.Date;
import java.util.UUID;

public class Query extends RustObject {

    public Query(Pointer pointer) {
        this.rawPointer = pointer;
    }

    void bindInt(String varName, int value) {
        if(this.rawPointer == null) {
            throw new NullPointerException("Query Builder consumed");
        }
        JNA.INSTANCE.query_builder_bind_int(this.rawPointer, varName, value);
    }

    void bindLong(String varName, long value) {
        if(this.rawPointer == null) {
            throw new NullPointerException("Query Builder consumed");
        }
        JNA.INSTANCE.query_builder_bind_long(this.rawPointer, varName, value);
    }

    void bindEntidReference(String varName, long value) {
        if(this.rawPointer == null) {
            throw new NullPointerException("Query Builder consumed");
        }
        JNA.INSTANCE.query_builder_bind_ref(this.rawPointer, varName, value);
    }

    void bindKeywordReference(String varName, String value) {
        if(this.rawPointer == null) {
            throw new NullPointerException("Query Builder consumed");
        }
        JNA.INSTANCE.query_builder_bind_ref_kw(this.rawPointer, varName, value);
    }

    void bindKeyword(String varName, String value) {
        if(this.rawPointer == null) {
            throw new NullPointerException("Query Builder consumed");
        }
        JNA.INSTANCE.query_builder_bind_kw(this.rawPointer, varName, value);
    }

    void bindBoolean(String varName, boolean value) {
        if(this.rawPointer == null) {
            throw new NullPointerException("Query Builder consumed");
        }
        JNA.INSTANCE.query_builder_bind_boolean(this.rawPointer, varName, value ? 1 : 0);
    }

    void bindDouble(String varName, double value) {
        if(this.rawPointer == null) {
            throw new NullPointerException("Query Builder consumed");
        }
        JNA.INSTANCE.query_builder_bind_double(this.rawPointer, varName, value);
    }

    void bindDate(String varName, Date value) {
        if(this.rawPointer == null) {
            throw new NullPointerException("Query Builder consumed");
        }
        JNA.INSTANCE.query_builder_bind_timestamp(this.rawPointer, varName, value.getTime());
    }

    void bindString(String varName, String value) {
        if(this.rawPointer == null) {
            throw new NullPointerException("Query Builder consumed");
        }
        JNA.INSTANCE.query_builder_bind_string(this.rawPointer, varName, value);
    }

    void bindUUID(String varName, UUID value) {
        if(this.rawPointer == null) {
            throw new NullPointerException("Query Builder consumed");
        }
        JNA.INSTANCE.query_builder_bind_uuid(this.rawPointer, varName, value.toString());
    }

    void executeMap(final QueryResultRowHandler handler) {
        if(this.rawPointer == null) {
            throw new NullPointerException("Query Builder consumed");
        }

        new Thread(new Runnable() {
            public void run() {
                NativeResult result = JNA.INSTANCE.query_builder_execute(rawPointer);
                rawPointer = null;

                if(result.isFailure()) {
                    Log.i("Query", result.err);
                    return;
                }
                ResultRows rows = new ResultRows(result.ok);
                for(ResultRow row: rows) {
                    handler.handleRow(row);
                }
            }
        }).start();
    }

    void execute(final QueryResultRowsHandler handler) {
        if(this.rawPointer == null) {
            throw new NullPointerException("Query Builder consumed");
        }

        new Thread(new Runnable() {
            public void run() {
                NativeResult result = JNA.INSTANCE.query_builder_execute(rawPointer);
                rawPointer = null;

                if(result.isFailure()) {
                    Log.i("Query", result.err);
                    return;
                }
                handler.handleRows(new ResultRows(result.ok));
            }
        }).start();
    }

    void executeScalar(final QueryResultValueHandler handler) {
        if(this.rawPointer == null) {
            throw new NullPointerException("Query Builder consumed");
        }

        new Thread(new Runnable() {
            public void run() {
                NativeResult result = JNA.INSTANCE.query_builder_execute_scalar(rawPointer);
                rawPointer = null;

                if(result.isFailure()) {
                    Log.i("Query", result.err);
                    return;
                }
                handler.handleValue(new TypedValue(result.ok));
            }
        }).start();
    }

    void executeColl(final QueryResultListHandler handler) {
        if(this.rawPointer == null) {
            throw new NullPointerException("Query Builder consumed");
        }

        new Thread(new Runnable() {
            public void run() {
                NativeResult result = JNA.INSTANCE.query_builder_execute_coll(rawPointer);
                rawPointer = null;

                if(result.isFailure()) {
                    Log.i("Query", result.err);
                    return;
                }
                handler.handleList(new ResultList(result.ok));
            }
        }).start();
    }

    void executeCollMap(final QueryResultValueHandler handler) {
        if(this.rawPointer == null) {
            throw new NullPointerException("Query Builder consumed");
        }

        new Thread(new Runnable() {
            public void run() {
                NativeResult result = JNA.INSTANCE.query_builder_execute_coll(rawPointer);
                rawPointer = null;

                if(result.isFailure()) {
                    Log.i("Query", result.err);
                    return;
                }

                ResultList list = new ResultList(result.ok);
                for(TypedValue value: list) {
                    handler.handleValue(value);
                }
            }
        }).start();
    }

    void executeTuple(final QueryResultRowHandler handler) {
        if(this.rawPointer == null) {
            throw new NullPointerException("Query Builder consumed");
        }

        new Thread(new Runnable() {
            public void run() {
                NativeResult result = JNA.INSTANCE.query_builder_execute_tuple(rawPointer);
                rawPointer = null;

                if(result.isFailure()) {
                    Log.i("Query", result.err);
                    return;
                }
                handler.handleRow(new ResultRow(result.ok));
            }
        }).start();
    }

    @Override
    public void close() {
        Log.i("Query", "close");

        if(this.rawPointer == null) {
            return;
        }
        JNA.INSTANCE.query_builder_destroy(this.rawPointer);
    }
}
