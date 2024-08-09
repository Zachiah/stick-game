use bevy::math::Vec2;

// TODO: Learn what this code does
pub fn convert_to_endpoints(center: Vec2, length: f32, rotation: f32) -> (Vec2, Vec2) {
    let half_length = length / 2.0;
    let dx = half_length * rotation.cos();
    let dy = half_length * rotation.sin();
    let start = Vec2::new(center.x + dx, center.y + dy);
    let end = Vec2::new(center.x - dx, center.y - dy);
    (start, end)
}

pub fn lines_intersect(p1: Vec2, p2: Vec2, p3: Vec2, p4: Vec2) -> Option<Vec2> {
    let den = (p1.x - p2.x) * (p3.y - p4.y) - (p1.y - p2.y) * (p3.x - p4.x);
    if den == 0.0 {
        return None; // Lines are parallel or coincident
    }

    let num1 = p1.x * p2.y - p1.y * p2.x;
    let num2 = p3.x * p4.y - p3.y * p4.x;

    let x = (num1 * (p3.x - p4.x) - (p1.x - p2.x) * num2) / den;
    let y = (num1 * (p3.y - p4.y) - (p1.y - p2.y) * num2) / den;

    let intersection = Vec2::new(x, y);

    if (x < p1.x.min(p2.x) || x > p1.x.max(p2.x) || y < p1.y.min(p2.y) || y > p1.y.max(p2.y))
        || (x < p3.x.min(p4.x) || x > p3.x.max(p4.x) || y < p3.y.min(p4.y) || y > p3.y.max(p4.y))
    {
        return None; // Intersection point is not within the line segments
    }

    Some(intersection)
}
