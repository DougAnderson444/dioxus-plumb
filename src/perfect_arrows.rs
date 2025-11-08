mod utils;
use utils::*;
pub use utils::{Pos2, Vec2};


use std::f64::consts::PI;

const PI2: f64 = PI * 2.0;
const MIN_ANGLE: f64 = PI / 24.0;

/// Arrow Options
#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct ArrowOptions {
    pub bow: f64,
    pub stretch: f64,
    pub stretch_min: f64,
    pub stretch_max: f64,
    pub pad_start: f64,
    pub pad_end: f64,
    pub flip: bool,
    pub straights: bool,
}

impl Default for ArrowOptions {
    /// Default with the following vals:
    /// bow = 0,
    /// stretch = 0.5,
    /// stretchMin = 0,
    /// stretchMax = 420,
    /// padStart = 0,
    /// padEnd = 0,
    /// flip = false,
    /// straights = true,
    fn default() -> Self {
        ArrowOptions {
            bow: 0.0,
            stretch: 0.25,
            stretch_min: 50.0,
            stretch_max: 800.0,
            pad_start: 0.0,
            pad_end: 0.0,
            flip: false,
            straights: false,
        }
    }
}

impl ArrowOptions {
    /// Default with flip as the given value
    pub fn with_flip(flip: bool) -> Self {
        ArrowOptions {
            flip,
            ..Default::default()
        }
    }
}

pub fn get_box_to_box_arrow(
    start: Pos2,
    start_size: Vec2,
    end: Pos2,
    end_size: Vec2,
    options: ArrowOptions,
) -> (Pos2, Pos2, Pos2, f64, f64, f64) {
    let ArrowOptions {
        bow,
        stretch,
        stretch_min,
        stretch_max,
        pad_start,
        pad_end,
        flip,
        straights,
    } = options;

    let px0 = start.x - pad_start;
    let py0 = start.y - pad_start;
    let pw0 = start_size.x + pad_start * 2.0;
    let ph0 = start_size.y + pad_start * 2.0;
    let px1 = end.x - pad_end;
    let py1 = end.y - pad_end;
    let pw1 = end_size.x + pad_end * 2.0;
    let ph1 = end_size.y + pad_end * 2.0;
    let cx0 = start.x + start_size.x / 2.0;
    let cy0 = start.y + start_size.y / 2.0;
    let cx1 = end.x + end_size.x / 2.0;
    let cy1 = end.y + end_size.y / 2.0;

    let angle_center = normalize_angle(get_angle(
        &Pos2 { x: cx0, y: cy0 },
        &Pos2 { x: cx1, y: cy1 },
    ));
    let distance = get_distance(&Pos2 { x: cx0, y: cy0 }, &Pos2 { x: cx1, y: cy1 });

    if distance == 0.0 {
        let s = Pos2 { x: cx0, y: py0 };
        let e = Pos2 { x: cx1, y: py1 };
        let c = get_point_between(&s, &e, 0.5);
        let ca = get_angle(&s, &e);
        return (s, c, e, ca, ca, ca);
    }

    let rot = if get_sector(angle_center, 8) % 2 == 0 {
        -1
    } else {
        1
    } * if flip { -1 } else { 1 };
    let mut card = get_intermediate(angle_center);

    if card < 1.0 && card > 0.85 {
        card = 0.99;
    }

    let is_colliding = do_rectangles_collide(px0, py0, pw0, ph0, px1, py1, pw1, ph1);
    let (di0, di1) = get_line_between_rounded_rectangles(
        px0, py0, pw0, ph0, pad_start, px1, py1, pw1, ph1, pad_end,
    );
    let distance_between = get_distance(&di0, &di1);

    if !is_colliding && straights && card % 0.5 == 0.0 {
        let mpd = get_point_between(&di0, &di1, 0.5);
        return (di0, mpd, di1, angle_center, angle_center - PI, angle_center);
    }

    let overlap_effect = if is_colliding {
        modulate(distance_between, (0.0, distance), (0.0, 1.0), true)
    } else {
        0.0
    };

    let dist_effect = 1.0 - distance_between / distance;
    let stretch_effect = modulate(
        distance_between,
        (stretch_min, stretch_max),
        (1.0, 0.0),
        true,
    );
    let mut arc = bow + stretch_effect * stretch;
    let angle_offset = modulate(card * card, (0.0, 1.0), (PI * 0.125, 0.0), true);
    let dist_offset = if is_colliding {
        PI * 0.5 * card
    } else {
        modulate(dist_effect, (0.75, 1.0), (0.0, PI * 0.5), true) * card
    };

    let combined_offset = dist_offset
        + angle_offset
            * if is_colliding {
                1.0 - overlap_effect
            } else {
                1.0
            };

    let final_angle0 = if overlap_effect >= 0.5 {
        angle_center + PI * rot as f64
    } else {
        angle_center + f64::max(MIN_ANGLE, combined_offset) * rot as f64
    };

    let (dx0, dy0) = get_delta(final_angle0.rem_euclid(PI2));

    let ts =
        get_ray_rounded_rectangle_intersection(cx0, cy0, dx0, dy0, px0, py0, pw0, ph0, pad_start);
    let start_seg =
        get_rectangle_segment_intersected_by_ray(px0, py0, pw0, ph0, cx0, cy0, dx0, dy0);
    let [ssx0, ssy0, ssx1, ssy1] = start_seg[0][..] else {
        todo!()
    };
    let smp = get_point_between(&Pos2 { x: ssx0, y: ssy0 }, &Pos2 { x: ssx1, y: ssy1 }, 0.5);
    let start = get_point_between(
        &ts[0],
        &smp,
        if is_colliding {
            f64::max(overlap_effect, 0.15)
        } else {
            0.15
        },
    );

    arc *= 1.0 + (dist_effect.clamp(-2.0, 2.0) * card - overlap_effect) / 2.0;

    if is_colliding {
        arc = if arc < 0.0 {
            f64::min(arc, -0.5)
        } else {
            f64::max(arc, 0.5)
        };
    }

    let end = if overlap_effect >= 0.5 {
        let ray_angle = get_angle(&Pos2 { x: cx0, y: cy0 }, &smp);
        let (dx1, dy1) = get_delta(ray_angle);
        let e_temp =
            get_ray_rounded_rectangle_intersection(cx1, cy1, dx1, dy1, px1, py1, pw1, ph1, pad_end);
        e_temp[0].clone()
    } else {
        let dist_offset1 = modulate(dist_effect, (0.75, 1.0), (0.0, 1.0), true);
        let overlap_effect1 = if is_colliding {
            modulate(overlap_effect, (0.0, 1.0), (0.0, PI / 8.0), true)
        } else {
            0.0
        };

        let card_effect1 = modulate(card * dist_offset1, (0.0, 1.0), (0.0, PI / 16.0), true);
        let combined_offset = dist_effect * (PI / 12.0)
            + (card_effect1 + overlap_effect1)
            + (dist_offset + angle_offset) / 2.0;
        let final_angle1 = if overlap_effect >= 0.5 {
            angle_center + PI * rot as f64
        } else {
            angle_center + PI - f64::max(combined_offset, MIN_ANGLE) * rot as f64
        };

        let (dx1, dy1) = get_delta(final_angle1.rem_euclid(PI2));

        let te = &get_ray_rounded_rectangle_intersection(
            cx1, cy1, dx1, dy1, px1, py1, pw1, ph1, pad_end,
        )[0];
        let end_seg =
            get_rectangle_segment_intersected_by_ray(px1, py1, pw1, ph1, cx1, cy1, dx1, dy1)[0];
        let [sex0, sey0, sex1, sey1] = end_seg[..] else {
            todo!()
        };
        let se0 = Pos2 { x: sex0, y: sey0 };
        let se1 = Pos2 { x: sex1, y: sey1 };
        let emp = get_point_between(&se0, &se1, 0.5);

        get_point_between(te, &emp, 0.25 + overlap_effect * 0.25)
    };

    let m1 = get_point_between(&start, &end, 0.5);
    let ti = get_point_between(&start, &end, (0.5 + arc).clamp(-1.0, 1.0));
    let ci_a = rotate_point(&ti, &m1, (PI / 2.0) * rot as f64);
    let ci_b = rotate_point(&ti, &m1, (PI / 2.0) * -rot as f64);

    let control = if is_colliding
        && get_distance(&ci_a, &Pos2 { x: cx1, y: cy1 })
            < get_distance(&ci_b, &Pos2 { x: cx1, y: cy1 })
    {
        ci_b
    } else {
        ci_a
    };

    let angle_start = get_angle(&control, &start);
    let angle_end = get_angle(&control, &end);

    (start, control, end, angle_end, angle_start, angle_center)
}
