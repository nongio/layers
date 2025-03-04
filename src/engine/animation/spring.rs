// http://hyperphysics.phy-astr.gsu.edu/hbase/oscda.html
// https://www.ryanjuckett.com/damped-springs/

#[derive(Debug, Clone, Copy)]
pub struct Spring {
    pub mass: f32,
    pub stiffness: f32,
    pub damping: f32,
    pub(crate) initial_velocity: f32,
    pub(crate) initial_position: f32,
    pub(crate) last_update: f32,
}

const TOLERANCE_POSITION: f32 = 0.003;
const TOLERANCE_VELOCITY: f32 = 0.003;

impl Spring {
    pub fn new(mass: f32, stiffness: f32, damping: f32) -> Self {
        Spring {
            mass,
            stiffness,
            damping,
            initial_velocity: 0.0,
            initial_position: 0.0,
            last_update: 0.0,
        }
    }

    pub fn new_with_velocity(
        mass: f32,
        stiffness: f32,
        damping: f32,
        initial_velocity: f32,
    ) -> Self {
        Spring {
            mass,
            stiffness,
            damping,
            initial_velocity,
            initial_position: 0.0,
            last_update: 0.0,
        }
    }

    pub fn with_duration_and_bounce(duration: f32, bounce: f32) -> Self {
        let mass = 1.0;
        let omega = 2.0 * std::f32::consts::PI / duration; // Natural frequency
        let stiffness = mass * omega.powi(2); // Stiffness based on natural frequency

        // Calculate damping based on bounciness
        let damping = if bounce < 0.0 {
            // Overdamped
            2.0 * mass * omega * (1.0 + bounce.abs())
        } else if bounce == 0.0 {
            // Critically damped
            2.0 * mass * omega
        } else {
            // Underdamped
            2.0 * mass * omega * (1.0 - bounce)
        };

        Spring {
            mass,
            stiffness,
            damping,
            initial_velocity: 0.0,
            initial_position: 0.0,
            last_update: 0.0,
        }
    }
    pub fn with_duration_bounce_and_velocity(
        duration: f32,
        bounce: f32,
        initial_velocity: f32,
    ) -> Self {
        let mut spring = Spring::with_duration_and_bounce(duration, bounce);
        spring.initial_velocity = initial_velocity;
        spring
    }
    pub fn update_pos_vel_at(&self, t: f32) -> (f32, f32) {
        let target = 1.0;
        let omega = (self.stiffness / self.mass).sqrt();
        let zeta = self.damping / (2.0 * (self.mass * self.stiffness).sqrt());
        let delta_x = self.initial_position - target;

        if zeta < 1.0 {
            // Underdamped case
            let omega_d = omega * (1.0 - zeta * zeta).sqrt();
            let exp_decay = (-zeta * omega * t).exp();
            let cos_term = (omega_d * t).cos();
            let sin_term = (omega_d * t).sin();
            let new_position = target
                + exp_decay
                    * (delta_x * cos_term
                        + (self.initial_velocity + zeta * omega * delta_x) / omega_d * sin_term);
            let new_velocity = exp_decay
                * (self.initial_velocity * cos_term
                    - (self.initial_velocity + zeta * omega * delta_x) * sin_term / omega_d);
            (new_position, new_velocity)
        } else if zeta == 1.0 {
            // Critically damped case
            let exp_decay = (-omega * t).exp();
            let new_position =
                target + exp_decay * (delta_x + (self.initial_velocity + omega * delta_x) * t);
            let new_velocity =
                exp_decay * (self.initial_velocity * (1.0 - omega * t) + omega * delta_x * t);
            (new_position, new_velocity)
        } else {
            // Overdamped case
            let r1 = -omega * (zeta + (zeta * zeta - 1.0).sqrt());
            let r2 = -omega * (zeta - (zeta * zeta - 1.0).sqrt());
            let exp_r1 = (r1 * t).exp();
            let exp_r2 = (r2 * t).exp();
            let new_position = target + delta_x * (exp_r1 + exp_r2) / 2.0;
            let new_velocity = delta_x * (r1 * exp_r1 + r2 * exp_r2) / 2.0;
            (new_position, new_velocity)
        }
    }

    pub fn update_at(&mut self, elapsed: f32) -> f32 {
        // let dt = elapsed - self.last_update;
        self.update_pos_vel_at(elapsed).0
    }

    pub fn done(&self, elapsed: f32) -> bool {
        let target = 1.0;
        let (position, velocity) = self.update_pos_vel_at(elapsed);

        (position - target).abs() < TOLERANCE_POSITION && velocity.abs() < TOLERANCE_VELOCITY
    }
}
