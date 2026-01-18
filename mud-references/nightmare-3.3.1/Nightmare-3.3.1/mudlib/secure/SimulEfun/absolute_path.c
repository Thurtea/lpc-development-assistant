/*    /adm/SimulEfun/absolute_path.c
 *    from Nightmare IV
 *    returns the full path of a string based on a relative path
 *    created by Huthar@Portals as resolv_path.c
 *    modifications by Pallando@Nightmare 930526
 *    changed to absolute_path() by Descartes of Borg 940501
 *    features added and speed doubled by Pallando, 940531
 */

string absolute_path(string curr, string new) {
    int i, len;
    string *tmp;
    string name, rest;

    if(curr && (curr == "cwd") && this_player())
      curr = (string)this_player()->get_path();
    if(!new || new == "" || new == ".") return curr;
    if( (new == "here") && this_player() )
    {
        return file_name(environment(this_player())) + ".c";
    }
    len = strlen( new );
    switch( new[0..0] )
    {
        case "~":
	    if( new == "~" || new == "~/" )
		new = user_path( (string)this_player()-> query_name() )[0..-2];
            else if( new[1..1] == "/" )
		new = user_path( (string)this_player()-> query_name() ) +
		  new[2..len];
            else if( sscanf( new, "~%s/%s", name, rest ) == 2 )
		new = user_path( name ) + rest;
	    else
		new = user_path( new[1..len] )[0..-2];
	    break;
        case "^":
	    new = "/domains/" + new[1..len];
	    break;
	case "/":
	    break;
	default:
	    new = curr + "/" + new;
    }

    if( -1 == strsrch( new, ".." ) ) return new;

    if(new[strlen(new) - 1] != '/')
        new += "/";
    tmp = explode(new,"/");
    if (!tmp) tmp = ({"/"});
    for(i = 0; i < sizeof(tmp); i++)
        if(tmp[i] == "..") {
            if(sizeof(tmp) > 2) {
                tmp = tmp[0..(i-2)] + tmp[(i+1)..(sizeof(tmp)-1)];
                i -= 2;
            } else {
                tmp = tmp[2 ..(sizeof(tmp)-1)];
                i = 0;
            }
        }
     new = "/" + implode(tmp,"/");
     if(new == "//") new = "/";
     return new;
}
