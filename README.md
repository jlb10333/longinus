# Longinus

## Default Controls

- Move: Left Stick
- Aim: Right Stick
- Shoot: Right Trigger
- Boost: Left Trigger
- Chain: Left Shoulder
- Inventory: X
- Pause: Y
- Confirm: A
- Cancel: B
- Menu Movement: D-Pad

## Important Callouts

- You can ONLY play with a controller right now (sorry). I have only tested this on Windows with a Switch Pro Controller (because I don't have any other controllers lol). Theoretically the library I used for gamepad input should work for most any controller, but I am somewhat skeptical since it took a while to get it to work wit hthe pro controller. Hopefully other controllers are easier though idk
- All visual presentation is subject to change, as it's basically just colorized colliders and debug text right now lol
- There's some stuff that is not properly communicated now, so I need to outline it here (hopefully this will be done better in v.0.1.0)
  - When you get the item in the first room, this is a weapon which you can equip on the inventory screen (as mentioned above, the binding is X)
  - On the inventory screen, the 4x4 grid you see on the left hand side is the area in which modules are equipped. The area to the right is for unequipped/extra modules. You can move modules back and forth between the sections, and also move them around within the sections, by picking up the modules with confirm and setting them down in a new space with confirm again
  - Blocks marked as "D" are destructible, and blocks marked as "H" will deal damage to you (and to enemies!)
  - When you get any further items which show up in the inventory and have a black dot on one of the sides, these are modules which can modify other modules, and the black dots are their attachment points. Modules that are attached to weapon modules (or to other modules that are transitively attached to a weapon) will modify that weapon with that module's effect
  - When you get to the point where you touch a sensor that activates gravity, this is also where you pick up the boost ability - you can press the left trigger to boost away from the gravity source
  - When you get to the point where you touch a sensor that causes gravity to activate and deactivate in a repeating oscillation, this is where you pick up the chain ability - you can press the left shoulder button to activate the chain when you are in range of a mount point/switch (the range is indicated by the translucent circle around each mount point). Pressing the left shoulder button deactivates the chain

Enjoy the prototype, and thank you for playing!
