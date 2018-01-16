#include <stdint.h>
#include "labels.h"

struct toodle;

struct toodle* new_toodle(const char* uri);
void toodle_destroy(struct toodle* toodle);