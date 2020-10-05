use cgmath::*;
use num_traits::cast;

#[allow(dead_code)]
pub fn look_at_rh(eye_p : Point3<f32>, target : Point3<f32>, up : Vector3<f32>) -> Matrix4<f32>
{
	let dir = eye_p - target;
	let eye = eye_p.to_vec();

	let f = dir.normalize(); // forward
	let s = up.cross(f).normalize(); // side
	let u = f.cross(s); // up

	#[cfg_attr(rustfmt, rustfmt_skip)]
	Matrix4::new(
		s.x.clone(), u.x.clone(), f.x.clone(), 0.0,
		s.y.clone(), u.y.clone(), f.y.clone(), 0.0,
		s.z.clone(), u.z.clone(), f.z.clone(), 0.0,
		-s.dot(eye), -u.dot(eye), -f.dot(eye), 1.0,
	)
}

#[allow(dead_code)]
pub fn look_at_lh(eye_p : Point3<f32>, target : Point3<f32>, up : Vector3<f32>) -> Matrix4<f32>
{
	let dir = target - eye_p;
	let eye = eye_p.to_vec();

	let f = dir.normalize(); // forward
	let s = up.cross(f).normalize(); // side
	let u = f.cross(s); // up

	#[cfg_attr(rustfmt, rustfmt_skip)]
	Matrix4::new(
		s.x.clone(), u.x.clone(), f.x.clone(), 0.0,
		s.y.clone(), u.y.clone(), f.y.clone(), 0.0,
		s.z.clone(), u.z.clone(), f.z.clone(), 0.0,
		-s.dot(eye), -u.dot(eye), -f.dot(eye), 1.0,
	)
}

// This outputs a right-handed erspective matrix that ranges in depth from -1 to
// 1
#[allow(dead_code)]
pub fn _glm_perspective_rh<S : BaseFloat>(persp : PerspectiveFov<S>) -> Matrix4<S>
{
	let mut m = Matrix4::<S>::zero();

	let two : S = cast(2).unwrap();
	let f = Rad::tan(persp.fovy / two);

	let a = S::one() / (persp.aspect * f);
	let b = S::one() / f;
	let c = -(persp.far + persp.near) / (persp.far - persp.near);
	let d = -S::one();
	let e = -(two * persp.far * persp.near) / (persp.far - persp.near);
	// glm::perspectiveRH_NO and glm::perspectiveLH_NO both Negate the e term.

	m[0][0] = a;
	m[1][1] = b;
	m[2][2] = c;
	m[2][3] = d;
	m[3][2] = e;

	return m;
}

// This outputs a left-handed erspective matrix that ranges in depth from -1 to
// 1
#[allow(dead_code)]
pub fn _glm_perspective_lh<S : BaseFloat>(persp : PerspectiveFov<S>) -> Matrix4<S>
{
	let mut m = Matrix4::<S>::zero();

	let two : S = cast(2).unwrap();
	let f = Rad::tan(persp.fovy / two);

	let a = S::one() / (persp.aspect * f);
	let b = S::one() / f;
	let c = (persp.far + persp.near) / (persp.far - persp.near);
	let d = S::one();
	let e = -(two * persp.far * persp.near) / (persp.far - persp.near);
	// glm::perspectiveRH_NO and glm::perspectiveLH_NO both Negate the e term.

	m[0][0] = a;
	m[1][1] = b;
	m[2][2] = c;
	m[2][3] = d;
	m[3][2] = e;

	return m;
}

// This outputs a right-handed erspective matrix that ranges in depth from 0 to
// 1
#[allow(dead_code)]
pub fn _dx_perspective_rh<S : BaseFloat>(persp : PerspectiveFov<S>) -> Matrix4<S>
{
	let mut m = Matrix4::<S>::zero();

	let two : S = cast(2).unwrap();
	let f = Rad::tan(persp.fovy / two);
	let height = f;
	let width = height / persp.aspect;
	let f_range = persp.far / (persp.far - persp.near);

	m[0][0] = width;
	m[1][1] = height;
	m[2][2] = f_range;
	m[2][3] = -S::one();
	m[3][2] = f_range * persp.near;

	return m;
}

// This outputs a left-handed erspective matrix that ranges in depth from 0 to 1
#[allow(dead_code)]
pub fn _dx_perspective_lh<S : BaseFloat>(persp : PerspectiveFov<S>) -> Matrix4<S>
{
	let mut m = Matrix4::<S>::zero();

	let two : S = cast(2).unwrap();
	let f = Rad::tan(persp.fovy / two);
	let height = f;
	let width = height / persp.aspect;
	let f_range = persp.far / (persp.far - persp.near);

	m[0][0] = width;
	m[1][1] = height;
	m[2][2] = f_range;
	m[2][3] = S::one();
	m[3][2] = -f_range * persp.near;

	return m;
}

#[allow(dead_code)]
pub fn perspective_lh<S : BaseFloat>(persp : PerspectiveFov<S>) -> Matrix4<S>
{
	_glm_perspective_lh(persp)
}

#[allow(dead_code)]
pub fn perspective_rh<S : BaseFloat>(persp : PerspectiveFov<S>) -> Matrix4<S>
{
	_glm_perspective_rh(persp)
}

#[cfg(test)]
mod transform_tests
{
	use crate::transforms;
	use cgmath::*;
	use std::f32::consts::PI;

	#[allow(dead_code)]
	const TEST_EYE : Point3<f32> = Point3::new(0.0, 0.0, -2.0);
	#[allow(dead_code)]
	const TEST_TARGET : Point3<f32> = Point3::new(0.0, 0.0, 0.0);
	#[allow(dead_code)]
	const UP : Vector3<f32> = Vector3::new(0.0, 1.0, 0.0);

	const TEST_FOV_RADIANS : f32 = 70.0;
	const TEST_ASPECT_RATIO : f32 = 1280.0 / 720.0;
	const TEST_NEAR_PLANE : f32 = 0.1;
	const TEST_FAR_PLANE : f32 = 100.0;

	const TEST_PERSPECTIVE_FOV : PerspectiveFov<f32> = PerspectiveFov {
		fovy :   cgmath::Rad(TEST_FOV_RADIANS * (PI / 180.0f32)),
		aspect : TEST_ASPECT_RATIO,
		near :   TEST_NEAR_PLANE,
		far :    TEST_FAR_PLANE,
	};

	// calculated using floats
	#[allow(dead_code)]
	const EXPECTED_A : f32 = 0.803333223;
	#[allow(dead_code)]
	const EXPECTED_B : f32 = 1.42814803;
	#[allow(dead_code)]
	const EXPECTED_C : f32 = 1.00200200;
	#[allow(dead_code)]
	const EXPECTED_D : f32 = 1.00000000;
	#[allow(dead_code)]
	const EXPECTED_E : f32 = 0.200200200;

	// calculated using doubles and result in float (slightly more precise on a)
	#[allow(dead_code)]
	const EXPECTED_A_FROM_DBL : f32 = 0.803333282;
	#[allow(dead_code)]
	const EXPECTED_B_FROM_DBL : f32 = 1.42814803;
	#[allow(dead_code)]
	const EXPECTED_C_FROM_DBL : f32 = 1.00200200;
	#[allow(dead_code)]
	const EXPECTED_D_FROM_DBL : f32 = 1.00000000;
	#[allow(dead_code)]
	const EXPECTED_F_FROM_DBL : f32 = 0.200200200;

	// these are the same between DirectX and GLM

	#[allow(non_upper_case_globals)]
	const _expected_lookat_rh : Matrix4<f32> = Matrix4 {
		x : Vector4::<f32>::new(-1.0, 0.0, 0.0, 0.0),
		y : Vector4::<f32>::new(0.0, 1.0, 0.0, 0.0),
		z : Vector4::<f32>::new(0.0, 0.0, -1.0, 0.0),
		w : Vector4::<f32>::new(0.0, 0.0, -2.0, 1.0),
	};

	#[allow(non_upper_case_globals)]
	const _expected_lookat_lh : Matrix4<f32> = Matrix4 {
		x : Vector4::<f32>::new(1.0, 0.0, 0.0, 0.0),
		y : Vector4::<f32>::new(0.0, 1.0, 0.0, 0.0),
		z : Vector4::<f32>::new(0.0, 0.0, 1.0, 0.0),
		w : Vector4::<f32>::new(0.0, 0.0, 2.0, 1.0),
	};

	// these use the float calculation value.

	#[allow(non_upper_case_globals)]
	const _expected_dx_perspective_rh : Matrix4<f32> = Matrix4 {
		x : Vector4::<f32>::new(0.803333104, 0.0, 0.0, 0.0),
		y : Vector4::<f32>::new(0.0, 1.42814779, 0.0, 0.0),
		z : Vector4::<f32>::new(0.0, 0.0, -1.00100100, -1.0),
		w : Vector4::<f32>::new(0.0, 0.0, -0.100100100, 0.0),
	};

	#[allow(non_upper_case_globals)]
	const _expected_dx_perspective_lh : Matrix4<f32> = Matrix4 {
		x : Vector4::<f32>::new(0.803333104, 0.0, 0.0, 0.0),
		y : Vector4::<f32>::new(0.0, 1.42814779, 0.0, 0.0),
		z : Vector4::<f32>::new(0.0, 0.0, 1.00100100, 1.0),
		w : Vector4::<f32>::new(0.0, 0.0, -0.100100100, 0.0),
	};

	// DirectX is left-handed by default, but samples use right-handed.

	#[allow(non_upper_case_globals)]
	const _expected_glm_perspective_rh : Matrix4<f32> = Matrix4 {
		x : Vector4::<f32>::new(0.803333223, 0.0, 0.0, 0.0),
		y : Vector4::<f32>::new(0.0, 1.42814803, 0.0, 0.0),
		z : Vector4::<f32>::new(0.0, 0.0, -1.00200200, -1.0),
		w : Vector4::<f32>::new(0.0, 0.0, -0.200200200, 0.0),
	};

	#[allow(non_upper_case_globals)]
	const _expected_glm_perspective_lh : Matrix4<f32> = Matrix4 {
		x : Vector4::<f32>::new(0.803333223, 0.0, 0.0, 0.0),
		y : Vector4::<f32>::new(0.0, 1.42814803, 0.0, 0.0),
		z : Vector4::<f32>::new(0.0, 0.0, 1.00200200, 1.0),
		w : Vector4::<f32>::new(0.0, 0.0, -0.200200200, 0.0),
	};

	// glm::perspective uses right-handed by default
	#[allow(non_upper_case_globals)]
	const _expected_glm_perspective_default : Matrix4<f32> = _expected_glm_perspective_rh;

	#[test]
	fn it_works()
	{
		assert_eq!(2 + 2, 4);
	}

	#[test]
	fn test_view_lh()
	{
		let result = transforms::look_at_lh(TEST_EYE, TEST_TARGET, UP);
		assert_eq!(_expected_lookat_lh, result);
	}

	#[test]
	fn test_view_rh()
	{
		let result = transforms::look_at_rh(TEST_EYE, TEST_TARGET, UP);
		assert_eq!(_expected_lookat_rh, result);
	}

	#[test]
	fn test_perspective_lh()
	{
		let result = transforms::perspective_lh(TEST_PERSPECTIVE_FOV);
		assert_eq!(_expected_glm_perspective_lh, result);
	}

	#[test]
	fn test_perspective_rh()
	{
		let result = transforms::perspective_rh(TEST_PERSPECTIVE_FOV);
		assert_eq!(_expected_glm_perspective_rh, result);
	}
}
