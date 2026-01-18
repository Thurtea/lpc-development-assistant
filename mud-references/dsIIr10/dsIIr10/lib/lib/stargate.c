#include <lib.h>
#include <daemons.h>

inherit LIB_ITEM;

void create(){ 
    item::create();
    SetKeyName("stargate");
    SetId(({"stargate", "gate", "gateway", "ring"}));
    SetAdjectives(({"stargate"}));
    SetShort("A broken stargate.");
    SetLong("A non-functioning teleportation device.");
}

void init(){
    ::init();
}

varargs void SetOrigin(mixed args...){
}

string GetOrigin(){
    return 0;
}

void eventConnect(mixed args...){
}

int eventDisconnect(){
    return 0;
}

string status(){
    return 0;  
}
