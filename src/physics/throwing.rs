#![allow(dead_code)]
use bevy::prelude::*;

/// Parameters for a single throw
/// This structure separates throwing logic from entity spawning
#[derive(Debug, Clone)]
pub struct ThrowParameters {
    /// Vertical throw angle in degrees (pitch)
    pub angle: f32,
    /// Horizontal aim rotation in degrees (-45 to 45)
    pub rotation: f32,
    /// Throw power (0.0 to 1.0, typically 0.5 to 1.0)
    pub power: f32,
    /// Power multiplier (from config)
    pub power_multiplier: f32,
    /// Optional custom spin (if None, will use random spin)
    pub custom_spin: Option<Vec3>,
}

/// Result of throw calculation containing velocity vectors
#[derive(Debug, Clone)]
pub struct ThrowResult {
    /// Linear velocity to apply to the block
    pub linear_velocity: Vec3,
    /// Angular velocity to apply to the block
    pub angular_velocity: Vec3,
}

impl ThrowParameters {
    /// Create throw parameters from current game state
    pub fn new(angle: f32, rotation: f32, power: f32, power_multiplier: f32) -> Self {
        Self {
            angle,
            rotation,
            power,
            power_multiplier,
            custom_spin: None,
        }
    }

    /// Set custom spin for the throw
    pub fn with_spin(mut self, spin: Vec3) -> Self {
        self.custom_spin = Some(spin);
        self
    }

    /// Calculate the throw result (velocities)
    /// This is a pure function that can be tested independently
    pub fn calculate(&self, random_spin_ranges: Option<&Vec3>) -> ThrowResult {
        let linear_velocity =
            calculate_throw_velocity(self.angle, self.rotation, self.power, self.power_multiplier);

        let angular_velocity = if let Some(spin) = self.custom_spin {
            spin
        } else if let Some(ranges) = random_spin_ranges {
            calculate_random_spin(ranges)
        } else {
            Vec3::ZERO
        };

        ThrowResult {
            linear_velocity,
            angular_velocity,
        }
    }
}

/// Calculate linear velocity for a throw
///
/// # Arguments
/// * `angle` - Vertical throw angle in degrees (pitch)
/// * `rotation` - Horizontal aim rotation in degrees
/// * `power` - Throw power (0.0 to 1.0)
/// * `power_multiplier` - Multiplier to convert power to velocity magnitude
///
/// # Returns
/// Linear velocity vector in m/s
pub fn calculate_throw_velocity(
    angle: f32,
    rotation: f32,
    power: f32,
    power_multiplier: f32,
) -> Vec3 {
    let angle_rad = angle.to_radians();
    let rotation_rad = rotation.to_radians();
    let velocity_magnitude = power * power_multiplier;

    // Calculate velocity components based on angle and rotation
    // Angle: vertical (pitch) - how high the block is thrown
    // Rotation: horizontal (yaw) - left/right aim
    Vec3 {
        x: angle_rad.cos() * velocity_magnitude * rotation_rad.sin(),
        y: angle_rad.sin() * velocity_magnitude,
        z: angle_rad.cos() * velocity_magnitude * rotation_rad.cos(),
    }
}

/// Calculate random spin for a thrown block
///
/// # Arguments
/// * `ranges` - Vec3 containing max spin ranges for each axis in radians/sec
///
/// # Returns
/// Angular velocity vector in radians/second
pub fn calculate_random_spin(ranges: &Vec3) -> Vec3 {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    Vec3 {
        x: (rng.gen::<f32>() - 0.5) * ranges.x,
        y: (rng.gen::<f32>() - 0.5) * ranges.y,
        z: (rng.gen::<f32>() - 0.5) * ranges.z,
    }
}

/// Calculate required angle to hit a target at a given distance
/// This is useful for AI opponents to aim at the target
/// Uses physics trajectory equations
///
/// # Arguments
/// * `distance` - Horizontal distance to target (m)
/// * `height_diff` - Vertical height difference (target_y - start_y)
/// * `velocity` - Initial velocity magnitude (m/s)
/// * `gravity` - Gravity magnitude (positive, typically 9.81)
///
/// # Returns
/// Optimal angle in degrees, or None if target is unreachable
pub fn calculate_trajectory_angle(
    distance: f32,
    height_diff: f32,
    velocity: f32,
    gravity: f32,
) -> Option<f32> {
    // Using trajectory equation: tan(θ) = (v² ± sqrt(v⁴ - g(gx² + 2yv²))) / gx
    // We use the lower angle solution (more direct shot)

    let v_squared = velocity * velocity;
    let v_fourth = v_squared * v_squared;
    let g_abs = gravity.abs();

    let discriminant =
        v_fourth - g_abs * (g_abs * distance * distance + 2.0 * height_diff * v_squared);

    if discriminant < 0.0 {
        // Target is unreachable with this velocity
        return None;
    }

    let numerator = v_squared - discriminant.sqrt();
    let denominator = g_abs * distance;

    if denominator == 0.0 {
        return Some(90.0); // Straight up if no horizontal distance
    }

    let angle_rad = (numerator / denominator).atan();
    Some(angle_rad.to_degrees())
}

/// Calculate horizontal rotation needed to aim at a target
///
/// # Arguments
/// * `start_pos` - Starting position of the throw
/// * `target_pos` - Target position to aim at
///
/// # Returns
/// Rotation angle in degrees
pub fn calculate_aim_rotation(start_pos: Vec3, target_pos: Vec3) -> f32 {
    let dx = target_pos.x - start_pos.x;
    let dz = target_pos.z - start_pos.z;

    // Calculate angle in radians, then convert to degrees
    let angle_rad = dx.atan2(dz);
    angle_rad.to_degrees()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_straight_throw() {
        // 45 degree angle, no rotation, half power
        let velocity = calculate_throw_velocity(45.0, 0.0, 0.5, 20.0);

        // At 45 degrees with rotation 0, x should be ~0
        assert!(velocity.x.abs() < 0.01);

        // Y and Z should be equal at 45 degrees
        assert!((velocity.y - velocity.z).abs() < 0.01);

        // Magnitude should be power * multiplier
        let magnitude = (velocity.x.powi(2) + velocity.y.powi(2) + velocity.z.powi(2)).sqrt();
        assert!((magnitude - 10.0).abs() < 0.1); // 0.5 * 20.0 = 10.0
    }

    #[test]
    fn test_angled_throw() {
        // 45 degree angle, 45 degree right rotation
        let velocity = calculate_throw_velocity(45.0, 45.0, 1.0, 20.0);

        // X should be positive (right)
        assert!(velocity.x > 0.0);

        // Z should be positive (forward)
        assert!(velocity.z > 0.0);

        // Y should be positive (up)
        assert!(velocity.y > 0.0);
    }

    #[test]
    fn test_throw_parameters() {
        let params = ThrowParameters::new(45.0, 0.0, 1.0, 20.0);
        let ranges = Vec3::new(5.0, 3.0, 5.0);
        let result = params.calculate(Some(&ranges));

        // Velocity should be non-zero
        assert!(result.linear_velocity.length() > 0.0);

        // Angular velocity should be set (random spin)
        assert!(result.angular_velocity.length() > 0.0);
    }

    #[test]
    fn test_custom_spin() {
        let custom_spin = Vec3::new(1.0, 2.0, 3.0);
        let params = ThrowParameters::new(45.0, 0.0, 1.0, 20.0).with_spin(custom_spin);

        let result = params.calculate(None);

        // Should use custom spin, not random
        assert_eq!(result.angular_velocity, custom_spin);
    }

    #[test]
    fn test_trajectory_angle_calculation() {
        // Test with reasonable game values
        let distance = 23.0; // ~distance from throwing line to target
        let height_diff = 0.0; // Target at same height
        let velocity = 20.0; // Max velocity
        let gravity = 9.81;

        let angle = calculate_trajectory_angle(distance, height_diff, velocity, gravity);
        assert!(angle.is_some());

        let angle = angle.unwrap();
        assert!(angle > 0.0 && angle < 90.0);
    }

    #[test]
    fn test_unreachable_target() {
        // Target too far away
        let angle = calculate_trajectory_angle(100.0, 0.0, 10.0, 9.81);
        assert!(angle.is_none());
    }

    #[test]
    fn test_aim_rotation() {
        // Test aiming straight forward
        let rotation = calculate_aim_rotation(Vec3::new(0.0, 0.0, -15.0), Vec3::new(0.0, 0.0, 8.0));
        assert!((rotation - 0.0).abs() < 0.1);

        // Test aiming to the right
        let rotation =
            calculate_aim_rotation(Vec3::new(0.0, 0.0, -15.0), Vec3::new(10.0, 0.0, 8.0));
        assert!(rotation > 0.0);

        // Test aiming to the left
        let rotation =
            calculate_aim_rotation(Vec3::new(0.0, 0.0, -15.0), Vec3::new(-10.0, 0.0, 8.0));
        assert!(rotation < 0.0);
    }
}
