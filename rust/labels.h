#ifndef categories_h
#define categories_h

#import "items.h"

struct listManager;
struct label;

struct label* _Nonnull list_manager_create_label(const struct listManager* _Nonnull manager, const char* _Nonnull name, const char* _Nonnull color);
const struct label* _Nonnull* _Nonnull list_manager_get_all_labels(const struct listManager* _Nonnull manager);
const size_t label_list_count(const struct label* _Nonnull* _Nonnull list);
const void label_list_destroy(const struct label* _Nonnull* _Nonnull list);
const struct label* _Nonnull label_list_entry_at(const struct label* _Nonnull* _Nonnull list, size_t index);
const void add_label(const struct label* _Nonnull* _Nonnull list, const struct label* _Nonnull label);

const void label_destroy(const struct label* _Nonnull label);
const char* _Nonnull label_get_name(const struct label* _Nonnull label);
const char* _Nonnull label_get_color(const struct label* _Nonnull label);
const void label_set_color(struct label* _Nonnull label, const char* _Nonnull color);


#endif /* categories_h */
