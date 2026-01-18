		     The Nightmare Object Library
		     Version 4.6 for MudOS v22b25
		 Copyright (c) 1992-1997 George Reese
			    3 January 1997

Version 4.6 is a beta release.  Going by the new Nightmare Object
Library naming standard (Nightmare IVr2.1 was the first release to go
by this new standard), this means it is the sixth beta for Nightmare V.

What does it mean that it is a beta release?  It means the
following:

	* Documentation is incomplete, and in some cases, missing
	* There may be some undiscovered mudlib bugs
	* The MudOS driver being used is in beta, and thus unstable at
	  times.

All documentation, including installation instructions, are available
solely on the web at http://www.imaginary.com/LPC/Nightmare.

The examples in /domains/Ylsrim, while still incomplete, work 100% and
are the definitive way to perform the tasks they are meant to show.
The major exception is the virtual rooms examples which are not yet
functional. 

There were no alpha releases between Nightmare IVr5 and Nightmare
IVr6, meaning that no new features were added for this release.
Instead, it represents a fixing of many bugs, cleaning of code, work
on examples and documentation.

Specific in Nightmare IVr6
	* New soul daemon with new soul adding commands
	  This system is quite complex and the interface to it is not
	  yet user friendly.  However, it is very powerful and works
	  better than the old system.  See 'help messaging' and
	  'help addemote' for more info.
	* Cleaned up a lot of code and BEGAN a restructuring of /lib
	  to be easier to navigate.  Note that this LOOKS a lot different
	  from Nightmare IVr5, but it really is not.  All of the moved
	  objects have been rewritten for ease of reading.  The ones
	  not yet moved will be cleaned up and moved before the 
	  final release.
	* Added ucache and targetted emotes for Intermud.
	* Added support for sending souls over channels
	* Fixed all known bugs.  There are no more known bugs.  I am
          sure, however, some are still lurking to be found.
	
Fixes for Nightmare IVr5:
	* COMPAT BUSTER: enters, touches, smells, reads, listens, and
	  searches MUST all be described as items BEFORE setting the
	  enter, touch, smell, read, listen, or search.
	* COMPAT BUSTER: climbs and jumps may no longer be included in
	  room code. They must be done as separate objects.  See Ylsrim
	  examples.
	* Completely redid combat.  No compat buster here.  Just easier
	  to read, a bit more efficient, and a lot more consistent.
	* Moved undead support to a separate inheritable.
	* Added a complex system for spell casting.
	* Added a messaging module for sending messages to multiple
	  targets.  See 'help messaging'.

Features added between Nightmare IVr2.1 and Nightmare IVr3
	* Created a new dynamic class and race management system
	* Added support for positions (standing, lying, sitting)
	* Created a new history.c object inherited by nmsh.c
	* Created a new lighting system.  In spite of the fact that this
	is very different from the old lighting system, old rooms will
	work properly with a backwards compat hack at the end of room.c.
	If you are starting from scratch, you can remove the SetProperty()
	method from room.c entirely.
	* Added some new inheritables for the new light system,
	light.c (no relation to old light.c), torch.c, fuel.c, burn.c,
	match.c, and lamp.c COMPAT BUSTER: Old torches will no longer work.
	* Added verb, help, press, close, lock, holder inheritables
	* Added a notifcation daemon, for notifying cres of changes
	* Added open and close verbs, modified lock and unlock verbs
	* Added new rules for the look verb
	* Added support for multiple-character/human tracking (links)
	* A brand new room maker
	* Made creating a monster with a yet undefined race easier
	* Fixed poisoning
	* Added invisibility support
	* Advanced, Zork style command parsing
	* Allowing all objects to have the same items/smells/touches/etc
	  interface as rooms
	* Bank, post office, digging, fishing, follow/lead, money pile,
	  scroll, and class leader inheritables
	* Removed AMCP support in favour of a new messaging system
	* New pager object
	
For support issues, Nightmare V has the following support mechanisms:
	http://www.imaginary.com/LPC/Nightmare/
	the mudlib.nightmare newsgroup at the Idea Exchange
		(telnet://ie.imaginary.com 7890)	
	the nightmare-mudlib mailing list 

To subscribe to the nightmare-mudlib mailing list, send mail to
majordomo@imaginary.com with:
	subscribe nightmare-mudlib

George Reese (Descartes of Borg)
borg@imaginary.com
8 October 1996
