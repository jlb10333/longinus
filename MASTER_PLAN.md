# Longinus Master Plan
## Gameplay
- Mechanical prototype
  - Visual explainability
    - Show boost cooldown (maybe)
  - Get it playtested
- Mechanical prototype v2
  - Visual explainability
    - Show all health tank and mana tanks in inventory
    - Show mana levels better
  - Map view (maybe)
  - Health tank pickup
  - First boss
  - Mana
    - Unrechargeable bars can be replenished with enemy drops
    - Both rechargeable and unrechargeable bars can be found in environment
  - Activation
    - Magnet balls
    - Crank
    - Not
    - Min
    - Max
    - Lasers
    - Saving/loading of activation levels
- Messy experiments in variety
  - Come up with a list of places where variety(!!) emerges from
  - For each one, develop a small prototype
  - Playtest them
  - Come up with a strategy for how to make the rest of the game
- Exploration game loop
  - Combat
    - Modules
      - LASE Laser
      - Status effects
        - 3XPL ExplosionStatus
        - BL3D BleedStatus
        - P4RA ParalyzeStatus
        - VLNR VulnerableStatus
        - W3KR WeakerStatus
      - STPT StatusPotency (Increases the amount by which statuses are applied)
      - STDR StatusDuration (Increases duration of applied statuses)
      - B1GR Bigger (Increases size of projectiles)
      - M4NC ManaCost (Increases damage of weapon drastically but makes it consume mana)
      - M4NF ManaFree (Makes weapon no longer consume mana, but also deal no damage)
      - G4TL Gatling (Increases power of weapon drastically but makes it less accurate)
    - Shield 
  - Non-combat modules
    - All mana bars are rechargeable, but recharge speed is reduced
    - Recharge speed is increased, but no mana bars are rechargeable
    - Increase boost force
    - Reduce boost mana usage
    - Make chain deal damage to enemies
    - Increase defensive power
    - Increase damage taken but also received
  - Water Physics (maybe)
### Squash bugs
- Fix loading of initial activation of switches/locomotors
- Chain can be activated outside of the mount radius
- Culling is over-eager for cuboids in the direct corners of the screen
- Seeker enemies will slam into the wall bc the speed cap uses absolute value
### Level design
- Research/figure out a sustainable loop for level design
- Figure out what level designs allow for modules to feel beneficial
### Playtesting
- Collect feedback from playtesters
- Refine feedback into actionable items
## Writing
- Why does the Spirit know about the erasure from history?
- What's the story behind the player character?
- Basic outline for each cyst layer expedition
## Art
## Sound
### Music 
### SFX
