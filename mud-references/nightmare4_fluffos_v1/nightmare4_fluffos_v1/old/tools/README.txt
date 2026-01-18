			    CreRemote 1.0
		   A remote editing tool for LPMuds
		   Copyright (c) 1995 George Reese

This directory contains two clients for the RCP protocol supported by
Foundation.  One is a text-based client for UNIX, and the other is a
16-bit Windows application.  Since there is currently no ftp daemon
available for Foundation II, CreRemote is the only way to edit MUD
files offline unless you have shell access to the MUD.  It just so
happens, however, that CreRemote is a much better way to do it.

CreRemote allows a creator to access their home directory on a MUD as
if it were local to their own machine.  The Windows version gives a
graphical, file manager-like presentation of the creator's home
directory and allows the creator to edit using an LPC-smart editor.
The UNIX version allows the creator to use basically whichever editr
they like.  Both version allow for a limited number of file commands
to be issued, like, rm, mv, cp, update, etc.

Remember to tell all users that CreRemote wants to know the RCP
port!!!!  Both version ask for a port number.  A common mistake is to
give the MUD port number.  By default, the RCP port is 10 below the
MUD port number.  This will change only if you change it.  Thus if
your MUD is on port 4000, then the RCP port is 3990.

A new version of CreRemote will likely be released in the next two
weeks.

George Reese
borg@imaginary.com
15 September 1995

