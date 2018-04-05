/* -*- Mode: Java; c-basic-offset: 4; tab-width: 20; indent-tabs-mode: nil; -*-
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package com.mozilla.toodle;

import android.content.Context;
import android.os.Handler;
import android.os.Looper;
import android.support.annotation.Nullable;
import android.support.v7.widget.RecyclerView;
import android.util.Log;
import android.view.LayoutInflater;
import android.view.View;
import android.view.ViewGroup;
import android.widget.CheckBox;
import android.widget.LinearLayout;
import android.widget.TextView;

import com.mozilla.toodle.rust.NativeItemSet;
import com.mozilla.toodle.rust.NativeItemsCallback;
import com.mozilla.toodle.rust.NativeItemsChangedCallback;
import com.mozilla.toodle.rust.NativeTxObserverCallback;
import com.mozilla.toodle.rust.NativeTxReportList;
import com.mozilla.toodle.rust.Toodle;

import java.lang.ref.WeakReference;
import java.text.SimpleDateFormat;
import java.util.ArrayList;
import java.util.Date;
import java.util.List;

public class ListAdapter extends RecyclerView.Adapter<ListAdapter.ViewHolder> {
    private static final String LOG_TAG = "RustyToodleJavaLA";

    private List<Item> dataset = new ArrayList<>(0);
    private final Context context;
    private final Toodle toodle;

    static class NativeTXObserverCallbackInner implements NativeTxObserverCallback {
        private final WeakReference<ListAdapter> listAdapterWeakReference;

        NativeTXObserverCallbackInner(WeakReference<ListAdapter> listAdapterWeakReference) {
            this.listAdapterWeakReference = listAdapterWeakReference;
        }

        public void transactionObserverCalled(String key, NativeTxReportList.ByReference reports) {
            final ListAdapter listAdapter = listAdapterWeakReference.get();
            if (listAdapter == null) {
                Log.i(LOG_TAG, "No list adapter");
                return;
            }

            // TODO This is a hack around observer firing at a wrong moment
            new Handler(Looper.getMainLooper()).post(new Runnable() {
                @Override
                public void run() {
                    listAdapter.fetchItems();
                }
            });
        }
    }

    // We must keep reference to the callback around, otherwise it'll get GC'd and the native code
    // will call an empty stub instead of our callback.
    private final NativeTxObserverCallback nativeTxObserverCallback = new NativeTXObserverCallbackInner(
            new WeakReference<>(this)
    );

    ListAdapter(Context context) {
        this.context = context;
        this.toodle = Toodle.getSharedInstance(context);
        String[] attributes = {":todo/uuid", ":todo/name", ":todo/completion_date", ":todo/due_date"};
        toodle.registerObserver("ListAdapter", attributes, nativeTxObserverCallback);
        this.fetchItems();
    }

    static class ViewHolder extends RecyclerView.ViewHolder {
        private final LinearLayout itemView;

        ViewHolder(LinearLayout v) {
            super(v);
            itemView = v;
        }
    }

    private void fetchItems() {
        final WeakReference<ListAdapter> listAdapterWeakReference = new WeakReference<>(this);
        toodle.getAllItems(new ItemsCallback() {
            @Override
            public void items(ArrayList<Item> itemSet) {
                Log.i(LOG_TAG, "fetchItems: items");
                final ListAdapter listAdapter = listAdapterWeakReference.get();
                if (listAdapter == null) {
                    Log.i(LOG_TAG, "fetchItems: no listadapter");
                    return;
                }
                listAdapter.dataset = itemSet;

                new Handler(Looper.getMainLooper()).post(new Runnable() {
                    @Override
                    public void run() {
                        listAdapter.notifyDataSetChanged();
                    }
                });
            }
        });
    }

    @Override
    public ViewHolder onCreateViewHolder(ViewGroup parent, int viewType) {
        final LinearLayout v = (LinearLayout) LayoutInflater.from(parent.getContext())
                .inflate(R.layout.item, parent, false);

        return new ViewHolder(v);

    }

    @Override
    public void onBindViewHolder(final ViewHolder holder, final int position) {
        final Item item = dataset.get(position);
        ((TextView) holder.itemView.findViewById(R.id.itemTitle)).setText(item.name());
        final Long dueDate = item.dueDate().getTime();
        if (dueDate != null) {
            ((TextView) holder.itemView.findViewById(R.id.itemDueDate)).setText(
                    context.getResources().getString(
                            R.string.due_date,
                            SimpleDateFormat.getDateInstance(SimpleDateFormat.MEDIUM).format(dueDate * 1000)
                    )
            );
        }
        Date date = item.completionDate();
        if (date != null) {
            final Long completionDate = date.getTime();
            final CheckBox itemDoneCheckbox = holder.itemView.findViewById(R.id.itemDone);
            itemDoneCheckbox.setChecked(completionDate != null);
            itemDoneCheckbox.setOnClickListener(new View.OnClickListener() {
                @Override
                public void onClick(View v) {
                    boolean currentState = ((CheckBox) v).isChecked();
                    final Item item = dataset.get(holder.getAdapterPosition());
                    if (currentState) {
                        item.completionDate(System.currentTimeMillis());
                    } else {
                        item.completionDate(null);
                    }
                    item.update(context);
                }
            });
        }
    }

    @Override
    public int getItemCount() {
        return dataset.size();
    }

    @Override
    public long getItemId(int position) {
        return dataset.get(position).uuid().hashCode();
    }

    @Override
    public void onViewRecycled(ViewHolder holder) {
        super.onViewRecycled(holder);
        try (final Toodle toodle = this.toodle) {
            toodle.unregisterObserver("ListAdapter");
        }
    }
}
