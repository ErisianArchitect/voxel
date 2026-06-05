use voxel_orient::*;

#[test]
fn bijection_test() {
    for [base, orientation] in Rotation::cartesian_product() {
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
