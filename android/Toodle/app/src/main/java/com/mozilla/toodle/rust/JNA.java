/* -*- Mode: Java; c-basic-offset: 4; tab-width: 20; indent-tabs-mode: nil; -*-
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package com.mozilla.toodle.rust;

import com.sun.jna.Library;
import com.sun.jna.Native;
import com.sun.jna.NativeLibrary;
import com.sun.jna.Pointer;
import com.sun.jna.ptr.NativeLongByReference;

public interface JNA extends Library {
    String JNA_LIBRARY_NAME = "toodle";
    NativeLibrary JNA_NATIVE_LIB = NativeLibrary.getInstance(JNA_LIBRARY_NAME);

    JNA INSTANCE = (JNA) Native.loadLibrary(JNA_LIBRARY_NAME, JNA.class);

    Pointer new_toodle(String dbPath);
    Pointer store_open(String dbPath);

    void toodle_create_item(Pointer listManager, String name, NativeLongByReference dueDate);
    void toodle_update_item_by_uuid(Pointer listManager, String uuid, String name, NativeLongByReference dueDate, NativeLongByReference completionDate);
    void toodle_all_items(Pointer listManager, NativeItemsCallback callback);

    void destroy(Pointer obj);
    void query_builder_destroy(Pointer obj);
    void store_destroy(Pointer obj);
    void typed_value_destroy(Pointer obj);
    void typed_value_list_destroy(Pointer obj);
    void typed_value_list_iter_destroy(Pointer obj);
    void typed_value_result_set_destroy(Pointer obj);
    void typed_value_result_set_iter_destroy(Pointer obj);

    void store_register_observer(Pointer Store, String key, Pointer attributes, int len, NativeTxObserverCallback callback );
    void store_unregister_observer(Pointer Store, String key);
    long store_entid_for_attribute(Pointer Store, String attr);

    NativeResult store_sync(Pointer toodle, String userUuid, String serverUri);

    // query
    Pointer value_at_index(Pointer rows, int index);
    long value_at_index_as_long(Pointer rows, int index);
    long value_at_index_as_entid(Pointer rows, int index);
    String value_at_index_as_kw(Pointer rows, int index);
    String value_at_index_as_string(Pointer rows, int index);
    String value_at_index_as_uuid(Pointer rows, int index);
    long value_at_index_as_boolean(Pointer rows, int index);
    double value_at_index_as_double(Pointer rows, int index);
    long value_at_index_as_timestamp(Pointer rows, int index);

    long typed_value_as_long(Pointer value);
    long typed_value_as_entid(Pointer value);
    String typed_value_as_kw(Pointer value);
    String typed_value_as_string(Pointer value);
    String typed_value_as_uuid(Pointer value);
    long typed_value_as_boolean(Pointer value);
    double typed_value_as_double(Pointer value);
    long typed_value_as_timestamp(Pointer value);

    Pointer row_at_index(Pointer rows, int index);
    Pointer rows_iter(Pointer rows);
    Pointer rows_iter_next(Pointer iter);

    Pointer values_iter(Pointer rows);
    Pointer values_iter_next(Pointer iter);

    // Query Building
    Pointer store_query(Pointer store, String query);
    NativeResult store_value_for_attribute(Pointer store, long entid, String attribute);
    void query_builder_bind_int(Pointer store, String var, int value);
    void query_builder_bind_long(Pointer query, String var, long value);
    void query_builder_bind_ref(Pointer query, String var, long value);
    void query_builder_bind_ref_kw(Pointer query, String var, String value);
    void query_builder_bind_kw(Pointer query, String var, String value);
    void query_builder_bind_boolean(Pointer query, String var, int value);
    void query_builder_bind_double(Pointer query, String var, double value);
    void query_builder_bind_timestamp(Pointer query, String var, long value);
    void query_builder_bind_string(Pointer query, String var, String value);
    void query_builder_bind_uuid(Pointer query, String var, String value);

    // Query Execution
    NativeResult query_builder_execute(Pointer query);
    NativeResult query_builder_execute_scalar(Pointer query);
    NativeResult query_builder_execute_coll(Pointer query);
    NativeResult query_builder_execute_tuple(Pointer query);
}
