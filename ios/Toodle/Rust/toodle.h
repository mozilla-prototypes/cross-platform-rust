#include <stdint.h>
#include "labels.h"
#include "store.h"

struct TmpCallback {
    void *_Nonnull obj;
    void (*_Nonnull destroy)(void *_Nonnull obj);
    void (*_Nonnull callback_fn)();
};

struct Store*_Nonnull new_toodle(const char*_Nonnull uri);
void toodle_destroy(struct Store*_Nonnull toodle);

void callback(struct TmpCallback callback);
