# game-rs

## TODO:

- Add story terminal
- Add first boss
- Add gameover menu
- Add kinematic damaging hazards (use enemy?)
- Add chain ability pickup
- Add max health increase pickup 
- Add map/minimap

- Allow switches to be loaded in any rotation
- Allow switches to be loaded with an initial activation

- Restrict camera movement to map boundaries
- Prevent infinite boost

- BUG: Culling is over-eager for cuboids in the direct corners of the screen
- BUG: Seeker enemies will slam into the wall bc the speed cap uses absolute value

- CLEANUP: Replace `HealOnCollision` with `Damager` with negative damage

## GAME DESIGN:

### Scenarios
- Two engines active at opposite periods. When one is at 1.0, the other is at 0.0, and vice versa. An And activatable on the two engines prevents the player from progressing. The player must stop one of the engines, and restart it in sync with the other engine so that the combined signal will cause the and to reach 1.0 during the period.
- An engine is giving activation to a rotator, which has 4 lasers attached at 90 degree angles. The player must disable and enable the rotator via a switch, until the laser reaches the desired path.

- The player is in a big dyson-sphere like ring area with strong gravity pushing outwards. The player must activate a locomotor with a mount point attached, in order to mount onto the point and ride it as an elevator towards the center of the ring.

- Player has to tug a magnet ball and launch it past a laser, meeting another ball to create an activation which disables the laser and allows the player to proceed.
- Player has to activate an engine which activates a gravity source which pushes a switch away from its negative end in a bouncing motion, not quite enough force to make it past the center even at the height of the gravity source activation. The player must grab onto the switch's mount point, the zone of which just barely crosses over a laser at the height of the gravity source activation.

- The player passes through an area where they trigger a touch sensor which is gated, and the resulting activation causes a gate to swing shut behind them.

### Activation
- Interactable
  + Switch 
    (Prismatic joint with motor pulling towards the nearest end)
    + Activator
      + Emits activation level corresponding to position of the main knob w/r/t the limits of the joint
      + 0.0 at one end, 1.0 at the other, LERPed in between
  - Crank
    (Revolute joint)
    - Activator
      - Emits activation corresponding to rotation of the knob w/r/t te limits of the joint
      - 0.0 at one end, 1.0 at the other, LERPed in between 
  + Touch Sensor
    (Fixed sensor collider)
    + Activator
      + Emits activation level depending on whether or not an entity is colliding with the sensor
      + 0.0 if no collision, 1.0 if collision
  - Magnet
    (Physics interactable balls; they attract each other)
    - Activator
      - Emits activation if touching another magnet
      - 0.0 if not touching, 1.0 if touching

- Output Objects
  - Gravity Source
    (Point that emits gravitational force at given radius and intensity following the inverse square law)
    - Activatable
      - 0-1 sources
      - Sets gravity intensity to activation lerped betwen min and max intensity
  + Locomotor
    (Prismatic joint which can have other objects attached to it via glue, e.g. gravity sources, mount points, doors)
    + Activatable
      + 1 source
      + Sets motor target position equal to activation level lerped between joint limits
  - Rotator
    (Revolute joint which can have other objects attached to it via glue, e.g. colliders)
    - Activatable
      - 1 source
      - Sets motor target position equal to activation level lerped between joint limits
  - Laser
    (Constant damaging sensor collider proceeding in a straight line from a given point)
    - Activatable
      - 0-1 sources
      - Sets laser intensity to activation lerped between min and max intensity

- Logic
  - Not
    - Activatable
      - 1 source
    - Activator
      - Emits activation level of 1.0 - received activation
  + And
    + Activatable
      + 2 sources
    + Activator
      + Emits activation level of (0.5 * A)
  + Or
    + Activatable
      + 2 sources
    + Activator
      + Emits activation level of (MAX(A, 1.0))
  + Gate
    + Activatable
      + 1 source
    + Activator
      + Emits activation at level corresponding to highest historical activation received
  - Flat
    - Activatable
      - 1 source
    - Activator
      - Emits activation at level equal to 1.0 if A > 0.5, otherwise 0.0
  - Engine
    - Activatable
      - 1 source
    - Emits activation at a level that oscillates back and forth between 0.0 and 1.0 in a sinusoidal pattern
    - Oscillates slower or faster depending on received activation level


### Combat
- Weapons
  + Missile
    + Self-propelling, slow moving, low frequency, overall hard to use, while bieng high-damage with an explosive blast radius
  - Laser
    - Very fast-moving, almost hitscan, as well as being high frequency, while being low-damage
- Modules
  + Slots
    + Front two
    + 45 degree
    + Side
    + Reverse
  - Status Effects (all build up an invisible bar before taking effect, like elden ring)
    - Slow
    - Paralysis
    - Decay
    - Bleed
    - Stop
    - Explode
  - Misc
    + Double damage, .75 freq
    + Double freq, .75 damage
    - Enemy-seeking
    - Pierce (pass through enemies/walls)

### Enemies
- Dragonspawn
  + Goblin
    + Shoots a slow moving projectile at the player, then moves in a random direction, then fires again once stopped.
  - Shade
    - Invulnerable while stopped, remains stopped for a bit before leaping towards the player in an arc, after which it slows down and is invulnerable again once stopped. At the apex of the arc, it fires two slow-moving shots to either side of it.
  - Spider
    - Lies dormant on a wall, with an egg separately laying away from it away from the wall. Launches towards the egg if and when the player touches it, at which point it remains in that spot and fires bursts of two quick-moving shots each at the player's . 
- Angel automatons
  + Defender
  + Seeker

### Keys & Locks:

+ Weapon (P)
  + Destructible blocks
  + Areas with enemies
- Weapon (L)
  - High gravity sections where you have to hit a target across a gap with a projectile and everything except the laser can't make it far enough
+ Boost
  + Gravity sources
    + Gravity walls you have to boost through
    + High gravity sections with gaps that you have to boost jump to get over
- Shield
  - Lock-on stun weapon emplacements
  - Fixed lasers (shield blocks the lasers' path and allows the player to proceed undamaged)
    - Cranks with part of their path blocked by a laser, so you need to shield in order to turn the crank
+ Chain
  - Cranks (grab on with chain, spin around to make something in the environment move)
    - High gravity sections where you have to boost against gravity to get around the crank
  + Switches (grab on with chain, pull in direction to flip switch)
    + (can be used to create one-way gates, or areas where the player has to go around a separate path to hit a switch from a different angle in order to progress through the main path)
  + High gravity sections where you have to swing on the chain to get across a gap
  + High gravity sections where you have to hang on a kinematic elevator to travel vertically

## STORY:

### Key Terms:

- Longinus
  - Setting of the entire game. Incomprehensively massive oblong megastructure which suddenly materialized at the edge of the solar system and punched through the Earth with little warning without taking any observable damage to its hull. causing the End of the World. Afterwards the structure has laid dormant on a significant orbit with a period of 43 years. It has become the subject of worship in the post-Earth society.
  - Longinus was originally manufactured by a race of alien creatures which are speculatively referred to as the Angels with the intent to spread the seeds of life to uninhabited planets across the galaxy. The Angels inhabitting the ship were attacked and wiped out after visiting a world which they failed to realize was already populated by Dragons. Having no knowledge of how to operate the ship and possessing animalistic intelligence, the Dragons populating the ship did not adjust its previously-entered course - which unfortunately happened to be a direct collision with Earth, another planet they did not know to be inhabited.
  - Longinus contains a sentient ship-board control system, referred to as the Spirit. 

- The End of the World
  - TODO
  
- Post-Earth Society
  - TODO
  
- Angels
  - TODO

- Dragons
  - TODO

- The Spirit
  - TODO

- Cyst Layer
  - When Longinus passes close to the Sun on its massive orbital trajectory, the end of the structure closest to the Sun is torn open by the intense radiation, allowing humans to enter. During its 43-year period spent almost entirely far away from the Sun, a cyst-like growth emerges to cover the gap, forming a new layer over the previous cyst. This growth is undeniably the work of Dragons.
  
