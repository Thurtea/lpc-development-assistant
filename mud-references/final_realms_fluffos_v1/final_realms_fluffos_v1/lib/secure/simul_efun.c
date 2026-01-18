#ifdef MUD_NAME
string mud_name() { return MUD_NAME; }
#else
string mud_name() { return "fr"; }
#endif

#ifdef VERSION
string version() { return VERSION; }
#else
#ifdef __VERSION__
string version() { return __VERSION__; }
#else
string version() { return "Unknown"; }
#endif
#endif

mapping Uncolor = ([ "RESET": "", "BOLD": "", "FLASH":"", "BLACK":"", "RED":"",
  "BLUE":"", "CYAN":"", "MAGENTA":"", "ORANGE":"", "YELLOW":"",
  "GREEN":"", "WHITE":"", "BLACK":"", "B_RED":"", "B_ORANGE":"",
  "B_YELLOW":"", "B_BLACK":"", "B_CYAN":"","B_WHITE":"", "B_GREEN":"",
  "B_MAGENTA":"", "STATUS":"", "WINDOW":"", "INITTERM": "",
  "ENDTERM":""]);

#if 0
int command ( string cmd )
  {
  //On Windows, the lack of itimer causes eval_cost to always
  //be zero. YOU fix this. In the meantime, this workaround
  //prevents constant error messages from add_actions when running
  //FR in windows.
  int ret = efun::command(cmd);
  //this_object()->tc("ret: "+ret);
  //if(strsrch(lower_case(__ARCH__),"windows") != -1) return -666;
  //else return ret;
  return ret;
  }
#endif

int absolute_value(int x) {
    return ( (x>-1) ? x : -x);
}

int abs(int x) {
    return absolute_value(x);
}

string strip_colours(string str){
    return terminal_color(str, Uncolor);
}

string cardinal(int x) {
    string tmp;
    int a;

    if(!x) return "zero";
    if(x < 0) {
        tmp = "negative ";
        x = absolute_value(x);
    }
    else tmp = "";
    switch(x) {
        case 1: return tmp+"one";
        case 2: return tmp+"two";
        case 3: return tmp+"three";
        case 4: return tmp+"four";
        case 5: return tmp+"five";
        case 6: return tmp+"six";
        case 7: return tmp+"seven";
        case 8: return tmp+"eight";
        case 9: return tmp+"nine";
        case 10: return tmp+"ten";
        case 11: return tmp+"eleven";
        case 12: return tmp+"twelve";
        case 13: return tmp+"thirteen";
        case 14: return tmp+"fourteen";
        case 15: return tmp+"fifteen";
        case 16: return tmp+"sixteen";
        case 17: return tmp+"seventeen";
        case 18: return tmp+"eighteen";
        case 19: return tmp+"nineteen";
        case 20: return tmp+"twenty";
        default:
            if(x > 1000000000) return "over a billion";
            else if(a = x /1000000) {
                if(x = x %1000000) 
                    return sprintf("%s million %s", cardinal(a), cardinal(x));
                else return sprintf("%s million", cardinal(a));
            }
            else if(a = x / 1000) {
                if(x = x % 1000) 
                    return sprintf("%s thousand %s", cardinal(a), cardinal(x));
                else return sprintf("%s thousand", cardinal(a));
            }
            else if(a = x / 100) {
                if(x = x % 100) 
                    return sprintf("%s hundred %s", cardinal(a), cardinal(x));
                else return sprintf("%s hundred", cardinal(a));
            }
            else {
                a = x / 10;
                if(x = x % 10) tmp = "-"+cardinal(x);
                else tmp = "";
                switch(a) {
                case 2: return "twenty"+tmp;
                case 3: return "thirty"+tmp;
                case 4: return "forty"+tmp;
                case 5: return "fifty"+tmp;
                case 6: return "sixty"+tmp;
                case 7: return "seventy"+tmp;
                case 8: return "eighty"+tmp;
                case 9: return "ninety"+tmp;
                default: return "error";
                }
            }
        }
    }

string consolidate(int x, string str) {
    string *words;
    string tmp;

    if( x == 1 || !sizeof(str) ) return str;
    words = explode(str, " ");
    if( sscanf(words[<1], "(%s)", tmp) ) {
        if( sizeof(words) == 1 ) 
            return "(" + consolidate(x, tmp) + ")";
        else return consolidate(x, implode(words[0..<2], " ")) + 
            " (" + tmp + ")";
    }
    if( sscanf(words[<1], "[%s]", tmp) ) {
        if( sizeof(words) == 1 )
            return "[" + consolidate(x, tmp) + "]";
        else return consolidate(x, implode(words[0..<2], " ")) +
            " [" + tmp + "]";
    }
    if( words[0][0..1] == "%^" ) {
        string *parts;
        string part, colour = "";
        int i = 0;

        parts = explode(words[0], "%^");
        if( sizeof(parts) == 1 ) {
            if( sizeof(words) == 1 ) return words[0];
            else return words[0] + consolidate(x, implode(words[1..], " "));
        }

        foreach(part in parts) {
            if( sizeof(part) && !sizeof(strip_colours("%^" + part + "%^")) )
                colour += ("%^" + part + "%^"); 
            else return colour + consolidate(x, 
                  (implode(parts[i..], "%^")) + " " + 
                  (implode(words[1..], " ")) );
            i++;
        }
        return words[0] + " " + consolidate(x, implode(words[1..], " "));

    }
    if( member_array(lower_case(strip_colours(words[0])), 
        ({"a", "an", "the", "one"}) ) > -1 ) words = words[1..];
    return (cardinal(x) + " " + pluralize(implode(words, " ")));
}

// roll_MdN stolen from dw by crat for fr
int roll_MdN( int dice, int sides ) {
   int roll;

   if ( ( dice > 0 ) && ( sides > 0 ) ) {
      while ( dice ) {
         roll += 1 + random( sides );
         dice--;
      }
   }
   return roll;
} /* roll_MdN() */

string dump_socket_status() {
    string ret;
    string *finalsocks, *sock_array = ({});
    int i, quant = sizeof(socket_status());
    for(i = 0; i < quant; i++){
        sock_array += ({ socket_status(i) });
    }
    finalsocks = sock_array;

    ret = @END
Fd    State      Mode       Local Address          Remote Address
--  ---------  --------  ---------------------  ---------------------
END;

    foreach (mixed *item in finalsocks) {
        int memb = member_array(item, finalsocks);
        ret += sprintf("%2d  %|9s  %|8s  %-21s  %-21s\n", memb, item[1], item[2], item[3], item[4]);
    }

    return ret;
}

string base_name(object ob) {
    string file, tmp;

    seteuid(geteuid(previous_object()));
    if(sscanf(file_name(ob), "%s#%s", file, tmp) != 2) file = file_name(ob);
    seteuid(0);
    return file;
}

varargs string identify( mixed a )
{
    int i, s;
    string ret;
    mapping RealMap;

    if( undefinedp( a ) ) return "UNDEFINED";
    if( nullp( a ) ) return "0";
    if( intp( a ) ) return "" + a;
    if( floatp( a ) ) return "" + a;
    if( objectp( a ) )
    {
        if( ret = a-> GetKeyName() ) ret += " ";
        else ret = "";
        return "OBJ(" + ret + file_name( a ) + ")";
    }
    if( stringp( a ) )
    {
        a = replace_string( a, "\"", "\\\"" );
        a = "\"" + a + "\"";
        a = replace_string( a, "\\", "\\\\" );
        a = replace_string( a, "\\\"", "\"" );
        a = replace_string( a, "\n", "\\n" );
        a = replace_string( a, "\t", "\\t" );
        return a;
    }
    if(classp(a)){
        ret = replace_string(sprintf("%O",a),"\n","");
        return ret;
    }
    if( pointerp( a ) ) 
    {
        ret = "({ ";
        s = sizeof( a );
        for( i = 0 ; i < s ; i++ )
        {
            if( i ) ret += ", ";
            ret += identify( a[i] );
        }
        return ret + ( s ? " " : "" ) + "})";
    }
    if( mapp( a ) )
    {
        ret = "([ ";
        RealMap = (mapping)(a);
        a = keys( RealMap );
        s = sizeof( a );
        for( i = 0 ; i < s ; i++ )
        {
            if( i ) ret += ", ";
            ret += identify( a[i] ) + " : " + identify( RealMap[a[i]] );
        }
        return ret + ( s ? " " : "" ) + "])";
    }
    if(functionp(a)) return sprintf("%O", a);
    return "UNKNOWN";
}

varargs string get_stack( int x) {
    int i, s;
    string list = "";
    string *stack0 = call_stack(0);
    string *stack1 = call_stack(1);
    string *stack2 = call_stack(2);
    for(i = 0, s = sizeof(stack1); i < s; i++){
        list +="\n"+i+":"+identify(stack2[i])+"."+identify(stack1[i])+"."+identify(stack2[i]);
    }

    if(x){
        list += "\n"+ identify(previous_object(-1));
    }

    return list;
}

void tc(string mess){
    object crat = find_player("god");
    string sauce = base_name((previous_object() || this_object()));
    if(crat) tell_object(crat, sauce +": "+mess+"\n");
    //if(crat) crat->receive_private_msg(sauce+": "+mess, PRIVATE_MSG);
    debug_message(sauce +": "+mess);
    flush_messages();
}

varargs string extract(string str, int i, int j);

mixed query_strange_inventory(mixed arr) {
    mixed inv;
    object ob;
    mixed desc;
    int i, k;

    inv = ({ });
    for (k=0;k < sizeof(arr); k++) {
    ob = arr[k];
    if (!(desc = (string)ob->short()) || (desc == ""))
        continue;
    if ((i = member_array(desc, inv)) >= 0)
        inv[i+1] += ({ ob });
    else
        inv += ({ desc, ({ ob }) });
    }
    return inv;
}

mixed *delete(mixed *arr, int start, int len) {
    return arr[0..start-1]+arr[start+len..sizeof(arr)];
} /* delete() */

mixed *slice_array(mixed *arr, int start, int fin) {
    return arr[start..fin];
} /* slice_array() */

mixed *insert(mixed *arr, mixed el, int pos) {
    return arr[0..pos-1]+({ el })+arr[pos..sizeof(arr)];
} /* insert() */

varargs mapping filter_mapping(mapping map, string func, mixed ob, mixed extra)
{
    mixed *bing;
    mapping ret;
    int i;

    ret = ([ ]);
    bing = keys(map);
    for (i=0;i<sizeof(bing);i++) {
    if (call_other(ob, func, map[bing[i]], extra))
        ret[bing[i]] = map[bing[i]];
    }
    return ret;
} /* filter_mapping() */

// Added by Taniwha 1995 efun one didn't
// #include "/secure/simul_efun/pluralize.c"
// added by Baldrick. Dummy for missing efun in new driver.
// Maybe we should remove it again ? not sure, we have a fix (aragorn)

/* Nativeness.. Baldrick. */
#include "/secure/simul_efun/event.c"

#include "/secure/simul_efun/add_a.c"
#include "/secure/simul_efun/log_file.c"
#include "/secure/simul_efun/find_match.c"
#include "/secure/simul_efun/m_delete.c"
#include "/secure/simul_efun/m_indices.c"
#include "/secure/simul_efun/m_sizeof.c"
#include "/secure/simul_efun/m_values.c"
#include "/secure/simul_efun/mappingp.c"
#include "/secure/simul_efun/modified_efuns.c"
// Baldrick.
#include "/secure/simul_efun/notify_fail.c"
#include "/secure/simul_efun/query_ident.c"
#include "/secure/simul_efun/query_number.c"
#include "/secure/simul_efun/replace.c"

// Roll.c, a dicer made by Sojan.
#include "/secure/simul_efun/roll.c"
#include "/secure/simul_efun/stat_string.c"

#include "/secure/simul_efun/shout.c"
#include "/secure/simul_efun/vowel.c"
#include "/secure/simul_efun/write.c"
#include "/secure/simul_efun/multiple_short.c"
#include "/secure/simul_efun/virtual.c"
#include "/secure/simul_efun/snoop_simul.c"
#include "/secure/simul_efun/extract.c"
#include "/secure/simul_efun/minimax.c"
#include "/secure/simul_efun/file_exists.c"

#include "/secure/simul_efun/user_exists.c"
#include "/secure/simul_efun/immortal_exists.c"
//#include "/secure/simul_efun/base_name.c"
#include "/secure/simul_efun/arrange_string.c"
#include "/secure/simul_efun/format_page.c"

#include "/secure/simul_efun/pretty_time.c"
#include "/secure/simul_efun/wrap.c"

// Added by Radix
#include "/secure/simul_efun/users.c"
//#include "/secure/simul_efun/file_commands.c"
#include "/secure/simul_efun/files_obj.c"

#include "/secure/simul_efun/uniq_array.c"
#include "/secure/simul_efun/exclude_array.c"
#include "/secure/simul_efun/atoi.c"

// Addedd 8 Nov 93 Chrisy for Driver upgrade to 0.9.18.9
#include "/secure/simul_efun/mudos.c"
// Taniwha 1995, we can use a checked version
#include "/secure/simul_efun/member_array.c"

/* Added feb '95, Baldrick. */
#include "/secure/simul_efun/log_attack.c"

#include "/secure/simul_efun/reload_object.c"
// Added March 96, andy@entropy.demon.co.uk
// security thingy.

/* Raskolnikov */
#include "/secure/simul_efun/children.c"

/* Upgrade to v22b22, Baldrick.  */
#include "/secure/simul_efun/process_string.c";
/* More fun, Baldrick dec '96 */
#include "/secure/simul_efun/mud_long_name.c";
#include "/secure/simul_efun/nice_list.c"
#include "/secure/simul_efun/cpdir.c"

/* Malik */
#include "/secure/simul_efun/secure_log_file.c"

// save_object, hacked by Baldrick.
//#include "/secure/simul_efun/save_object.c"

/* Tail.c, needed after the v22.1 upgrade, Baldrick. */
#include "/secure/simul_efun/tail.c"

//#include "/secure/simul_efun/command.c"
