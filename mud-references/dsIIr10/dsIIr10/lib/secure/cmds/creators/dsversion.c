#include <lib.h>
inherit LIB_DAEMON;

mixed cmd(string args) {
    write("This command is disabled in Dead Souls II, which "+
      "is an unsupported version of Dead Souls.");
    return 1;
}

string GetHelp() {
    return ("A broken command.");
}
