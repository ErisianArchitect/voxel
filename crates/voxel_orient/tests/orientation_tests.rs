
use std::time::{
    Instant,
    Duration,
};
use voxel_orient::{
    Orientation,
    Direction,
    Rotation,
    Flip,
    Axis,
};


// Exhaustively verify Orientation functions.
#[test]
fn bijection_test() {
    for base in Orientation::iter() {
        for orientation in Orientation::iter() {
            let conjugate = base.conjugate(orientation);
            let local1 = base.reorient_local(orientation);
            let local2 = orientation.reorient(base);
            let local3 = base.reorient(conjugate);
            assert_eq!(local1, local2);
            assert_eq!(local2, local3);
            let l_to_r = base.reorient(orientation);
            let r_to_l = l_to_r.deorient(orientation);
            assert_eq!(base, r_to_l);
            let l_to_r = base.reorient_local(orientation);
            let r_to_l = l_to_r.deorient_local(orientation);
            assert_eq!(base, r_to_l);
            let difference = base.difference(orientation);
            let reoriented = base.reorient(difference);
            let deoriented = orientation.deorient(difference);
            let difference2 = deoriented.difference(reoriented);
            assert_eq!(reoriented, orientation);
            assert_eq!(deoriented, base);
            assert_eq!(difference, difference2);
        }
    }
}

#[test]
fn associativity_test() {
    let start_time = Instant::now();
    for a in Orientation::iter() {
        for b in Orientation::iter() {
            for c in Orientation::iter() {
                assert_eq!(
                    a.reorient(b).reorient(c),
                    a.reorient(b.reorient(c)),
                );
                assert_eq!(
                    a.reorient_local(b).reorient_local(c),
                    a.reorient_local(b.reorient_local(c)),
                );
                // 8 operations
                for axis1 in [Axis::X, Axis::Y, Axis::Z] {
                    assert_eq!(
                        a.reorient_canonical_axis(axis1, b).reorient_canonical_axis(axis1, c),
                        a.reorient_canonical_axis(axis1, b.reorient_canonical_axis(axis1, c)),
                    );
                    // 12 operations
                    for axis2 in [Axis::X, Axis::Y, Axis::Z] {
                        let lhs = a.reorient_canonical_axis(axis1, b).reorient_canonical_axis(axis2, c);
                        let rhs = a.reorient_canonical_axis(axis1, b.reorient_canonical_axis(axis2, c));
                        assert!(lhs.is_equivalent(rhs));
                        // 36 operations
                    }
                }
                // 56 operations
            }
        }
    }
    let elapsed_time = start_time.elapsed();
    println!("396,361,728 operations in {elapsed_time:.3?}");
}

#[test]
fn canonical_isomorphism_test() {
    for base in Orientation::iter() {
        for orientation in Orientation::iter() {
            let l_to_r = base.reorient_canonical_x(orientation);
            let r_to_l = l_to_r.deorient_canonical_x(orientation);
            assert!(base.is_equivalent(r_to_l));
            let l_to_r = base.reorient_canonical_y(orientation);
            let r_to_l = l_to_r.deorient_canonical_y(orientation);
            assert!(base.is_equivalent(r_to_l));
            let l_to_r = base.reorient_canonical_z(orientation);
            let r_to_l = l_to_r.deorient_canonical_z(orientation);
            assert!(base.is_equivalent(r_to_l));
            let l_to_r = base.reorient_canonical_x_local(orientation);
            let r_to_l = l_to_r.deorient_canonical_x_local(orientation);
            assert!(base.is_equivalent(r_to_l));
            let l_to_r = base.reorient_canonical_y_local(orientation);
            let r_to_l = l_to_r.deorient_canonical_y_local(orientation);
            assert!(base.is_equivalent(r_to_l));
            let l_to_r = base.reorient_canonical_z_local(orientation);
            let r_to_l = l_to_r.deorient_canonical_z_local(orientation);
            assert!(base.is_equivalent(r_to_l));
        }
    }
}
