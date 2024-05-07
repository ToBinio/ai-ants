use glam::Vec2;

/// math based on https://www.youtube.com/watch?v=23kTf-36Fcw
pub fn ray_inserect_circle(
    circle_center: Vec2,
    circle_radius: f32,
    ray_center: Vec2,
    ray_direction: Vec2,
) -> Option<f32> {
    let e_vec = circle_center - ray_center;

    let a = e_vec.dot(ray_direction);

    let e_length = e_vec.length_squared();

    let b_sq = e_length - a * a;

    if b_sq > (circle_radius * circle_radius) {
        return None;
    }

    let f_sq = circle_radius * circle_radius - b_sq;

    if f_sq < 0. {
        return None;
    }

    let f = (f_sq).sqrt();
    let result = a - f;

    if result < 0. {
        return None;
    }

    Some(result)
}

#[cfg(test)]
mod tests {
    use glam::vec2;

    use crate::math::ray_inserect_circle;

    #[test]
    fn ray_misses() {
        let center = vec2(5., 5.);
        let radius = 2.;

        let ray_center = vec2(9., 2.);
        let ray_direction = (vec2(6., 8.) - ray_center).normalize();

        let result = ray_inserect_circle(center, radius, ray_center, ray_direction);
        assert_eq!(result, None);

        let ray_center = vec2(6., 8.);
        let ray_direction = (vec2(2., 6.) - ray_center).normalize();

        let result = ray_inserect_circle(center, radius, ray_center, ray_direction);
        assert_eq!(result, None);

        let ray_center = vec2(4., 2.);
        let ray_direction = (vec2(2., 4.) - ray_center).normalize();

        let result = ray_inserect_circle(center, radius, ray_center, ray_direction);
        assert_eq!(result, None);

        let ray_center = vec2(3., 6.);
        let ray_direction = (vec2(2., 7.) - ray_center).normalize();

        let result = ray_inserect_circle(center, radius, ray_center, ray_direction);
        assert_eq!(result, None);
    }

    #[test]
    fn ray_hits() {
        let center = vec2(5., 5.);
        let radius = 2.;

        let ray_center = vec2(9., 2.);
        let ray_direction = (vec2(3., 4.) - ray_center).normalize();

        let result = ray_inserect_circle(center, radius, ray_center, ray_direction);
        assert!(result.is_some());

        let ray_center = vec2(6., 8.);
        let ray_direction = (vec2(5., 4.) - ray_center).normalize();

        let result = ray_inserect_circle(center, radius, ray_center, ray_direction);
        assert!(result.is_some());

        let ray_center = vec2(4., 2.);
        let ray_direction = (vec2(8., 6.) - ray_center).normalize();

        let result = ray_inserect_circle(center, radius, ray_center, ray_direction);
        assert!(result.is_some());

        let ray_center = vec2(3., 6.);
        let ray_direction = (vec2(5., 7.) - ray_center).normalize();

        let result = ray_inserect_circle(center, radius, ray_center, ray_direction);
        assert!(result.is_some());
    }

    #[test]
    fn does_not_intersect_from_inside() {
        let center = vec2(5., 5.);
        let radius = 2.;

        let ray_center = vec2(5., 6.);
        let ray_direction = (vec2(3., 4.) - ray_center).normalize();

        let result = ray_inserect_circle(center, radius, ray_center, ray_direction);
        assert_eq!(result, None);
    }
}
