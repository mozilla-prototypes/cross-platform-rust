#ifndef items_h
#define items_h

struct Toodle;
struct CItem {
    char* _Nullable uuid;
    char* _Nonnull name;
    int64_t* _Nullable dueDate;
    int64_t* _Nullable completionDate;
};

typedef struct CItem CItem;

struct CItemList {
    struct CItem*_Nonnull* _Nonnull list;
    uint64_t* _Nonnull len;
};

struct Label;

const struct CItem* _Nullable toodle_create_item(const struct Toodle* _Nonnull manager, const char* _Nonnull name, const int64_t* _Nullable due_date);
const void toodle_update_item(const struct Toodle* _Nonnull manager, const struct CItem* _Nonnull item, const char* _Nonnull name, const int64_t* _Nullable due_date, const int64_t* _Nullable completion_date, struct label*_Nonnull* _Nullable list);
const void toodle_update_item_by_uuid (const struct Toodle* _Nonnull manager, const char* _Nonnull uuid, const char* _Nonnull name, const int64_t* _Nullable due_date, const int64_t* _Nullable completion_date);
const struct CItemList*_Nonnull toodle_get_all_items(const struct Toodle* _Nonnull manager);
const uint64_t item_list_count(const struct CItemList* _Nonnull list);
const struct CItem* _Nullable item_list_entry_at(const struct CItemList* _Nonnull list, size_t index);
const struct CItem* _Nullable toodle_item_for_uuid(const struct Toodle* _Nonnull manager, const char* _Nonnull uuid);

const void item_set_name(struct CItem* _Nonnull item, const char* _Nonnull description);
const void item_set_due_date(struct CItem* _Nonnull item, const int64_t* _Nullable due_date);
const void item_set_completion_date(struct CItem* _Nonnull item, const int64_t* _Nullable completion_date);

#endif /* items_h */
