#include "immortals.h"
#include "immort.h"
#include "path.h"
inherit"/std/room.c";

void init() {
//wtf is this? shield_it(SHIELDCITY,"/d/ss/daggerford/ladyluck");
::init();
}

void setup() {
set_long("You are on Immortal avenue.  You stand in the City of"+
" the Immortals.  You stare in awe at the wondrous buildings around"+
" you.  You feel the magic in the air, and feel at peace here. To"+
" the west of you lies "+ IM8A + 
", while to the east of you lies "+ IM8B +".  To the south of you"+
" lies a square.  In the center of the square lies three statues.\n");
set_short("Immortal square");
add_exit("north",ROOM+"ave7","road");
add_exit("south",ROOM+"ave9","road");
add_exit("west",ROOM+"ave11","road");
add_exit("east",ROOM+"ave14","road");
set_light(100);
}      
