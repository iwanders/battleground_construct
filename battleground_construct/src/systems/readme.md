# Systems


## Projectiles...
On projectiles, their flow is roughly;
```
  Projectile gets created and has:
    - Projectile
    - DamageHit
    - DamageSplash
    - Source <- Must be permanent!
    - BulletTrail

  System projectile_hit.
    Strips projectile, velocity, acceleration, any mesh
    Applies Impact.

    #- Projectile
    - DamageHit (vectorfunction?)
    - DamageSplash (vectorfunction?)
    - Source
    - Impact (describes entity impacted, at what velocity & position).
    New entity with...
      - damagehit effect?
      - damagesplash effect?
      - copy BulletTrail emitter?

  System impact_handler.
    Uses Impact.
      For DamageHit, uses Impact & entity
        Modifies HitBy:
          Adds (DamageHit, Impact, Source)
      for DamageSplash, uses Impact & entity:
        Modifies HitBy:
          Adds (DamageSplash, Impact, Source)

  system hit_by.
    Updates health.
    Moves past hits into HitHistory
    Record statistics...
    Empties HitBy.

  system health_check.
    Checks if healthy, if not, apply destroyed component.

  destroyer.
    Uses HitHistory to create appropriate Deconstructor effect
    Removes the group (save for root?)
```