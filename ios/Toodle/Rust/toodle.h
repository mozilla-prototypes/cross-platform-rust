#include <stdint.h>
#include "items.h"

struct toodle;

struct toodle* new_toodle(const char* uri);
void toodle_destroy(struct toodle* toodle);
