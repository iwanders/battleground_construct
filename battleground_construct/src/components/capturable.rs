use crate::components::team::TeamId;
use engine::prelude::*;

/// Only a single team can have a capture point, another team must first fully reduce the current
/// owner's strength, before ownership flips and the new owner can start increasing its strength.
#[derive(Copy, Debug, Clone)]
pub struct Capturable {
    /// Which team currently owns this capture point, empty if no team owns it.
    owner: Option<TeamId>,
    /// The strength at which the team owns this point. Moves between 0.0 and 1.0.
    strength: f32,
}
impl Default for Capturable {
    fn default() -> Self {
        Capturable::new()
    }
}

impl Capturable {
    pub fn new() -> Self {
        Capturable {
            owner: None,
            strength: 0.5,
        }
    }
    pub fn owner(&self) -> Option<TeamId> {
        self.owner
    }

    pub fn set_owner(&mut self, owner: Option<TeamId>) {
        self.owner = owner
    }

    pub fn strength(&self) -> f32 {
        self.strength
    }

    pub fn set_strength(&mut self, strength: f32) {
        self.strength = strength;
    }

    pub fn update(&mut self, contenders: &[(TeamId, f32)]) {
        let mut subtracted_strengths = contenders.to_vec();
        let start_strengths = contenders.to_vec();
        let start_value = self.strength;

        let mut summed_strength = 0.0;

        // First pass, calculate new strength.
        for (i, (team_id, value)) in contenders.iter().enumerate() {
            // Subtract all teams except itself with this strength.
            for (j, (_, other_v)) in subtracted_strengths.iter_mut().enumerate() {
                if i != j {
                    *other_v -= value;
                }
            }
            if Some(*team_id) == self.owner {
                self.strength += value;
            } else {
                self.strength -= value;
                summed_strength += value;
            }
        }
        // Now, if the strength is zero or equal to zero, the team may change.
        // The ownership will change to the strongest contender, but because all non-owning teams
        // are helping bring down the owner, we need to check if the strongest contender's value
        // exceeds the combined strength that would bring down their ownership.
        // That way the owner stays empty if there's two or more contenders of equal strength.

        if self.strength <= 0.0 {
            // Strength went below zero, or equal to zero, no owner anymore.
            self.owner = None;

            // Determine the strongest contender, this is the highest value in subtracted_strengths
            // if that is above zero, it is dominant and will claim ownership, if it is below zero
            // the owner will stay empty.

            let strongest = subtracted_strengths
                .iter()
                .enumerate()
                .max_by(|(_i, (_, a)), (_j, (_, b))| {
                    a.total_cmp(b)
                })
                .unwrap();
            let (strongest_index, (strongest_team, strongest_strength)) = strongest;
            if *strongest_strength > 0.0 {
                // Remove the contribution this team had to bringing down the flag from their
                // initial start value.
                let contribution_ratio = start_strengths[strongest_index].1 / summed_strength;
                let v = start_value / (contribution_ratio);
                // We have a new owner....
                self.owner = Some(*strongest_team);
                self.strength = strongest_strength - v;
            }
        }
        self.strength = self.strength.clamp(0.0, 1.0);
    }
}

impl Component for Capturable {}

#[cfg(test)]
mod test {
    use super::*;
    use crate::components::team::make_team_id;
    use crate::util::test_util::*;

    #[test]
    fn test_capturable() {
        let t1 = make_team_id(1);
        let t2 = make_team_id(2);
        let t3 = make_team_id(3);
        let t4 = make_team_id(4);
        let mut c = Capturable::new();
        c.set_owner(None);
        c.set_strength(0.3);
        assert!(c.owner().is_none());
        assert_eq!(c.strength(), 0.3);
        // If two non-owning teams both apply 0.1 pressure, the strength should reduce by 0.2
        c.update(&[(t1, 0.1), (t2, 0.1)]);
        approx_equal!(c.strength(), 0.1, 0.0001);

        // Another cycle of that, should reduce strength to 0, but not create a new owner.
        c.update(&[(t1, 0.1), (t2, 0.1)]);
        assert!(c.owner().is_none());
        approx_equal!(c.strength(), 0.0, 0.0001);

        // Now, if t3 arrives with a strength of 0.25, exceeding the combined strength of t1 and t2.
        // It should become owner, with 0.05 strength per update tick.
        c.update(&[(t1, 0.1), (t2, 0.1), (t3, 0.25)]);
        assert_eq!(c.owner(), Some(t3));
        approx_equal!(c.strength(), 0.05, 0.0001);

        // Next tick, t3 again gaining 0.05 after t1 and t2 reduce.
        c.update(&[(t1, 0.1), (t2, 0.1), (t3, 0.25)]);
        assert_eq!(c.owner(), Some(t3));
        approx_equal!(c.strength(), 0.10, 0.0001);

        // If t4 enters the fold, t3 does not have enough pull to exceed t1, t2 and t4 combined's
        // strength, the hold starts reducing.
        c.update(&[(t1, 0.1), (t2, 0.1), (t3, 0.25), (t4, 0.1)]);
        assert_eq!(c.owner(), Some(t3));
        approx_equal!(c.strength(), 0.05, 0.0001);

        // And with another round, the owner becomes None, keeping the flag at zero.
        c.update(&[(t1, 0.1), (t2, 0.1), (t3, 0.25), (t4, 0.1)]);
        assert!(c.owner().is_none());
        approx_equal!(c.strength(), 0.0, 0.0001);

        // If just t1 exists, it will immediately claim the flag with 0.1
        c.update(&[(t1, 0.1)]);
        assert_eq!(c.owner(), Some(t1));
        approx_equal!(c.strength(), 0.1, 0.0001);

        // If t1 disappears, and t2 enters the fold with a strength exceeding t1's ownership, it
        // should claim immediately, with a strength equal to the leftover after removing the
        // current strength.
        c.update(&[(t2, 0.3)]);
        assert_eq!(c.owner(), Some(t2));
        approx_equal!(c.strength(), 0.2, 0.0001);

        c.set_owner(None);
        c.set_strength(0.2);
        c.update(&[(t1, 0.3), (t2, 0.4)]);
        // 0.4 - 0.3 is 0.1,
        // But t2 contributes (0.4 / (0.3 + 0.4)) of the 0.2
        // Which means it contributed ((0.4 / (0.3 + 0.4))) * 0.2 = 0.1142 to bringing down the
        // flag, so it claims ownership, but without any margin.
        assert_eq!(c.owner(), Some(t2));
        approx_equal!(c.strength(), 0.0, 0.0001);
        c.update(&[(t1, 0.3), (t2, 0.4)]);
        // It keeps ownership, and now climbs at 0.1 per update.
        assert_eq!(c.owner(), Some(t2));
        approx_equal!(c.strength(), 0.1, 0.0001);
    }
}
