#include <stdint.h>
#include "labels.h"
#import "items.h"
#include "store.h"

struct Store*_Nonnull new_toodle(const char*_Nonnull uri);
struct Store*_Nonnull new_label(const char*_Nonnull uri);
void toodle_destroy(struct Store*_Nonnull toodle);
struct Result*_Nonnull toodle_sync(struct Store*_Nonnull toodle, const char*_Nonnull user_uuid, const char*_Nonnull server_uri);

