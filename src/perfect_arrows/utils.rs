//! Utilities for the perfect arrows crate.
use std::f32::consts::PI;

#[repr(C)]
#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

#[repr(C)]
#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct Pos2 {
    pub x: f32,
    pub y: f32,
}

///  Modulate a value between two ranges.
/// @param value
/// @param rangeA from [low, high]
/// @param rangeB to [low, high]
/// @param clamp
/// @returns The modulated value.
pub fn modulate(value: f32, range_a: (f32, f32), range_b: (f32, f32), clamp: bool) -> f32 {
    let (from_low, from_high) = range_a;
    let (to_low, to_high) = range_b;
    let result = to_low + ((value - from_low) / (from_high - from_low)) * (to_high - to_low);
    if clamp {
        if to_low < to_high {
            if result < to_low {
                return to_low;
            }
            if result > to_high {
                return to_high;
            }
        } else {
            if result > to_low {
                return to_low;
            }
            if result < to_high {
                return to_high;
            }
        }
    }
    result
}

/// Rotate a point around a center.
/// * @param point The x, y coordinate of the point.
/// * @param center The x, y coordinate of the point to rotate round.
/// * @param angle The distance (in radians) to rotate.
///
pub fn rotate_point(point: &Pos2, center: &Pos2, angle: f32) -> Pos2 {
    let s = angle.sin();
    let c = angle.cos();

    let px = point.x - center.x;
    let py = point.y - center.y;

    let nx = px * c - py * s;
    let ny = px * s + py * c;

    Pos2 {
        x: nx + center.x,
        y: ny + center.y,
    }
}

/// Get the distance between two points.
/// * @param pos0 The x, y coordinate of the first point.
/// * @param pos1 The x, y coordinate of the second point.
/// @returns The distance between the two points.
pub fn get_distance(pos0: &Pos2, pos1: &Pos2) -> f32 {
    ((pos1.y - pos0.y).powi(2) + (pos1.x - pos0.x).powi(2)).sqrt()
}

/// Get an angle (radians) between two points.
/// * @param pos0 The x, y coordinate of the first point.
/// * @param pos1 The x, y coordinate of the second point.
/// @returns The angle between the two points.
pub fn get_angle(pos0: &Pos2, pos1: &Pos2) -> f32 {
    (pos1.y - pos0.y).atan2(pos1.x - pos0.x)
}

/// Move a point in an angle by a distance.
/// * @param pos0 The x, y coordinate of the point.
/// * @param a The angle in radians.
/// * @param d The distance to move.
/// @returns The new point.
pub fn project_point(pos0: Pos2, a: f32, d: f32) -> Pos2 {
    Pos2 {
        x: pos0.x + a.cos() * d,
        y: pos0.y + a.sin() * d,
    }
}

/// Get a point between two points.
/// * @param pos0 The x, y coordinate of the first point.
/// * @param pos1 The x, y coordinate of the second point.
/// * @param d The normalized distance between the two points.
/// @returns The point between the two points.
pub fn get_point_between(pos0: &Pos2, pos1: &Pos2, d: f32) -> Pos2 {
    Pos2 {
        x: pos0.x + (pos1.x - pos0.x) * d,
        y: pos0.y + (pos1.y - pos0.y) * d,
    }
}

/// Get the sector of an angle (e.g. quadrant, octant)
/// * @param a The angle to check.
/// * @param s The number of sectors to check.
/// @returns The sector of the angle.
pub fn get_sector(a: f32, s: i32) -> i32 {
    (s as f32 * (0.5 + ((a / (PI * 2.0)) % s as f32))).floor() as i32
}

/// Get a normal value representing how close two points are from being at a 45 degree angle.
/// * @param pos0 The x, y coordinate of the first point.
/// * @param pos1 The x, y coordinate of the second point.
/// @returns The angliness value.
pub fn get_angliness(pos0: Pos2, pos1: Pos2) -> f32 {
    ((pos1.x - pos0.x) / (pos1.y - pos0.y)).abs()
}

/// Check whether two rectangles will collide (overlap).
/// * @param x0 The x-axis coordinate of the first rectangle.
/// * @param y0 The y-axis coordinate of the first rectangle.
/// * @param w0 The width of the first rectangle.
/// * @param h0 The height of the first rectangle.
/// * @param x1 The x-axis coordinate of the second rectangle.
/// * @param y1 The y-axis coordinate of the second rectangle.
/// * @param w1 The width of the second rectangle.
/// * @param h1 The height of the second rectangle.
/// @returns Whether the rectangles collide.
pub(crate) fn do_rectangles_collide(
    x0: f32,
    y0: f32,
    w0: f32,
    h0: f32,
    x1: f32,
    y1: f32,
    w1: f32,
    h1: f32,
) -> bool {
    !(x0 >= x1 + w1 || x1 >= x0 + w0 || y0 >= y1 + h1 || y1 >= y0 + h0)
}

/// Find the point(s) where a segment intersects a rectangle.
/// * @param x0 The x-axis coordinate of the segment's starting point.
/// * @param y0 The y-axis coordinate of the segment's starting point.
/// * @param x1 The x-axis coordinate of the segment's ending point.
/// * @param y1 The y-axis coordinate of the segment's ending point.
/// * @param x The x-axis coordinate of the rectangle.
/// * @param y The y-axis coordinate of the rectangle.
/// * @param w The width of the rectangle.
/// * @param h The height of the rectangle.
/// @returns The intersection points.
pub fn get_segment_rectangle_intersection_points(
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
) -> Vec<[f32; 2]> {
    let mut points: Vec<[f32; 2]> = vec![];

    for [px0, py0, px1, py1] in [
        [x, y, x + w, y],
        [x + w, y, x + w, y + h],
        [x + w, y + h, x, y + h],
        [x, y + h, x, y],
    ] {
        let ints = get_segment_segment_intersection(px0, py0, px1, py1, x0, y0, x1, y1);

        if let Some(ints) = ints {
            points.push(ints);
        }
    }

    points
}

/// Find the point, if any, where two segments intersect.
/// * @param x0 The x-axis coordinate of the first segment's starting point.
/// * @param y0 The y-axis coordinate of the first segment's starting point.
/// * @param x1 The x-axis coordinate of the first segment's ending point.
/// * @param y1 The y-axis coordinate of the first segment's ending point.
/// * @param x2 The x-axis coordinate of the second segment's starting point.
/// * @param y2 The y-axis coordinate of the second segment's starting point.
/// * @param x3 The x-axis coordinate of the second segment's ending point.
/// * @param y3 The y-axis coordinate of the second segment's ending point.
/// @returns The intersection point.
pub fn get_segment_segment_intersection(
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    x3: f32,
    y3: f32,
) -> Option<[f32; 2]> {
    let denom = (y3 - y2) * (x1 - x0) - (x3 - x2) * (y1 - y0);
    let nume_a = (x3 - x2) * (y0 - y2) - (y3 - y2) * (x0 - x2);
    let nume_b = (x1 - x0) * (y0 - y2) - (y1 - y0) * (x0 - x2);

    if denom == 0.0 {
        if nume_a == 0.0 && nume_b == 0.0 {
            return None; // Colinear
        }
        return None; // Parallel
    }

    let u_a = nume_a / denom;
    let u_b = nume_b / denom;

    if (0.0..=1.0).contains(&u_a) && (0.0..=1.0).contains(&u_b) {
        return Some([x0 + u_a * (x1 - x0), y0 + u_a * (y1 - y0)]);
    }

    None // No intersection
}

/// Get the point(s) where a line segment intersects a circle.
/// * @param cx The x-axis coordinate of the circle's center.
/// * @param cy The y-axis coordinate of the circle's center.
/// * @param r The circle's radius.
/// * @param x0 The x-axis coordinate of the segment's starting point.
/// * @param y0 The y-axis coordinate of ththe segment's ending point.
/// * @param x1 The delta-x of the ray.
/// * @param y1 The delta-y of the ray.
/// @returns The intersection points.
pub fn get_segment_circle_intersections(
    cx: f32,
    cy: f32,
    r: f32,
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
) -> Option<Vec<[f32; 2]>> {
    let v1 = [x1 - x0, y1 - y0];
    let v2 = [x0 - cx, y0 - cy];

    let b = v1[0] * v2[0] + v1[1] * v2[1];
    let c = 2.0 * (v1[0] * v1[0] + v1[1] * v1[1]);
    let b = -2.0 * b;
    let d = (b * b - 2.0 * c * (v2[0] * v2[0] + v2[1] * v2[1] - r * r)).sqrt();

    if d.is_nan() {
        return None;
    }

    let u1 = (b - d) / c;
    let u2 = (b + d) / c;
    let mut ret_p1 = [0.0, 0.0];
    let mut ret_p2 = [0.0, 0.0];
    let mut ret = vec![];

    if (0.0..=1.0).contains(&u1) {
        ret_p1[0] = x0 + v1[0] * u1;
        ret_p1[1] = y0 + v1[1] * u1;
        ret.push(ret_p1);
    }

    if (0.0..=1.0).contains(&u2) {
        ret_p2[0] = x0 + v1[0] * u2;
        ret_p2[1] = y0 + v1[1] * u2;
        ret.push(ret_p2);
    }

    Some(ret)
}

/// Normalize an angle (in radians)
/// * @param radians The radians quantity to normalize.
/// @returns The normalized angle.
pub fn normalize_angle(radians: f32) -> f32 {
    radians - PI * 2.0 * (radians / (PI * 2.0)).floor()
}

/// Get the point at which a ray intersects a segment.
/// * @param x The x-axis coordinate of the ray's origin.
/// * @param y The y-axis coordinate of the ray's origin.
/// * @param dx The x-axis delta of the angle.
/// * @param dy The y-axis delta of the angle.
/// * @param x0 The x-axis coordinate of the segment's start point.
/// * @param y0 The y-axis coordinate of the segment's start point.
/// * @param x1 The x-axis coordinate of the segment's end point.
/// * @param y1 The y-axis coordinate of the segment's end point.
/// @returns The intersection point.
pub fn get_ray_segment_intersection(
    x: f32,
    y: f32,
    dx: f32,
    dy: f32,
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
) -> Option<Pos2> {
    let d = dx * (y1 - y0) - dy * (x1 - x0);

    if dy * (x1 - x0) != dx * (y1 - y0) && d != 0.0 {
        let r = ((y - y0) * (x1 - x0) - (x - x0) * (y1 - y0)) / d;
        let s = ((y - y0) * dx - (x - x0) * dy) / d;

        if r >= 0.0 && (0.0..=1.0).contains(&s) {
            return Some(Pos2 {
                x: x + r * dx,
                y: y + r * dy,
            });
        }
    }

    None
}

/// Get the normalized delta (x and y) for an angle.
/// * @param angle The angle in radians
/// @returns The delta.
pub fn get_delta(angle: f32) -> (f32, f32) {
    (angle.cos(), angle.sin())
}

/// Get a normal value representing how close two points are from being at a 45 degree angle.
/// * @param angle The angle in radians
/// @returns The intermediate value.
pub fn get_intermediate(angle: f32) -> f32 {
    let pi_over_4 = PI / 4.0;
    let inner = (angle % (PI / 2.0)).abs() - PI / 4.0;
    inner.abs() / pi_over_4
}

/// Get a line between two rounded rectangles.
/// * @param x0
/// * @param y0
/// * @param w0
/// * @param h0
/// * @param r0
/// * @param x1
/// * @param y1
/// * @param w1
/// * @param h1
/// * @param r1
/// @returns The line.
pub fn get_line_between_rounded_rectangles(
    x0: f32,
    y0: f32,
    w0: f32,
    h0: f32,
    r0: f32,
    x1: f32,
    y1: f32,
    w1: f32,
    h1: f32,
    r1: f32,
) -> (Pos2, Pos2) {
    let cx0 = x0 + w0 / 2.0;
    let cy0 = y0 + h0 / 2.0;
    let cx1 = x1 + w1 / 2.0;
    let cy1 = y1 + h1 / 2.0;
    let di0 =
        get_ray_rounded_rectangle_intersection(cx0, cy0, cx1 - cx0, cy1 - cy0, x0, y0, w0, h0, r0)
            [0]
        .clone();
    let di1 =
        get_ray_rounded_rectangle_intersection(cx1, cy1, cx0 - cx1, cy0 - cy1, x1, y1, w1, h1, r1)
            [0]
        .clone();

    (di0, di1)
}

/// Get the intersection points between a ray and a rectangle with rounded corners.
/// * @param ox The x-axis coordinate of the ray's origin.
/// * @param oy The y-axis coordinate of the ray's origin.
/// * @param dx The delta-x of the ray.
/// * @param dy The delta-y of the ray.
/// * @param x The x-axis coordinate of the rectangle.
/// * @param y The y-axis coordinate of the rectangle.
/// * @param w The width of the rectangle.
/// * @param h The height of the rectangle.
/// * @param r The corner radius of the rectangle.
/// @returns The intersection points.
pub fn get_ray_rounded_rectangle_intersection(
    ox: f32,
    oy: f32,
    dx: f32,
    dy: f32,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    r: f32,
) -> Vec<Pos2> {
    let mx = x + w;
    let my = y + h;
    let rx = x + r;
    let ry = y + r;
    let mrx = x + w - r;
    let mry = y + h - r;

    let segments = [
        [x, mry, x, ry],
        [rx, y, mrx, y],
        [mx, ry, mx, mry],
        [mrx, my, rx, my],
    ];

    let corners = [
        [rx, ry, PI, PI * 1.5],
        [mrx, ry, PI * 1.5, PI * 2.0],
        [mrx, mry, 0.0, PI * 0.5],
        [rx, mry, PI * 0.5, PI],
    ];

    let mut points: Vec<Pos2> = vec![];

    for (i, segment) in segments.iter().enumerate() {
        let [px0, py0, px1, py1] = segment;
        let [cx, cy, as_, ae] = corners[i];

        let intersections = get_ray_circle_intersections(cx, cy, r, ox, oy, dx, dy);

        if let Some(intersections) = intersections {
            for pt in intersections {
                let point_angle = normalize_angle(get_angle(
                    &Pos2 { x: cx, y: cy },
                    &Pos2 { x: pt.x, y: pt.y },
                ));
                if point_angle > as_ && point_angle < ae {
                    points.push(pt);
                }
            }
        }

        let segment_int = get_ray_segment_intersection(ox, oy, dx, dy, *px0, *py0, *px1, *py1);

        if let Some(segment_int) = segment_int {
            points.push(segment_int);
        }
    }
    points
}

/// Get Rectangle Segment Intersected By Ray
/// * @param x
/// * @param y
/// * @param w
/// * @param h
/// * @param ox
/// * @param oy
/// * @param dx
/// * @param dy
/// @returns The intersection points.
pub fn get_rectangle_segment_intersected_by_ray(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    ox: f32,
    oy: f32,
    dx: f32,
    dy: f32,
) -> Vec<[f32; 4]> {
    get_rectangle_segments(x, y, w, h)
        .into_iter()
        .filter(|[sx0, sy0, sx1, sy1]| {
            get_ray_segment_intersection(ox, oy, dx, dy, *sx0, *sy0, *sx1, *sy1).is_some()
        })
        .collect()
}

/// Get Rectangle Segments.
/// * @param x
/// * @param y
/// * @param w
/// * @param h
/// @returns The segments.
pub fn get_rectangle_segments(x: f32, y: f32, w: f32, h: f32) -> Vec<[f32; 4]> {
    vec![
        [x, y, x + w, y],
        [x + w, y, x + w, y + h],
        [x + w, y + h, x, y + h],
        [x, y + h, x, y],
    ]
}

/// Get the intersection points between a ray and a circle.
/// * @param cx The x-axis coordinate of the circle's center.
/// * @param cy The y-axis coordinate of the circle's center.
/// * @param r The circle's radius.
/// * @param ox The x-axis coordinate of the ray's origin.
/// * @param oy The y-axis coordinate of the ray's origin.
/// * @param dx The delta-x of the angle.
/// * @param dy The delta-y of the angle.
/// @returns The intersection points.
pub fn get_ray_circle_intersections(
    cx: f32,
    cy: f32,
    r: f32,
    ox: f32,
    oy: f32,
    dx: f32,
    dy: f32,
) -> Option<Vec<Pos2>> {
    let a = dx * dx + dy * dy;
    let b = 2.0 * dx * (ox - cx) + 2.0 * dy * (oy - cy);
    let c = (ox - cx) * (ox - cx) + (oy - cy) * (oy - cy) - r * r;
    let d = b * b - 4.0 * a * c;

    if d < 0.0 {
        return None;
    }

    let mut t_values: Vec<f32> = vec![];

    if d == 0.0 {
        t_values.push(-b / 2.0 / a);
    } else {
        t_values.push((-b + d.sqrt()) / 2.0 / a);
        t_values.push((-b - d.sqrt()) / 2.0 / a);
    }

    let mut ret: Vec<Pos2> = vec![];

    for t in t_values {
        let x = ox + dx * t;
        let y = oy + dy * t;
        ret.push(Pos2 { x, y });
    }

    Some(ret)
}
