# game-rs

## TODO:

- Add story terminal
- Add first boss
- Add gameover menu
- Add kinematic damaging hazards
- Add ability pickup
- Restrict camera movement to map boundaries

- BUG: Seeker enemies will slam into the wall bc the speed cap uses absolute value

- CLEANUP: Replace `HealOnCollision` with `Damager` with negative damage

- IDEA: Add human-readable label to entities/sensors, use to draw labels in debug mode

## GAME DESIGN:

### Keys & Locks:

+ Weapon (P)
  + Destructible blocks
  + Areas with enemies
+ Boost
  + Gravity repulsors
- Shield
  - Lock-on stun weapon emplacements
  - Fixed lasers (shield blocks the lasers' path and allows the player to proceed undamaged)
- Chain
  - Cranks (grab on with chain, spin around to make something in the environment move)

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
  
