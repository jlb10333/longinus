# Longinus Master Plan
## Gameplay
- Mechanical prototype v2
  - Graphics
    - All menus
    - Misc
      - Mana tanks
      - Status effects
      - UI elements
      - Damaging tiles
      - Locomotor
      - Ability pickup
      - Chain
      - Boost?
  - Core elements
    - 2nd boss
  - Full blocking
    - Add level layout leading to 2nd boss, accessible via chain
      - Add a section where you have to navigate through a tight corrider of damaging tiles
    - Settle on distribution and selection of upgrade modules/mana bars/health bars
  - Polish
    - Bugs
      - Fix bug causing menu update from item acquisition to not be consumed by physics
      - Fix player movement (normalize diagonal)
    - Visual explainability
      - Show all health tank and mana tanks in inventory
  - Testing
    - Test at different frame rates
  
- Mechanical prototype v3
  - Intro cutscene
    - Overhead view of the solar system
    - Zooms into where earth would be to see that there's a very long and thin object in its place, along with a bunch of scattered asteroids and dust
    - Zooms in further on the long and thin object, but specifically on one of its ends, 
    - We zoom in past the structure to finally halt on our player character in an enclosed larger vessel against the backdrop of space, zooming stars 
    - Player character is movable in this scene but is unable to break out of the enclosure
    - After watching the larger vessel zoom past stars for a while, it slams into the side of another, much larger structure (presumably the long and thin object from before), throwing the player object around inside
    - Doors on the side of the larger vessel open up and allow the player to access the structure, and the game begins
  - Enemies
    - Time slow effect on killing an enemy
  - Ability
    - Ability that stops all momentum and reflects any objects in range
      - Costs mana proportional to the total MOMENTUM of objects reflected with a baseline 
    - Ability similar to metroid 2's spider ball (stick to and roll along walls)
  - Modules
    - Add upgrades that affect projectile speed
- Messy experiments in variety
  - Come up with a list of places where variety(!!) emerges from
  - For each one, develop a small prototype
  - Playtest them
  - Come up with a strategy for how to make the rest of the game
- Activation
  - Cannon thing
    - Explodes, damages player and enemies
    - Triggers when reaching > 0.5 activation
  - Magnet balls
  - Crank
  - Min
  - Max
  - Saving/loading of activation levels
- Exploration game loop
  - Combat
    - Modules
      - LASE Laser
      - Status effects
        - 3XPL ExplosionStatus
        - BL3D BleedStatus
        - P4RA ParalyzeStatus
      - STPT StatusPotency (Increases the amount by which statuses are applied)
      - STDR StatusDuration (Increases duration of applied statuses)
      - B1GR Bigger (Increases size of projectiles)
      - M4NF ManaFree (damagefree, manafree)
      - G4TL Gatling (Increases frequency of weapon drastically but makes it less accurate)
      - MDET MutualDeterioration (Applies deterioration status, selfinflict deterioration)
      - S31V Seive (Increases frequency if weapon has selfinflict)
      - H4MR Hammer (Increases damage drastically if it is manafree)
    - Shield 
  - Non-combat modules
    - All mana bars are rechargeable, but recharge speed is reduced
    - Mana recharge speed is increased
    - Gain mana passively from damaging enemies
    - Increase mana drop rates from enemeis
    - Increase boost force
    - Reduce boost mana usage
    - Make chain deal damage to enemies
    - Increase defensive power
    - Increase damage taken but also received
  - Water Physics (maybe)
  - Time-slow zones?
### Squash bugs
- Culling is over-eager for cuboids in the direct corners of the screen
- Seeker enemies will slam into the wall bc the speed cap uses absolute value
### Level design
- Research/figure out a sustainable loop for level design
- Figure out what level designs allow for modules to feel beneficial
### Playtesting
- Collect feedback from playtesters
- Refine feedback into actionable items
## Writing
- What's the story behind the player character?
- Basic outline for each cyst layer expedition
## Art
## Sound
### Music 
### SFX
