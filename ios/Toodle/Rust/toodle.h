#include <stdint.h>
#include "labels.h"
#include "store.h"

struct Toodle;

struct TmpCallback {
    void *_Nonnull obj;
    void (*_Nonnull destroy)(void *_Nonnull obj);
    void (*_Nonnull callback_fn)();
};

struct Toodle*_Nonnull new_toodle(struct Store*_Nonnull store);
void toodle_destroy(struct Toodle*_Nonnull toodle);

void callback(struct TmpCallback callback);
