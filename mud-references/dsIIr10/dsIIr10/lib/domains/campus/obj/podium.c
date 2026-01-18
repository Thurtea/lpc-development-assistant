#include <lib.h>
#include <vendor_types.h>

inherit LIB_STORAGE;

static void create() {
    storage::create();
    SetShort("a podium");
    SetLong("An object often used in lectures.");
    SetMass(100);
    SetId( ({"podium", "speaker's podium", "podium.c"}) );
    SetKeyName("podium");
    SetId(({"podium","handler"}));
    SetAdjectives(({"wood","wooden","meeting","speaker's","Speaker's"}));
    SetPreventGet("You can't get that.");
    SetMaxCarry(20);
}

void init() {
    ::init();
}
