#include <stdint.h>
#include "labels.h"
#include "store.h"

struct Result {
    char* _Nullable error;
};

struct Store*_Nonnull new_toodle(const char*_Nonnull uri);
void toodle_destroy(struct Store*_Nonnull toodle);

struct Result*_Nonnull toodle_sync(struct Store*_Nonnull toodle);
