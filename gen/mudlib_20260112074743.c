Here is an example of how you could create a base room object in LPC with the features you mentioned:
```
// Room.h

#ifndef ROOM_H
#define ROOM_H

// Exit system
typedef enum { NORTH, SOUTH, EAST, WEST } Exit;

// Item descriptions
typedef struct {
    const char* name;
    const char* desc;
} ItemDesc;

// Player enter/exit messages
typedef struct {
    const char* message;
} PlayerEnterExitMessage;

// Room object
struct Room {
    Exit exit[4]; // North, South, East, West exits
    ItemDesc items[10]; // Item descriptions
    PlayerEnterExitMessage player_messages[2]; // Player enter/exit messages
};

#endif // ROOM_H
```

```
// Room.c

#include "Room.h"

// Exit system
typedef enum { NORTH, SOUTH, EAST, WEST } Exit;

// Item descriptions
typedef struct {
    const char* name;
    const char* desc;
} ItemDesc;

// Player enter/exit messages
typedef struct {
    const char* message;
} PlayerEnterExitMessage;

// Room object
struct Room {
    Exit exit[4]; // North, South, East, West exits
    ItemDesc items[10]; // Item descriptions
    PlayerEnterExitMessage player_messages[2]; // Player enter/exit messages
};

Room* create_room(void) {
    Room* room = malloc(sizeof(Room));
    room->exit = { NORTH, SOUTH, EAST, WEST };
    room->items = { { "Sword", "A sword for fighting" }, { "Shield", "A shield for protection" } };
    room->player_messages = { { "Welcome to the room!", NULL }, { "Thank you for visiting!", NULL } };
    return room;
}

void room_exit(Room* room, Exit exit) {
    // Handle exit logic here
}

void room_item_desc(Room* room, ItemDesc item) {
    // Handle item description logic here
}

void room_player_message(Room* room, PlayerEnterExitMessage message) {
    // Handle player enter/exit message logic here
}
```

In this example, we define a `Room` struct that contains an array of `Exit` pointers, an array of `ItemDesc` structs, and an array of `PlayerEnterExitMessage` structs. We also provide functions for creating a new room object, handling exits, handling item descriptions, and handling player enter/exit messages.

The `create_room()` function creates a new room object and sets its exit system, item descriptions, and player enter/exit messages. The `room_exit()` function handles logic for when the player exits the room through one of the four exits, while the `room_item_desc()` function handles logic for when an item is described to the player. Finally, the `room_player_message()` function handles logic for when the player enters or exits the room.

Note that this is just a basic example and you may want to add more features or customize the code to fit your specific needs.