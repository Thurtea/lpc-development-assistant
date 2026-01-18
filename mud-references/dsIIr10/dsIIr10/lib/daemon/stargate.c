#include <lib.h>

inherit LIB_DAEMON;

static void create(){
}

int SetStargate(mixed args...){
    return 0;
}

mapping GetStargate(string address){
    return 0;
}

int RemoveStargate(string address){
    return 0;
}

mapping GetStargates(){
    return ([]);
}

int SetStatus(mixed args...){
    return 1;
}

string GetStatus(mixed args...){
    return "unknown";
}

mixed GetDestination(mixed args...){
    return 0;
}

mixed GetEndpoint(mixed args...){
    return 0;
}

int eventConnect(mixed args...){
    return 0;
}

int eventDisconnect(mixed args...){
    return 0;
}

void ResetGates(){
}
