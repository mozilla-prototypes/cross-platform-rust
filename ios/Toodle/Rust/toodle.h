#include <stdint.h>
#include "labels.h"
#include "store.h"

struct Store*_Nonnull new_toodle(const char*_Nonnull uri);
void toodle_destroy(struct Store*_Nonnull toodle);
