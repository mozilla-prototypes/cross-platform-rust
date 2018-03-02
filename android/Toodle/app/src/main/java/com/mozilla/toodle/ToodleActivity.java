/* -*- Mode: Java; c-basic-offset: 4; tab-width: 20; indent-tabs-mode: nil; -*-
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package com.mozilla.toodle;

import android.app.Activity;
import android.content.Intent;
import android.os.Bundle;
import android.support.design.widget.FloatingActionButton;
import android.support.v4.widget.SwipeRefreshLayout;
import android.support.v7.widget.LinearLayoutManager;
import android.support.v7.widget.RecyclerView;
import android.text.TextUtils;
import android.util.Log;
import android.view.View;

import com.mozilla.toodle.rust.NativeResult;
import com.mozilla.toodle.rust.Toodle;

public class ToodleActivity extends Activity {
    private static final String LOG_TAG = "ToodleActivity";
    private RecyclerView listRecyclerView;
    private RecyclerView.Adapter listAdapter;
    private RecyclerView.LayoutManager layoutManager;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_toodle);

        listRecyclerView = findViewById(R.id.listItems);

        layoutManager = new LinearLayoutManager(this);
        listRecyclerView.setLayoutManager(layoutManager);

        listAdapter = new ListAdapter(getApplicationContext());
        listAdapter.setHasStableIds(true);
        listRecyclerView.setAdapter(listAdapter);

        final SwipeRefreshLayout refreshWrapper = findViewById(R.id.swiperefresh_wrapper);
        refreshWrapper.setOnRefreshListener(new SwipeRefreshLayout.OnRefreshListener() {
            @Override
            public void onRefresh() {
                NativeResult result = Toodle.getSharedInstance(getApplicationContext()).sync();
                Log.i(LOG_TAG, "Sync result: " + result);
                if (!TextUtils.isEmpty(result.error)) {
                    Log.i(LOG_TAG, "Sync error: " + result.error);
                    UiUtils.showError(getApplicationContext(), result.error);
                }
                refreshWrapper.setRefreshing(false);
            }
        });

        final FloatingActionButton newItemBtn = findViewById(R.id.newItem);
        newItemBtn.setOnClickListener(new View.OnClickListener() {
            @Override
            public void onClick(View v) {
                startActivity(new Intent(ToodleActivity.this, NewItemActivity.class));
            }
        });
    }
}
