/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#ifndef store_h
#define store_h

struct Store;

struct AttributeList {
    int64_t*_Nonnull* _Nonnull attributes;
    uint64_t len;
};

struct TxReportList {
    struct ExternTxReport*_Nonnull* _Nonnull reports;
    uint64_t len;
};

struct ExternTxReport {
    int64_t txid;
    struct AttributeList*_Nonnull* _Nonnull changes;
};

struct Store*_Nonnull  new_store(const char*_Nonnull uri);
void store_destroy(struct Store*_Nonnull  store);

void store_register_observer(struct Store*_Nonnull  store, const char* _Nonnull key, const int64_t* _Nonnull attributes, const int64_t len, void (*_Nonnull callback_fn)(const char* _Nonnull key, const struct TxReportList* _Nonnull reports));
void store_unregister_observer(struct Store*_Nonnull  store, const char* _Nonnull key);
int64_t store_entid_for_attribute(struct Store*_Nonnull store, const char*_Nonnull attr);

const struct ExternTxReport* _Nullable tx_report_list_entry_at(const struct TxReportList* _Nonnull list, size_t index);
const struct int64_t changelist_entry_at(const struct AttributeList* _Nonnull list, size_t index);

#endif /* store_h */
