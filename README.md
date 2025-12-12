# game-rs

## TODO:

- Add story terminal
- Add first boss
- Add gameover menu
- Add kinematic damaging hazards (use enemy?)
- Restrict camera movement to map boundaries
- Add stationary chain mount point map loading
- Allow switches to be loaded in any rotation
- Allow switches to be loaded with an initial activation
- Add chain ability pickup
- Prevent infinite boost

- BUG: Culling is over-eager for cuboids in the direct corners of the screen
- BUG: Seeker enemies will slam into the wall bc the speed cap uses absolute value

- CLEANUP: Replace `HealOnCollision` with `Damager` with negative damage

## GAME DESIGN:

### Combat
- Weapons
  - Missile
    - Self-propelling, slow moving, low frequency, overall hard to use, while bieng high-damage with an explosive blast radius
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
  - Goblin
    - Shoots a slow moving projectile at the player, then moves in a random direction, then fires again once stopped.
  - Shade
    - Invulnerable while stopped, remains stopped for a bit before leaping towards the player in an arc, after which it slows down and is invulnerable again once stopped. At the apex of the arc, it fires two slow-moving shots to either side of it.
  - Spider
    - Lies dormant on a wall, with an egg separately laying away from it away from the wall. Launches towards the egg if and when the player touches it, at which point it remains in that spot and fires bursts of two quick-moving shots each at the player's location. 
- Angel automatons
  + Defender
  + Seeker

### Keys & Locks:

+ Weapon (P)
  + Destructible blocks
  + Areas with enemies
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
  
