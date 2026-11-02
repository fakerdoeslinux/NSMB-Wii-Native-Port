#[repr(C)]
#[derive(Clone, Copy)]
pub struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

#[no_mangle]
pub unsafe extern "C" fn aurora_rust_mtx_mult_vec_array(
    matrix: *const [f32; 4],
    src: *const Vec3,
    dst: *mut Vec3,
    count: u32,
) {
    let row0 = matrix.add(0);
    let row1 = matrix.add(1);
    let row2 = matrix.add(2);
    for i in 0..count as usize {
        let input = *src.add(i);
        let x = (*row0)[3]
            + ((*row0)[2] * input.z + ((*row0)[0] * input.x + (*row0)[1] * input.y));
        let y = (*row1)[3]
            + ((*row1)[2] * input.z + ((*row1)[0] * input.x + (*row1)[1] * input.y));
        let z = (*row2)[3]
            + ((*row2)[2] * input.z + ((*row2)[0] * input.x + (*row2)[1] * input.y));
        *dst.add(i) = Vec3 { x, y, z };
    }
}

#[cfg(test)]
mod tests {
    use super::aurora_rust_mtx_mult_vec_array;

    #[test]
    fn transforms_vectors_in_place_and_out_of_place() {
        let matrix = [
            [2.0, 3.0, 4.0, 5.0],
            [6.0, 7.0, 8.0, 9.0],
            [10.0, 11.0, 12.0, 13.0],
        ];
        let mut vectors = [
            super::Vec3 { x: 1.0, y: 2.0, z: 3.0 },
            super::Vec3 { x: -1.0, y: 0.5, z: 2.0 },
        ];
        let mut output = [super::Vec3 { x: 0.0, y: 0.0, z: 0.0 }; 2];

        unsafe {
            aurora_rust_mtx_mult_vec_array(matrix.as_ptr(), vectors.as_ptr(), output.as_mut_ptr(), 2);
            aurora_rust_mtx_mult_vec_array(matrix.as_ptr(), vectors.as_ptr(), vectors.as_mut_ptr(), 2);
        }

        assert_eq!((output[0].x, output[0].y, output[0].z), (25.0, 53.0, 81.0));
        assert_eq!((output[1].x, output[1].y, output[1].z), (12.5, 22.5, 32.5));
        assert_eq!((vectors[0].x, vectors[0].y, vectors[0].z), (25.0, 53.0, 81.0));
        assert_eq!((vectors[1].x, vectors[1].y, vectors[1].z), (12.5, 22.5, 32.5));
    }
}
