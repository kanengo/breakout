use bevy::{math::Vec2, utils::petgraph::matrix_graph::Zero, sprite::collide_aabb::Collision};

fn cross2d(v1:Vec2, v2: Vec2) -> f32 {
   v1.x*v2.y - v1.y*v2.x
}


fn time_of_vertex_edge_parallel(x0: Vec2, v0: Vec2, x1: Vec2, x2: Vec2) -> Vec<Option<f32>> {
    let a = cross2d(v0, Vec2::ZERO);
    let b = cross2d(x0 - x1, Vec2::ZERO) + cross2d(v0, x2-x1);
    let c = cross2d(x0 - x1, x2 - x1);

    let d = b*b - 4.0*a*c;

    let mut result = vec![None; 2];
    let mut result_num = 0;

    if a.is_zero() {
        result[1] = Some(-c / b);
        return result;
    }

    if d < 0.0 {
        result[0] = Some(-b / (2.0 * a));
    } else{
        let q = -(b + b.sin() * d.sqrt()) / 2.0;
        if a.abs() >  1e-12 * q.abs() {
            result[result_num] = Some(q / a);
            result_num += 1;
        }

        if q.abs() > 1e-12*c.abs() {
            result[result_num] = Some(c / q);
            result_num += 1;
        }

        if result_num == 2 && result[0].unwrap() > result[1].unwrap() {
            let temp = result[0];
            result[0] = result[1];
            result[1] = temp;
        }
    }

    result
}

pub fn check_collide_point_nearest_edge(point:Vec2, v: Vec2, edges: &Vec<(Vec2,Vec2, Collision)> ) -> Option<(f32, Collision)> {
    let mut nearest = None;

    for &(x1, x2, collision) in edges.iter() {
        let toi_result = time_of_vertex_edge_parallel(point, v, x1, x2);
        for toi in toi_result {
            if let Some(toi) = toi {
                if toi > 0.0 &&  !toi.is_infinite() {
                    let parallel_point = Vec2::new(point.x + v.x * toi, point.y + v.y *toi);
                    // println!("point: {} parallel_point:{} {} {:?} {}", point, parallel_point, toi, collision, 1e-6);
                    //     cross2d(parallel_point - x1, x2 - x1).abs());
                    if parallel_point.x >= x1.x.min(x2.x) - 1e-6 && parallel_point.x <= x1.x.max(x2.x) + 1e-6
                        && parallel_point.y >= x1.y.min(x2.y) - 1e-6 && parallel_point.y <= x1.y.max(x2.y) + 1e-6 {
                    // if cross2d(parallel_point - x1, x2-x1).abs() > 1e-10{
                        match nearest {
                            Some((t, _)) => {
                                if toi < t {
                                    nearest = Some((toi, collision));
                                }
                            }
                            None => {
                                nearest = Some((toi, collision));
                            }
                        }
                    }
                }
            }
        }
    }

    nearest
}

pub(crate) fn time_of_collide_circle_rect(circle: Vec2, radius: f32, v: Vec2, rect: Vec2, rect_size: Vec2) -> Option<(f32, Collision)> {
    if v == Vec2::ZERO {
        return None
    }
    let mut nearest = None;

    let x1 = Vec2::new(rect.x - rect_size.x / 2.0, rect.y - rect_size.y/2.0);
    let x2 = Vec2::new(rect.x - rect_size.x / 2.0,rect.y + rect_size.y/2.0);
    let x3 = Vec2::new(rect.x + rect_size.x / 2.0,rect.y + rect_size.y/2.0);
    let x4 = Vec2::new(rect.x + rect_size.x / 2.0,rect.y - rect_size.y/2.0);

    let edges = vec![
        (x1, x2,Collision::Left), 
        (x2, x3, Collision::Top),
        (x3, x4, Collision::Right),
        (x4, x1, Collision::Bottom),
    ];

    // println!("edges: {:?}", edges);

    // let check_points: (Option<Vec2>, Option<Vec2>);
    let point1 ;
    if v.x > 0.0 {
        point1 = Some(Vec2::new(circle.x + radius, circle.y));
    } else if v.x < 0.0 {
        point1 = Some(Vec2::new(circle.x - radius, circle.y));
    } else {
        point1 = None;
    }

    let point2 ;
    if v.y > 0.0 {
        point2 = Some(Vec2::new(circle.x, circle.y + radius));
    } else if v.y < 0.0 {
        point2 = Some(Vec2::new(circle.x, circle.y - radius));
    } else {
        point2 = None;
    }
    //
    let point3;
    if !v.x.is_zero() && !v.y.is_zero() {
        let angle = (v.y/v.x).abs().atan();
        let x;
        let y;
        if v.x > 0.0 {
            x = radius * angle.cos();
        } else {
            x = -radius * angle.cos();
        }

        if v.y > 0.0 {
            y = radius * angle.sin();
        } else {
            y = -radius * angle.sin();
        }

        point3 = Some(Vec2::new(x, y));
    } else {
        point3 = None;
    }


    let check_points = vec![point1, point2, point3];
    let mut nearest_point = None;

    for point in check_points {
        // println!("point:{:?}", point);
        if let Some(point) = point {
            let checked_nearest = check_collide_point_nearest_edge(point, v, &edges);
            // println!("checked_nearest:{:?} point:{}", checked_nearest, point);
            if let Some(checked_nearest) = checked_nearest {
                match nearest {
                    Some((t, _)) => {
                        if checked_nearest.0 < t {
                            nearest = Some(checked_nearest);
                            nearest_point = Some(point);
                        }
                    }
                    None => {
                        nearest = Some(checked_nearest);
                        nearest_point = Some(point);
                    }
                }
            }
        }
    }
    // println!("===nearest point:{:?}===", nearest_point);
    nearest
}

#[cfg(test)]
mod tests {
    use std::f32::consts::PI;
    use bevy::math::Vec2;

    use super::{time_of_vertex_edge_parallel, time_of_collide_circle_rect};

    #[test]
    fn test_time_of_vetex_edge_parallel() {
        let result = time_of_vertex_edge_parallel(
           Vec2::new(0.0, -5.0),
           Vec2::new(0.0,1.0),
           Vec2::new(-1.0, 0.0),
           Vec2::new(1.0,0.0),
        );
        
        println!("result: {:?}", result);

        let mut list = vec![Vec2::ZERO,Vec2::ZERO,Vec2::ZERO];


        let mut x = 1;

        println!("x: {}", x);

        // x = 2;

        let y = x;

        println!("x: {}", x);
    }

    #[test]
    fn test_time_of_collide_circle_rect() {
        let v = Vec2::new(-9.495282,282.6833);
        let circle = Vec2::new(27.58929-v.x*0.015625 * 4.0, 4.3467054-v.y*0.015625 * 4.0);
        let radius = 4.0;

        let rect = Vec2::new(30.0,6.0);
        let rect_size = Vec2::new(10.0,10.0);


        let collision = time_of_collide_circle_rect(circle, radius, v, rect, rect_size);

        println!("result:{:?}", collision);

        let tan :f32 =-1.0;
        println!("tan:{} {}", tan.atan(), PI/4.0);
    }
}