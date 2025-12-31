## GAME DESIGN:

### Tasks to Accomplish
- Communicate that exploration is valuable/necessary
- Communicate that backtracking is valuable/necessary
- Introduce boost
- Provide a space to practice boost
- Provide a space to demonstrate mastery of boost
- Introduce chain
- Introduce activation
- Create a frighteningly intense atmosphere
- Create a frighteningly desolate atmosphere
  - ideas
    - petrified forest
      - off-white grey
      - enemies exist, but as petrified husks which crumble when touched



### Scenarios
- Two engines active at opposite periods. When one is at 1.0, the other is at 0.0, and vice versa. An And activatable on the two engines prevents the player from progressing. The player must stop one of the engines, and restart it in sync with the other engine so that the combined signal will cause the and to reach 1.0 during the period.
- An engine is giving activation to a rotator, which has 4 lasers attached at 90 degree angles. The player must disable and enable the rotator via a switch, until the laser reaches the desired path.

- The player is in a big dyson-sphere like ring area with strong gravity pushing outwards. The player must activate a locomotor with a mount point attached, in order to mount onto the point and ride it as an elevator towards the center of the ring.

- Player has to tug a magnet ball and launch it past a laser, meeting another ball to create an activation which disables the laser and allows the player to proceed.
- Player has to activate an engine which activates a gravity source which pushes a switch away from its negative end in a bouncing motion, not quite enough force to make it past the center even at the height of the gravity source activation. The player must grab onto the switch's mount point, the zone of which just barely crosses over a laser at the height of the gravity source activation.

+ The player passes through an area where they trigger a touch sensor which is gated, and the resulting activation causes a gate to swing shut behind them.

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
  - Min
    - Activatable
      - 1 source
    - Activator
      - Emits activation at level equal to (MIN(A, M)), where A is incoming activation and M is the given threshold
  - Max
    - Activatable
      - 1 source
    - Activator
      - Emits activation at level equal to (MAX(A, M)), where A is incoming activation and M is the given threshold
  + Engine
    + Activatable
      + 1 source
    + Emits activation at a level that oscillates back and forth between 0.0 and 1.0 in a sinusoidal pattern
    + Oscillates slower or faster depending on received activation level


### Combat
- Weapons
  + Missile
    + Self-propelling, slow moving, low frequency, overall hard to use, while being high-damage with an explosive blast radius
  - Laser
    - Very fast-moving, almost hitscan, as well as being high frequency, while being low-damage
  - Railgun
    - Very fast-moving, almost hitscan, but very very low frequency and extremely high-damage
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

