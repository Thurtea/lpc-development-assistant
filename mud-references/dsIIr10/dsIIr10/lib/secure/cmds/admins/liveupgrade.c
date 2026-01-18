#include <lib.h>

mixed cmd(string str) {
    write("This command is disabled for Dead Souls II, because it "+
      "is public domain and unsupported.");
    return 1;
}

string GetHelp() {
    return ("An unsupported command for this mud.");
}
