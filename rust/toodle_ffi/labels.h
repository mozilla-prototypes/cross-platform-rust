#ifndef categories_h
#define categories_h

#import "items.h"

struct Toodle;
struct Label;

struct Label* _Nonnull toodle_create_label(const struct Toodle* _Nonnull manager, const char* _Nonnull name, const char* _Nonnull color);
const struct Label* _Nonnull* _Nonnull toodle_get_all_labels(const struct Toodle* _Nonnull manager);
const size_t label_list_count(const struct Label* _Nonnull* _Nonnull list);
const void label_list_destroy(const struct Label* _Nonnull* _Nonnull list);
const struct Label* _Nonnull label_list_entry_at(const struct Label* _Nonnull* _Nonnull list, size_t index);
const void add_label(const struct Label* _Nonnull* _Nonnull list, const struct label* _Nonnull label);

const void label_destroy(const struct Label* _Nonnull label);
const char* _Nonnull label_get_name(const struct Label* _Nonnull label);
const char* _Nonnull label_get_color(const struct Label* _Nonnull label);
const void label_set_color(struct Label* _Nonnull label, const char* _Nonnull color);


#endif /* categories_h */