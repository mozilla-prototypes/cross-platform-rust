/* -*- Mode: Java; c-basic-offset: 4; tab-width: 20; indent-tabs-mode: nil; -*-
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package com.mozilla.toodle;

import android.content.Context;

import com.mozilla.toodle.rust.NativeItem;
import com.mozilla.toodle.rust.NativeResultException;
import com.mozilla.toodle.rust.Toodle;

import java.util.ArrayList;
import java.util.Date;
import java.util.List;

public class Item {
    private String uuid;
    private String name;
    private Long dueDate;
    private Long completionDate;

    public String name() {
        return name;
    }

    Item name(final String name) {
        this.name = name;
        return this;
    }

    public Long dueDate() {
        return dueDate;
    }

    Item dueDate(final int year, final int month, final int date) {
        dueDate = new Date(year, month, date).getTime();
        return this;
    }

    private static Item fromNativeItem(NativeItem nativeItem) {
        final Item item = new Item();
        item.uuid = nativeItem.uuid;
        item.name = nativeItem.itemName;
        item.dueDate = nativeItem.dueDate.longValue();
        item.completionDate = nativeItem.completionDate.longValue();
        return item;
    }

    static ArrayList<Item> fromNativeItems(List<NativeItem> nativeItems) {
        final ArrayList<Item> items = new ArrayList<>(nativeItems.size());

        for (NativeItem nativeItem : nativeItems) {
            items.add(fromNativeItem(nativeItem));
        }

        return items;
    }

    void create(Context context) {
        try (final Toodle toodle = new Toodle(context)) {
            toodle.createItem(this);
        } catch (NativeResultException e) {
            UiUtils.showError(context, e.getMessage());
        }
    }
}